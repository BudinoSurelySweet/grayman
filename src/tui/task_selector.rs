use crate::{
    data::Task,
    generate_keybinds,
    serializer::{config::load_config, last_task::get_last_task_name},
    tui::{
        style::get_style_by_status,
        task_manager::{TakeTaskManagerRequest, TaskManagerRequest},
        trait_panel::PanelWidget,
    },
};
use anyhow::{Context, Result};
use crossterm::event::KeyCode;
use ratatui::{
    layout::Alignment,
    prelude::{Buffer, Rect},
    style::{Color, Stylize},
    symbols,
    text::Line,
    widgets::{Block, List, ListState, Padding, Paragraph, StatefulWidget, Widget, Wrap},
};

pub struct TaskSelector {
    pub task_list_state: ListState,
    selected_task_idx: usize,

    focused: bool,
    selected: bool,
    task_manager_request: Option<TaskManagerRequest>,
}

impl TaskSelector {
    pub fn new() -> Self {
        let mut task_list_state = ListState::default();
        let selected_task_idx;

        // Select the first selected task on open
        if let Ok(name) = get_last_task_name()
            && let Ok(config) = load_config()
        {
            let mut task_list = config.tasks.clone();
            task_list.sort_by(|a, b| a.name.cmp(&b.name));
            let index = task_list.iter().position(|task| task.name == name);

            task_list_state.select(index);
            selected_task_idx = index.unwrap_or_default();
        } else {
            task_list_state.select_first();
            selected_task_idx = 0;
        }

        Self {
            focused: false,
            selected: false,
            task_list_state,
            task_manager_request: None,
            selected_task_idx,
        }
    }

    pub fn get_selected_task(&self) -> Result<Task> {
        let config = load_config()?;

        let mut task_list = config.tasks.clone();

        task_list.sort_by(|a, b| a.name.cmp(&b.name));

        let task = task_list.get(self.selected_task_idx).context(format!(
            "There's no task in index \"{}\"",
            self.selected_task_idx
        ))?;

        Ok(task.clone())
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

        "Select" ["space"]:
        KeyCode::Char(' ') => {
            if let Some(index) = self.task_list_state.selected() {
                self.selected_task_idx = index;
            }
        }

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
        let block = Block::bordered()
            .title(Line::from(" Tasks ").alignment(Alignment::Center))
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
                let mut task_list: Vec<String> =
                    config.tasks.iter().map(|task| task.name.clone()).collect();

                task_list.sort();

                let task_list: Vec<Line> = task_list
                    .into_iter()
                    .enumerate()
                    .map(|(i, task)| {
                        Line::from(task).bg(if i == self.selected_task_idx {
                            Color::Black
                        } else {
                            Color::default()
                        })
                    })
                    .collect();

                StatefulWidget::render(
                    List::new(task_list)
                        .highlight_symbol(format!("{} ", symbols::DOT))
                        .highlight_style(get_style_by_status(self.selected, self.focused)),
                    inner_area,
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
