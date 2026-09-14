use ratatui::style::{Color, Style};

pub const SELECTED_STYLE: Style = Style::new().green();
pub const FOCUSED_STYLE: Style = Style::new().blue();
pub const DEFAULT_STYLE: Style = Style::new();

pub const SELECTED_COLOR: Color = Color::Green;
pub const FOCUSED_COLOR: Color = Color::Blue;
pub const DEFAULT_COLOR: Color = Color::Blue;

pub fn get_style_by_status(selected: bool, focused: bool) -> Style {
    if selected {
        SELECTED_STYLE
    } else if focused {
        FOCUSED_STYLE
    } else {
        DEFAULT_STYLE
    }
}

pub fn get_color_by_status(selected: bool, focused: bool) -> Color {
    if selected {
        SELECTED_COLOR
    } else if focused {
        FOCUSED_COLOR
    } else {
        DEFAULT_COLOR
    }
}
