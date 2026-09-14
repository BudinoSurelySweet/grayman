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

use crate::{
    engine::multithread::{runner::run_task_with_deps, watcher::start_watcher},
    serializer::{config::load_config, last_task::save_last_task_name},
    tui::{
        panel_bottom_left::BottomLeftPanel, panel_right::RightPanel, panel_top_left::TopLeftPanel,
        trait_panel::PanelWidget,
    },
};

use super::panel_right::RightPanelStatus;

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

#[derive(Clone)]
enum CurrentPanel {
    TopLeft,
    BottomLeft,
    Right,
}

impl CurrentPanel {
    fn next(&mut self) {
        *self = match self {
            CurrentPanel::TopLeft => CurrentPanel::BottomLeft,
            CurrentPanel::BottomLeft => CurrentPanel::Right,
            CurrentPanel::Right => CurrentPanel::TopLeft,
        };
    }

    fn previous(&mut self) {
        *self = match self {
            CurrentPanel::TopLeft => CurrentPanel::Right,
            CurrentPanel::BottomLeft => CurrentPanel::TopLeft,
            CurrentPanel::Right => CurrentPanel::BottomLeft,
        };
    }

    fn jump(&mut self, direction: PanelJumpDirection) {
        type P = CurrentPanel;
        type D = PanelJumpDirection;

        let dont_jump = self.clone();

        *self = match (&self, direction) {
            (P::TopLeft, D::Left) => dont_jump,
            (P::TopLeft, D::Right) => CurrentPanel::Right,
            (P::TopLeft, D::Up) => dont_jump,
            (P::TopLeft, D::Down) => CurrentPanel::BottomLeft,
            (P::BottomLeft, D::Left) => dont_jump,
            (P::BottomLeft, D::Right) => CurrentPanel::Right,
            (P::BottomLeft, D::Up) => CurrentPanel::TopLeft,
            (P::BottomLeft, D::Down) => dont_jump,
            (P::Right, D::Left) => CurrentPanel::TopLeft,
            (P::Right, D::Right) => dont_jump,
            (P::Right, D::Up) => dont_jump,
            (P::Right, D::Down) => dont_jump,
        };
    }
}

pub struct State {
    running: bool,

    top_left_panel: TopLeftPanel,
    bottom_left_panel: BottomLeftPanel,
    right_panel: RightPanel,

    focused_panel: CurrentPanel,
    selected_panel: Option<CurrentPanel>,
    hide_left_panels: bool,
    task_mode: TaskMode,
    output_receiver: Option<Receiver<String>>,
    stop_watcher_flag: Option<Arc<AtomicBool>>,
    last_output_time: Option<Instant>,
}

impl State {
    pub fn new() -> Self {
        let top_left_panel = TopLeftPanel::new();
        let bottom_left_panel = BottomLeftPanel::new();
        let right_panel = RightPanel::new();

        let mut state = Self {
            running: true,
            focused_panel: CurrentPanel::TopLeft,
            selected_panel: None,
            hide_left_panels: false,
            task_mode: TaskMode::Run,
            output_receiver: None,
            stop_watcher_flag: None,
            last_output_time: None,

            top_left_panel,
            bottom_left_panel,
            right_panel,
        };

        state.set_focused_panel_focus(true);

        state
    }

    fn set_focused_panel_focus(&mut self, value: bool) {
        match self.focused_panel {
            CurrentPanel::TopLeft => self.top_left_panel.set_focused(value),
            CurrentPanel::BottomLeft => self.bottom_left_panel.set_focused(value),
            CurrentPanel::Right => self.right_panel.set_focused(value),
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

    fn deselect_panel(&mut self) {
        if let Some(panel) = &self.selected_panel {
            match panel {
                CurrentPanel::TopLeft => self.top_left_panel.set_selected(false),
                CurrentPanel::BottomLeft => self.bottom_left_panel.set_selected(false),
                CurrentPanel::Right => self.right_panel.set_selected(false),
            }

            self.selected_panel = None;
        }
    }

    fn select_panel(&mut self) {
        if self.selected_panel.is_none() {
            match self.focused_panel {
                CurrentPanel::TopLeft => self.top_left_panel.set_selected(true),
                CurrentPanel::BottomLeft => self.bottom_left_panel.set_selected(true),
                CurrentPanel::Right => self.right_panel.set_selected(true),
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
                            self.right_panel.status = RightPanelStatus::Off;
                            self.output_receiver = None;
                        }
                    }
                    Err(TryRecvError::Empty) => {
                        if let Some(last_output_time) = self.last_output_time
                            && last_output_time.elapsed() > Duration::from_millis(WAIT_TIME)
                        {
                            self.last_output_time = None;
                            self.right_panel.status = RightPanelStatus::Watching;
                        }
                    }
                    Ok(first_msg) => {
                        self.last_output_time = Some(Instant::now());
                        self.right_panel.update_output(first_msg);

                        for s in receiver.try_iter() {
                            self.right_panel.update_output(s);
                        }

                        self.right_panel.status = RightPanelStatus::Executing;
                    }
                };
            }

            terminal.draw(|f| self.render(f))?;

            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_input(key);
                }
            }
        }

        Ok(())
    }

    fn handle_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.running = false,

            _ if self.selected_panel.is_none() => match key.code {
                KeyCode::Char('c') => {
                    // TODO: Add a panel for confirmation
                    self.right_panel.clear_output();
                }
                KeyCode::Char('s') => match self.task_mode {
                    TaskMode::Run => {
                        let Ok(config) = load_config() else { return };
                        let Ok(task) = self.top_left_panel.get_selected_task() else {
                            return;
                        };

                        // TODO: Manage this error and show it to the user
                        let _ = save_last_task_name(&task.name);

                        self.output_receiver = run_task_with_deps(task, config).ok();
                    }
                    TaskMode::Watch => {
                        let Ok(config) = load_config() else { return };
                        let Ok(task) = self.top_left_panel.get_selected_task() else {
                            return;
                        };

                        // TODO: Manage this error and show it to the user
                        let _ = save_last_task_name(&task.name);

                        let Some((receiver, stop_flag)) = start_watcher(task, config).ok() else {
                            return;
                        };

                        self.output_receiver = Some(receiver);
                        self.stop_watcher_flag = Some(stop_flag);
                    }
                },
                KeyCode::Char('d') => {
                    if let Some(flag) = &self.stop_watcher_flag {
                        flag.store(true, Ordering::Relaxed);
                    }

                    self.stop_watcher_flag = None;
                }

                KeyCode::Char('f') => {
                    self.hide_left_panels = !self.hide_left_panels;

                    match self.focused_panel {
                        CurrentPanel::TopLeft | CurrentPanel::BottomLeft
                            if self.hide_left_panels =>
                        {
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

                    self.hide_left_panels = false;
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.jump_focus(PanelJumpDirection::Right);
                }

                // Select the current panel
                KeyCode::Char(' ') => self.select_panel(),

                _ => {}
            },

            // Deselect the current panel
            KeyCode::Esc => self.deselect_panel(),

            // Passthrough
            _ => match &self.selected_panel {
                Some(CurrentPanel::TopLeft) => self.top_left_panel.handle_input(key),
                Some(CurrentPanel::BottomLeft) => self.bottom_left_panel.handle_input(key),
                Some(CurrentPanel::Right) => self.right_panel.handle_input(key),
                _ => {}
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

        let left_panel_constraint = if self.hide_left_panels {
            if frame.area().width > min_width_for_hiding {
                Constraint::Length(4)
            } else {
                Constraint::Length(0)
            }
        } else {
            Constraint::Length(30)
        };
        let layout = Layout::horizontal([left_panel_constraint, Constraint::Fill(1)]).spacing(1);
        let [left, right] = center.layout(&layout);

        let layout = Layout::vertical([Constraint::Ratio(1, 2), Constraint::Fill(1)]);
        let [top_left, bottom_left] = left.layout(&layout);

        /*
         * Widgets
         */

        if self.hide_left_panels {
            if frame.area().width > min_width_for_hiding {
                let top_block = Block::bordered();
                let bottom_block = Block::bordered();

                frame.render_widget(top_block, top_left);
                frame.render_widget(bottom_block, bottom_left);
            }
        } else {
            frame.render_widget(&mut self.top_left_panel, top_left);
            frame.render_widget(&mut self.bottom_left_panel, bottom_left);
        }

        let available_keybinds = if let Some(panel) = &self.selected_panel {
            match panel {
                CurrentPanel::TopLeft => self.top_left_panel.get_available_keybinds(),
                CurrentPanel::BottomLeft => self.bottom_left_panel.get_available_keybinds(),
                CurrentPanel::Right => self.right_panel.get_available_keybinds(),
            }
        } else {
            Line::from_iter([
                Span::from(" Mode"),
                Span::styled(format!(" {} [m]", self.task_mode), Style::default().blue()),
                Span::from(" Select"),
                Span::styled(" [space]", Style::default().blue()),
                Span::from(" Clear"),
                Span::styled(" [c]", Style::default().blue()),
                Span::from(" Start"),
                Span::styled(" [s]", Style::default().blue()),
                Span::from(" Stop"),
                Span::styled(" [d]", Style::default().blue()),
                Span::from(" Fullscreen"),
                Span::styled(" [f]", Style::default().blue()),
                Span::from(" Quit"),
                Span::styled(" [q] ", Style::default().blue()),
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
        frame.render_widget(&mut self.right_panel, right);
    }
}
