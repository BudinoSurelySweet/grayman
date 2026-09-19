use super::output_viewer::OutputViewerStatus;
use crate::{
    engine::multithread::{runner::run_task_with_deps, watcher::start_watcher},
    serializer::{cache::Cache, config::load_config},
    tui::{
        env_editor::EnvEditor, output_viewer::OutputViewer, style::HIGHLIGHT_STYLE,
        task_manager::TaskManager, trait_panel::PanelWidget,
    },
};
use anyhow::Result;
use core::fmt;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph},
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, TryRecvError},
    },
    time::{Duration, Instant},
};

enum TaskMode {
    Run,
    Watch,
}

impl fmt::Display for TaskMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskMode::Run => write!(f, "Run"),
            TaskMode::Watch => write!(f, "Watch"),
        }
    }
}

enum PanelJumpDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, PartialEq)]
enum Panel {
    TopRight,
    BottomRight,
    Left,
}

impl Panel {
    fn next(&mut self) {
        *self = match self {
            Panel::TopRight => Panel::BottomRight,
            Panel::BottomRight => Panel::Left,
            Panel::Left => Panel::TopRight,
        };
    }

    fn previous(&mut self) {
        *self = match self {
            Panel::TopRight => Panel::Left,
            Panel::BottomRight => Panel::TopRight,
            Panel::Left => Panel::BottomRight,
        };
    }

    fn jump(&mut self, direction: PanelJumpDirection) {
        type P = Panel;
        type D = PanelJumpDirection;

        let panel = match (&self, direction) {
            (P::TopRight, D::Left) => Some(Panel::Left),
            (P::TopRight, D::Right) => None,
            (P::TopRight, D::Up) => None,
            (P::TopRight, D::Down) => Some(Panel::BottomRight),

            (P::BottomRight, D::Left) => Some(Panel::Left),
            (P::BottomRight, D::Right) => None,
            (P::BottomRight, D::Up) => Some(Panel::TopRight),
            (P::BottomRight, D::Down) => None,

            (P::Left, D::Left) => None,
            (P::Left, D::Right) => Some(Panel::TopRight),
            (P::Left, D::Up) => None,
            (P::Left, D::Down) => None,
        };

        let Some(panel) = panel else { return };

        *self = panel;
    }
}

pub struct State {
    running: bool,

    task_manager: TaskManager,
    env_editor: EnvEditor,
    output_viewer: OutputViewer,

    focused_panel: Panel,
    selected_panel: Option<Panel>,
    hide_left_panels: bool,
    task_mode: TaskMode,
    output_receiver: Option<Receiver<String>>,
    stop_watcher_flag: Option<Arc<AtomicBool>>,
    last_output_time: Option<Instant>,
}

impl State {
    pub fn new() -> Self {
        let task_manager = TaskManager::new();
        let env_editor = EnvEditor::new();
        let output_viewer = OutputViewer::new();

        let mut state = Self {
            running: true,
            focused_panel: Panel::Left,
            selected_panel: None,
            hide_left_panels: false,
            task_mode: TaskMode::Run,
            output_receiver: None,
            stop_watcher_flag: None,
            last_output_time: None,

            task_manager,
            env_editor,
            output_viewer,
        };

        state.set_focused_panel_focus(true);

        state
    }

    fn set_focused_panel_focus(&mut self, value: bool) {
        match self.focused_panel {
            Panel::TopRight => self.task_manager.set_focused(value),
            Panel::BottomRight => self.env_editor.set_focused(value),
            Panel::Left => self.output_viewer.set_focused(value),
        }
    }

    fn focus_next(&mut self) {
        self.set_focused_panel_focus(false);
        self.focused_panel.next();
        self.set_focused_panel_focus(true);
    }

    fn focus_previous(&mut self) {
        self.set_focused_panel_focus(false);
        self.focused_panel.previous();
        self.set_focused_panel_focus(true);
    }

    fn jump_focus(&mut self, direction: PanelJumpDirection) {
        self.set_focused_panel_focus(false);
        self.focused_panel.jump(direction);
        self.set_focused_panel_focus(true);
    }

    fn deselect_focused_panel(&mut self) {
        if let Some(panel) = &self.selected_panel {
            match panel {
                Panel::TopRight => self.task_manager.set_selected(false),
                Panel::BottomRight => self.env_editor.set_selected(false),
                Panel::Left => self.output_viewer.set_selected(false),
            }

            self.selected_panel = None;
        }
    }

    fn select_focused_panel(&mut self) {
        if self.selected_panel.is_none() {
            match self.focused_panel {
                Panel::TopRight => self.task_manager.set_selected(true),
                Panel::BottomRight => self.env_editor.set_selected(true),
                Panel::Left => self.output_viewer.set_selected(true),
            }

            self.selected_panel = Some(self.focused_panel.clone())
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            if let Some(receiver) = &self.output_receiver {
                const WAIT_TIME: u64 = 500;

                match receiver.try_recv() {
                    Err(TryRecvError::Disconnected) => {
                        let should_turn_off = match self.last_output_time {
                            Some(time) => time.elapsed() > Duration::from_millis(WAIT_TIME),
                            None => true,
                        };

                        if should_turn_off {
                            self.last_output_time = None;
                            self.output_viewer.status = OutputViewerStatus::Off;
                            self.output_receiver = None;
                        }
                    }
                    Err(TryRecvError::Empty) => {
                        if let Some(last_output_time) = self.last_output_time
                            && last_output_time.elapsed() > Duration::from_millis(WAIT_TIME)
                        {
                            self.last_output_time = None;
                            self.output_viewer.status = OutputViewerStatus::Watching;
                        }
                    }
                    Ok(first_msg) => {
                        self.last_output_time = Some(Instant::now());
                        self.output_viewer.update_output(first_msg);

                        for s in receiver.try_iter() {
                            self.output_viewer.update_output(s);
                        }

                        self.output_viewer.status = OutputViewerStatus::Executing;
                    }
                };
            }

            terminal.draw(|f| self.render(f))?;

            if !event::poll(Duration::from_millis(50))? {
                continue;
            }

            if let Event::Key(key) = event::read()? {
                self.handle_input(key);
            }
        }

        Ok(())
    }

    fn handle_input(&mut self, key: KeyEvent) {
        // Force to take control of the input from a sub-panel
        if let Some(keybinds) = match &self.selected_panel {
            Some(Panel::TopRight) => self.task_manager.take_keybinds_control(),
            Some(Panel::BottomRight) => self.env_editor.take_keybinds_control(),
            Some(Panel::Left) => self.output_viewer.take_keybinds_control(),
            None => None,
        } && keybinds.contains(&key.code)
        {
            match &self.selected_panel {
                Some(Panel::TopRight) => self.task_manager.handle_input(key),
                Some(Panel::BottomRight) => self.env_editor.handle_input(key),
                Some(Panel::Left) => self.output_viewer.handle_input(key),
                None => {}
            }

            return;
        }

        match key.code {
            KeyCode::Char('q') => self.running = false,

            _ if self.selected_panel.is_none() => match key.code {
                KeyCode::Char('c') => {
                    // TODO: Add a panel for confirmation
                    self.output_viewer.clear_output();
                }
                KeyCode::Char('s') => {
                    // Stop the watcher if it's on
                    if let Some(flag) = &self.stop_watcher_flag {
                        flag.store(true, Ordering::Relaxed);
                        self.stop_watcher_flag = None;

                        return;
                    }

                    let Ok(config) = load_config() else { return };
                    let Ok(task) = self.task_manager.get_selected_task() else {
                        return;
                    };
                    let cache = Cache {
                        last_task: Some(task.name.clone()),
                    };

                    // TODO: Manage this error and show it to the user
                    let _ = cache.save();

                    match self.task_mode {
                        TaskMode::Run => {
                            self.output_receiver = run_task_with_deps(task, config).ok();
                        }
                        TaskMode::Watch => {
                            let Some((receiver, stop_flag)) = start_watcher(task, config).ok()
                            else {
                                return;
                            };

                            self.output_receiver = Some(receiver);
                            self.stop_watcher_flag = Some(stop_flag);
                        }
                    }
                }

                KeyCode::Char('f') => {
                    self.hide_left_panels = !self.hide_left_panels;

                    match self.focused_panel {
                        Panel::TopRight | Panel::BottomRight if self.hide_left_panels => {
                            self.jump_focus(PanelJumpDirection::Right)
                        }
                        _ => {}
                    }
                }

                KeyCode::Char('m') => {
                    self.task_mode = match self.task_mode {
                        TaskMode::Run => TaskMode::Watch,
                        TaskMode::Watch => TaskMode::Run,
                    }
                }

                // Movement between panels
                KeyCode::BackTab => {
                    self.focus_previous();
                }
                KeyCode::Tab => {
                    self.focus_next();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.jump_focus(PanelJumpDirection::Up);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.jump_focus(PanelJumpDirection::Down);
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.jump_focus(PanelJumpDirection::Left);
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.jump_focus(PanelJumpDirection::Right);

                    self.hide_left_panels = false;
                }

                // Select the current panel
                KeyCode::Char(' ') => self.select_focused_panel(),

                _ => {}
            },

            // Deselect the current panel
            KeyCode::Esc => self.deselect_focused_panel(),

            // Passthrough
            _ => match &self.selected_panel {
                Some(Panel::TopRight) => self.task_manager.handle_input(key),
                Some(Panel::BottomRight) => self.env_editor.handle_input(key),
                Some(Panel::Left) => self.output_viewer.handle_input(key),
                None => {}
            },
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let min_width = 24;
        let min_height = 12;
        let min_width_for_hiding = 48;

        /*
         * Terminal is too small
         */

        if frame.area().width < min_width || frame.area().height < min_height {
            // Setup colors
            let w_color = if frame.area().width < min_width {
                Color::LightRed
            } else {
                Color::White
            };
            let h_color = if frame.area().height < min_height {
                Color::LightRed
            } else {
                Color::White
            };

            // Setup texts
            let text = Text::from(vec![
                Line::from("Terminal size too small"),
                Line::from(vec![
                    Span::raw("Width: "),
                    Span::styled(
                        format!("{:<2}", frame.area().width),
                        Style::default().fg(w_color),
                    ),
                    Span::raw("  Height: "),
                    Span::styled(
                        format!("{:<2}", frame.area().height),
                        Style::default().fg(h_color),
                    ),
                ]),
                Line::from(""),
                Line::from("Needed at least"),
                Line::from(format!("Width: {}  Height: {}", min_width, min_height)),
            ]);

            // Setup the layout
            let text_height = 5;
            let y_offset = frame.area().height.saturating_sub(text_height) / 2;

            let center_area = Rect {
                x: frame.area().x,
                y: frame.area().y + y_offset,
                width: frame.area().width,
                height: text_height,
            };

            frame.render_widget(
                Paragraph::new(text).alignment(Alignment::Center),
                center_area,
            );

            return;
        }

        /*
         * Main Layout
         */

        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);
        let [_, center, bottom] = frame.area().layout(&layout);

        let right_panel_constraint = if self.hide_left_panels {
            if frame.area().width > min_width_for_hiding {
                Constraint::Length(4)
            } else {
                Constraint::Length(0)
            }
        } else {
            Constraint::Length(30)
        };
        let layout = Layout::horizontal([Constraint::Fill(1), right_panel_constraint]).spacing(1);
        let [left, right] = center.layout(&layout);

        let layout = Layout::vertical([Constraint::Ratio(1, 2), Constraint::Fill(1)]);
        let [top_right, bottom_right] = right.layout(&layout);

        /*
         * Widgets
         */

        if self.hide_left_panels {
            if frame.area().width > min_width_for_hiding {
                let top_block = Block::bordered();
                let bottom_block = Block::bordered();

                frame.render_widget(top_block, top_right);
                frame.render_widget(bottom_block, bottom_right);
            }
        } else {
            frame.render_widget(&mut self.task_manager, top_right);
            frame.render_widget(&mut self.env_editor, bottom_right);
        }

        let available_keybinds = if let Some(panel) = &self.selected_panel {
            match panel {
                Panel::TopRight => self.task_manager.get_available_keybinds(),
                Panel::BottomRight => self.env_editor.get_available_keybinds(),
                Panel::Left => self.output_viewer.get_available_keybinds(),
            }
        } else {
            let start_stop_label = if self.stop_watcher_flag.is_some() {
                " Stop"
            } else {
                " Start"
            };

            Line::from_iter([
                Span::from(" Mode"),
                Span::styled(format!(" {} [m]", self.task_mode), HIGHLIGHT_STYLE),
                Span::from(start_stop_label),
                Span::styled(" [s]", HIGHLIGHT_STYLE),
                Span::from(" Clear"),
                Span::styled(" [c]", HIGHLIGHT_STYLE),
                Span::from(" Select"),
                Span::styled(" [space]", HIGHLIGHT_STYLE),
                Span::from(" Fullscreen"),
                Span::styled(" [f]", HIGHLIGHT_STYLE),
                Span::from(" Quit"),
                Span::styled(" [q] ", HIGHLIGHT_STYLE),
            ])
        };

        const VERSION: &str = env!("CARGO_PKG_VERSION");

        let title_and_version_and_global_keybinds = Line::from_iter([
            Span::styled(" Grayman ", Style::default().bold()),
            Span::styled(format!("v{}        ", VERSION), Style::default().gray()),
        ]);

        if frame.area().width > min_width_for_hiding {
            frame.render_widget(available_keybinds.alignment(Alignment::Right), bottom);
        }

        frame.render_widget(title_and_version_and_global_keybinds, bottom);

        if !self.hide_left_panels && frame.area().width < min_width_for_hiding {
            let block = Block::bordered();

            frame.render_widget(block, left);
        } else {
            frame.render_widget(&mut self.output_viewer, left);
        }
    }
}
