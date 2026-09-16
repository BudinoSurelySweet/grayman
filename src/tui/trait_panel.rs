use crossterm::event::{KeyCode, KeyEvent};
use ratatui::text::Line;

pub trait PanelWidget {
    fn set_focused(&mut self, value: bool);

    fn _is_focused(&self) -> bool;

    fn set_selected(&mut self, value: bool);

    fn _is_selected(&self) -> bool;

    fn take_keybinds_control(&self) -> Option<Vec<KeyCode>>;

    fn handle_input(&mut self, key: KeyEvent);

    fn get_available_keybinds(&self) -> Line<'static>;
}
