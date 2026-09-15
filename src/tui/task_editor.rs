use super::task_manager::TaskManagerRequest;
use crate::{
    data::Task,
    tui::{
        style::{HIGHLIGHT_STYLE, get_style_by_status},
        task_manager::TakeTaskManagerRequest,
        trait_panel::PanelWidget,
    },
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Alignment,
    prelude::{Buffer, Rect},
    text::{Line, Span},
    widgets::{Block, Widget},
};

pub struct TaskEditor {
    pub editing_task: Option<Task>,

    focused: bool,
    selected: bool,
    task_manager_request: Option<TaskManagerRequest>,
}

impl TaskEditor {
    pub fn new() -> Self {
        Self {
            focused: false,
            selected: false,
            task_manager_request: None,
            editing_task: None,
        }
    }
}

impl PanelWidget for TaskEditor {
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
            // TODO: Change this keybind
            KeyCode::Char('b') => {
                self.task_manager_request = Some(TaskManagerRequest::OpenSelector)
            }
            _ => {}
        }
    }

    fn get_available_keybinds(&self) -> Line<'static> {
        Line::from_iter([Span::from(""), Span::styled("", HIGHLIGHT_STYLE)])
    }
}

impl Widget for &mut TaskEditor {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Block::bordered()
            .title(Line::from(" Editor ").alignment(Alignment::Center))
            .border_style(get_style_by_status(self.selected, self.focused))
            .render(area, buf);
    }
}

impl TakeTaskManagerRequest for TaskEditor {
    fn take_request(&mut self) -> Option<TaskManagerRequest> {
        self.task_manager_request.take()
    }
}
