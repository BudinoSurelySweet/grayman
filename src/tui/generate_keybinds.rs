#[macro_export]
macro_rules! generate_keybinds {
    (
        $self:ident,
        $(
            $label:literal [$shortcut:literal] : $pattern:pat => $action:block
        ),* $(,)?
    ) => {
        fn handle_input(&mut $self, key: crossterm::event::KeyEvent) {
            match key.code {
                $(
                    $pattern => $action,
                )*
                _ => {}
            }
        }

        fn get_available_keybinds(&$self) -> ratatui::text::Line<'static> {
            {
            	use $crate::tui::style::HIGHLIGHT_STYLE;

	            ratatui::text::Line::from_iter([
	                $(
	                    ratatui::text::Span::from($label),
	                    ratatui::text::Span::styled(concat!(" [", $shortcut, "] "), HIGHLIGHT_STYLE),
	                )*
	            ])
	            }
        }
    };
}
