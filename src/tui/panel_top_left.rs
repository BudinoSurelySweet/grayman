use crate::serializer::config::load_config;
use crate::tui::trait_panel::Panel;
use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Offset,
    prelude::{Frame, Rect, Span},
    style::Style,
    symbols,
    text::Line,
    widgets::{Block, List, ListState, Paragraph, Wrap},
};

pub struct TopLeftPanel {
    focused: bool,
    selected: bool,
    task_list_state: ListState,
}

impl TopLeftPanel {
    pub fn new() -> Self {
        let mut task_list_state = ListState::default();
        task_list_state.select_first();

        Self {
            focused: false,
            selected: false,
            task_list_state,
        }
    }

    pub fn get_selected_task_name(&self) -> Result<String> {
        let config = load_config()?;

        let mut task_list: Vec<String> =
            config.tasks.iter().map(|task| task.name.clone()).collect();

        task_list.sort();

        let index = self
            .task_list_state
            .selected()
            .context("No selected task")?;
        let task = task_list
            .get(index)
            .context(format!("There's no task in index \"{}\"", index))?;

        return Ok(task.clone());
    }
}

impl Panel for TopLeftPanel {
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
                self.task_list_state.select_previous();
            }
            KeyCode::Char('j') => {
                self.task_list_state.select_next();
            }
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered().border_style(self.get_style());
        let title = Paragraph::new(" Tasks ").centered();

        frame.render_widget(block, area);
        frame.render_widget(title, area);

        match load_config() {
            Err(_) => {
                let fallback =
                    Paragraph::new("There is no configuration.").wrap(Wrap { trim: true });
                frame.render_widget(fallback, area + Offset::new(2, 1));
            }
            Ok(config) => {
                let mut task_list: Vec<String> =
                    config.tasks.iter().map(|task| task.name.clone()).collect();

                task_list.sort();

                let task_list = List::new(task_list)
                    .highlight_symbol(format!("{} ", symbols::DOT))
                    .highlight_style(self.get_style());

                frame.render_stateful_widget(
                    task_list,
                    area + Offset::new(2, 1),
                    &mut self.task_list_state,
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
