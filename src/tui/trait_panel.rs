use crossterm::event::KeyEvent;
use ratatui::text::Line;

pub trait PanelWidget {
    fn set_focused(&mut self, value: bool);

    fn is_focused(&self) -> bool;

    fn set_selected(&mut self, value: bool);

    fn is_selected(&self) -> bool;

    fn handle_input(&mut self, key: KeyEvent);

    fn get_available_keybinds(&self) -> Line<'static>;
}
