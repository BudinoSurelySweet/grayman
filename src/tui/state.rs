use anyhow::Result;
use core::fmt;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Layout},
    style::Style,
    text::{Line, Span},
    widgets::Block,
};

use crate::tui::{
    panel_bottom_left::BottomLeftPanel, panel_right::RightPanel, panel_top_left::TopLeftPanel,
    trait_panel::Panel,
};

enum TaskMode {
    Forge,
    Watch,
}

impl fmt::Display for TaskMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskMode::Forge => write!(f, "Forge"),
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
            task_mode: TaskMode::Forge,

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
            terminal.draw(|f| self.render(f))?;

            if let Event::Key(key) = event::read()? {
                self.handle_input(key);
            }
        }

        Ok(())
    }

    fn handle_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.running = false,

            _ if self.selected_panel.is_none() => match key.code {
                KeyCode::Char('s') => match self.task_mode {
                    TaskMode::Forge => todo!(),
                    TaskMode::Watch => todo!(),
                },

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
                        TaskMode::Forge => TaskMode::Watch,
                        TaskMode::Watch => TaskMode::Forge,
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
        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);
        let [_, center, bottom] = frame.area().layout(&layout);

        let left_panels_len = if self.hide_left_panels { 4 } else { 30 };
        let layout = Layout::horizontal([Constraint::Length(left_panels_len), Constraint::Fill(1)])
            .spacing(1);
        let [left, right] = center.layout(&layout);

        let layout = Layout::vertical([Constraint::Ratio(1, 2), Constraint::Fill(1)]);
        let [top_left, bottom_left] = left.layout(&layout);

        let title_and_version_and_global_keybinds = Line::from_iter([
            Span::styled("Faber ", Style::default().bold()),
            Span::styled("v0.1.0", Style::default().gray()),
            Span::from("      Quit"),
            Span::styled(" [q] ", Style::default().blue()),
        ]);

        if self.hide_left_panels {
            let top_block = Block::bordered();
            let bottom_block = Block::bordered();

            frame.render_widget(top_block, top_left);
            frame.render_widget(bottom_block, bottom_left);
        } else {
            let _ = self.top_left_panel.render(frame, top_left);
            let _ = self.bottom_left_panel.render(frame, bottom_left);
        }

        let available_keybinds = if let Some(panel) = &self.selected_panel {
            match panel {
                CurrentPanel::TopLeft => self.top_left_panel.get_available_keybinds(),
                CurrentPanel::BottomLeft => self.bottom_left_panel.get_available_keybinds(),
                CurrentPanel::Right => self.right_panel.get_available_keybinds(),
            }
        } else {
            let selected_task = self
                .top_left_panel
                .get_selected_task_name()
                .unwrap_or(String::from("No task"));

            Line::from_iter([
                Span::from(" Task"),
                Span::styled(format!(" {}", selected_task), Style::default().blue()),
                Span::from(" Mode"),
                Span::styled(format!(" {} [m]", self.task_mode), Style::default().blue()),
                Span::from(" Start"),
                Span::styled(" [s]", Style::default().blue()),
                Span::from(" Fullscreen"),
                Span::styled(" [f] ", Style::default().blue()),
            ])
        };

        frame.render_widget(title_and_version_and_global_keybinds, bottom);
        frame.render_widget(available_keybinds.alignment(Alignment::Right), bottom);

        let _ = self.right_panel.render(frame, right);
    }
}
