use crate::{serializer::config::load_config, tui::trait_panel::PanelWidget};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Offset},
    prelude::{Buffer, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Paragraph, Row, StatefulWidget, Table, TableState, Widget, Wrap},
};

pub struct BottomLeftPanel {
    pub variable_table_state: TableState,

    focused: bool,
    selected: bool,
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

impl PanelWidget for BottomLeftPanel {
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

    fn get_available_keybinds(&self) -> Line<'static> {
        Line::from_iter([
            Span::from(" Exit"),
            Span::styled(" [esc]", Style::default().blue()),
            Span::from(" Down"),
            Span::styled(" [j/Down]", Style::default().blue()),
            Span::from(" Up"),
            Span::styled(" [k/Up] ", Style::default().blue()),
        ])
    }
}

impl Widget for &mut BottomLeftPanel {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Block::bordered()
            .title(Line::from(" Variables ").alignment(Alignment::Center))
            .border_style(self.get_style())
            .render(area, buf);

        match load_config() {
            Err(_) => {
                Paragraph::new("There is no configuration.")
                    .wrap(Wrap { trim: true })
                    .render(area + Offset::new(2, 1), buf);
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

                StatefulWidget::render(
                    table,
                    area + Offset::new(2, 1),
                    buf,
                    &mut self.variable_table_state,
                );
            }
        }
    }
}
