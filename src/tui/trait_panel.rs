use crossterm::event::KeyEvent;
use ratatui::{style::Style, text::Line};

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

    fn set_focused(&mut self, value: bool);

    fn is_focused(&self) -> bool;

    fn set_selected(&mut self, value: bool);

    fn is_selected(&self) -> bool;

    fn handle_input(&mut self, key: KeyEvent);

    fn get_available_keybinds(&self) -> Line<'static>;
}
