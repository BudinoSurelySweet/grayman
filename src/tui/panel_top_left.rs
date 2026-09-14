use crate::serializer::last_task::get_last_task_name;
use crate::tui::trait_panel::PanelWidget;
use crate::{data::Task, serializer::config::load_config};
use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Offset},
    prelude::{Buffer, Rect, Span},
    style::Style,
    symbols,
    text::Line,
    widgets::{Block, List, ListState, Paragraph, StatefulWidget, Widget, Wrap},
};

pub struct TopLeftPanel {
    pub task_list_state: ListState,

    focused: bool,
    selected: bool,
}

impl TopLeftPanel {
    pub fn new() -> Self {
        let mut task_list_state = ListState::default();

        // Select the first selected task on open
        if let Ok(name) = get_last_task_name()
            && let Ok(config) = load_config()
        {
            let mut task_list = config.tasks.clone();
            task_list.sort_by(|a, b| a.name.cmp(&b.name));
            let index = task_list.iter().position(|task| task.name == name);

            task_list_state.select(index);
        } else {
            task_list_state.select_first()
        }

        Self {
            focused: false,
            selected: false,
            task_list_state,
        }
    }

    pub fn get_selected_task(&self) -> Result<Task> {
        let config = load_config()?;

        let mut task_list = config.tasks.clone();

        task_list.sort_by(|a, b| a.name.cmp(&b.name));

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

impl PanelWidget for TopLeftPanel {
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
            KeyCode::Char('k') | KeyCode::Up => {
                self.task_list_state.select_previous();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.task_list_state.select_next();
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

impl Widget for &mut TopLeftPanel {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Block::bordered()
            .title(Line::from(" Tasks ").alignment(Alignment::Center))
            .border_style(self.get_style())
            .render(area, buf);

        match load_config() {
            Err(_) => {
                Paragraph::new("There is no configuration.")
                    .wrap(Wrap { trim: true })
                    .render(area + Offset::new(2, 1), buf);
            }
            Ok(config) => {
                let mut task_list: Vec<String> =
                    config.tasks.iter().map(|task| task.name.clone()).collect();

                task_list.sort();

                StatefulWidget::render(
                    List::new(task_list)
                        .highlight_symbol(format!("{} ", symbols::DOT))
                        .highlight_style(self.get_style()),
                    area + Offset::new(2, 1),
                    buf,
                    &mut self.task_list_state,
                );
            }
        }
    }
}
