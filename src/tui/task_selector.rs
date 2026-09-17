use crate::{
    data::Task,
    generate_keybinds,
    serializer::{config::load_config, last_task::get_last_task_name},
    tui::{
        style::{HIGHLIGHT_STYLE, get_style_by_status},
        task_manager::{TakeTaskManagerRequest, TaskManagerRequest},
        trait_panel::PanelWidget,
    },
};
use anyhow::{Context, Result};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Offset},
    prelude::{Buffer, Rect},
    symbols,
    text::Line,
    widgets::{Block, List, ListState, Paragraph, StatefulWidget, Widget, Wrap},
};

pub struct TaskSelector {
    pub task_list_state: ListState,

    focused: bool,
    selected: bool,
    task_manager_request: Option<TaskManagerRequest>,
}

impl TaskSelector {
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
            task_manager_request: None,
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

impl PanelWidget for TaskSelector {
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

    generate_keybinds! { self,
        "Esc" ["esc"]:
        KeyCode::Esc => {},

        "Up" ["k/up"]:
        KeyCode::Char('k') | KeyCode::Up => {
            self.task_list_state.select_previous();
        },

        "Down" ["j/down"]:
        KeyCode::Char('j') | KeyCode::Down => {
            self.task_list_state.select_next();
        },

        // "Edit" ["e"]:
        // KeyCode::Char('e') => {
        //     if let Ok(task) = self.get_selected_task() {
        //         self.task_manager_request = Some(TaskManagerRequest::OpenEditor(task));
        //     }
        // }
    }
}

impl Widget for &mut TaskSelector {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Block::bordered()
            .title(Line::from(" Tasks ").alignment(Alignment::Center))
            .border_style(get_style_by_status(self.selected, self.focused))
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
                        .highlight_style(get_style_by_status(self.selected, self.focused)),
                    area + Offset::new(2, 1),
                    buf,
                    &mut self.task_list_state,
                );
            }
        }
    }
}

impl TakeTaskManagerRequest for TaskSelector {
    fn take_request(&mut self) -> Option<TaskManagerRequest> {
        self.task_manager_request.take()
    }
}
