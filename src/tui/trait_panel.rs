use crossterm::event::KeyEvent;
use ratatui::{
    style::{Color, Style},
    text::Line,
};

pub trait PanelWidget {
    fn get_style(&self) -> Style {
        if self.is_selected() {
            Style::default().green()
        } else if self.is_focused() {
            Style::default().blue()
        } else {
            Style::default()
        }
    }

    fn get_color(&self) -> Color {
        if self.is_selected() {
            Color::Green
        } else if self.is_focused() {
            Color::Blue
        } else {
            Color::Gray
        }
    }

    fn set_focused(&mut self, value: bool);

    fn is_focused(&self) -> bool;

    fn set_selected(&mut self, value: bool);

    fn is_selected(&self) -> bool;

    fn handle_input(&mut self, key: KeyEvent);

    fn get_available_keybinds(&self) -> Line<'static>;
}
