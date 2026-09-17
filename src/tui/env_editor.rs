use crate::{
    generate_keybinds,
    serializer::config::load_config,
    tui::{
        style::{HIGHLIGHT_STYLE, get_style_by_status},
        trait_panel::PanelWidget,
    },
};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint},
    prelude::{Buffer, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{Block, Padding, Paragraph, Row, StatefulWidget, Table, TableState, Widget, Wrap},
};

pub struct EnvEditor {
    pub variable_table_state: TableState,

    focused: bool,
    selected: bool,
}

impl EnvEditor {
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

impl PanelWidget for EnvEditor {
    fn set_focused(&mut self, value: bool) {
        self.focused = value
    }

    fn _is_focused(&self) -> bool {
        self.focused
    }

    fn set_selected(&mut self, value: bool) {
        self.selected = value
    }

    fn _is_selected(&self) -> bool {
        self.selected
    }

    fn take_keybinds_control(&self) -> Option<Vec<KeyCode>> {
        None
    }

    generate_keybinds! {self,
        "Exit" ["esc"]:
        KeyCode::Esc => {},

        "Up" ["k/up"]:
        KeyCode::Char('k') | KeyCode::Up => {
            self.variable_table_state.select_previous();
        },

        "Down" ["j/down"]:
        KeyCode::Char('j') | KeyCode::Down => {
            self.variable_table_state.select_next();
        }
    }
}

impl Widget for &mut EnvEditor {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered()
            .title(Line::from(" Env ").alignment(Alignment::Center))
            .padding(Padding::new(1, 1, 0, 0))
            .border_style(get_style_by_status(self.selected, self.focused));

        let inner_area = block.inner(area);

        block.render(area, buf);

        match load_config() {
            Err(_) => {
                Paragraph::new("There is no configuration.")
                    .wrap(Wrap { trim: true })
                    .render(inner_area, buf);
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
                    .enumerate()
                    .map(|(i, (name, value))| {
                        Row::new(vec![name.clone(), value.clone()]).bg(if i % 2 == 0 {
                            Color::Black
                        } else {
                            Color::default()
                        })
                    })
                    .collect();

                let widths = [Constraint::Percentage(40), Constraint::Fill(1)];
                let table = Table::new(rows, widths)
                    .header(Row::new(["Key", "Value"]).bold())
                    .row_highlight_style(get_style_by_status(self.selected, self.focused));

                StatefulWidget::render(table, inner_area, buf, &mut self.variable_table_state);
            }
        }
    }
}
