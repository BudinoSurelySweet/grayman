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
    widgets::{Block, Padding, Widget},
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
        if self.selected {
            return Some(vec![KeyCode::Esc]);
        }

        None
    }

    fn handle_input(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.task_manager_request = Some(TaskManagerRequest::OpenSelector)
        }
    }

    fn get_available_keybinds(&self) -> Line<'static> {
        Line::from_iter([
            Span::from(" Exit"),
            Span::styled(" [esc] ", HIGHLIGHT_STYLE),
        ])
    }
}

impl Widget for &mut TaskEditor {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered()
            .title(Line::from(" Editor ").alignment(Alignment::Center))
            .border_style(get_style_by_status(self.selected, self.focused))
            .padding(Padding::new(1, 1, 0, 0));

        let _inner_area = block.inner(area);

        block.render(area, buf);
    }
}

impl TakeTaskManagerRequest for TaskEditor {
    fn take_request(&mut self) -> Option<TaskManagerRequest> {
        self.task_manager_request.take()
    }
}
