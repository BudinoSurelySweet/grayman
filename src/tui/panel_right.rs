use crate::tui::trait_panel::Panel;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::Alignment,
    prelude::{Frame, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Block,
};

pub struct RightPanel {
    focused: bool,
    selected: bool,
}

impl RightPanel {
    pub fn new() -> Self {
        Self {
            focused: false,
            selected: false,
        }
    }
}

impl Panel for RightPanel {
    fn set_focused(&mut self, value: bool) {
        self.focused = value
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_selected(&mut self, value: bool) {
        self.selected = value
    }

    fn is_selected(&self) -> bool {
        self.selected
    }

    fn handle_input(&mut self, _key: KeyEvent) {}

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .title(Line::from(" Console ").alignment(Alignment::Center))
            .border_style(self.get_style());

        frame.render_widget(block, area);
    }

    fn get_available_keybinds(&self) -> Line<'static> {
        Line::from_iter([Span::from(""), Span::styled("", Style::default().blue())])
    }
}
