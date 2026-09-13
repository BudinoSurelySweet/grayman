use crate::{serializer::config::load_config, tui::trait_panel::Panel};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Offset},
    prelude::{Frame, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Paragraph, Row, Table, TableState, Wrap},
};

pub struct BottomLeftPanel {
    focused: bool,
    selected: bool,
    variable_table_state: TableState,
}

impl BottomLeftPanel {
    pub fn new() -> Self {
        let mut variable_table_state = TableState::default();
        variable_table_state.select_first();

        Self {
            variable_table_state,

            focused: false,
            selected: false,
        }
    }
}

impl Panel for BottomLeftPanel {
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

    fn handle_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('k') => {
                self.variable_table_state.select_previous();
            }
            KeyCode::Char('j') => {
                self.variable_table_state.select_next();
            }
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered().border_style(self.get_style());
        let title = Paragraph::new(" Variables ").centered();

        frame.render_widget(block, area);
        frame.render_widget(title, area);

        match load_config() {
            Err(_) => {
                let fallback =
                    Paragraph::new("There is no configuration.").wrap(Wrap { trim: true });
                frame.render_widget(fallback, area + Offset::new(2, 1));
            }
            Ok(config) => {
                let env = config.env.unwrap_or_default();
                let mut rows: Vec<(String, String)> = env
                    .iter()
                    .map(|(name, value)| (name.clone(), value.clone()))
                    .collect();

                rows.sort();

                let rows: Vec<Row> = rows
                    .iter()
                    .map(|(name, value)| Row::new(vec![name.clone(), value.clone()]))
                    .collect();

                let widths = [Constraint::Percentage(40), Constraint::Fill(1)];
                let table = Table::new(rows, widths).row_highlight_style(self.get_style());

                frame.render_stateful_widget(
                    table,
                    area + Offset::new(2, 1),
                    &mut self.variable_table_state,
                );
            }
        }
    }

    fn get_available_keybinds(&self) -> Line<'static> {
        Line::from_iter([
            Span::from(" Down"),
            Span::styled(" [j]", Style::default().blue()),
            Span::from(" Up"),
            Span::styled(" [k] ", Style::default().blue()),
        ])
    }
}
