use crate::{
    data::Task,
    tui::{task_editor::TaskEditor, task_selector::TaskSelector, trait_panel::PanelWidget},
};
use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    prelude::{Buffer, Rect},
    text::Line,
    widgets::Widget,
};

pub enum TaskManagerRequest {
    OpenSelector,
    OpenEditor(Task),
}

pub trait TakeTaskManagerRequest {
    fn take_request(&mut self) -> Option<TaskManagerRequest>;
}

#[derive(Default)]
pub enum TaskManagerView {
    #[default]
    Selector,
    Editor,
}

pub struct TaskManager {
    view: TaskManagerView,

    selector: TaskSelector,
    editor: TaskEditor,

    focused: bool,
    selected: bool,
}

impl TaskManager {
    pub fn new() -> Self {
        let selector = TaskSelector::new();
        let editor = TaskEditor::new();

        Self {
            view: TaskManagerView::default(),
            focused: false,
            selected: false,

            selector,
            editor,
        }
    }

    pub fn get_selected_task(&self) -> Result<Task> {
        self.selector.get_selected_task()
    }

    fn change_view(&mut self, view: TaskManagerView) {
        self.view = view;

        match self.view {
            TaskManagerView::Selector => {
                self.selector.set_focused(self.is_focused());
                self.selector.set_selected(self.is_selected());
            }
            TaskManagerView::Editor => {
                self.editor.set_focused(self.is_focused());
                self.editor.set_selected(self.is_selected());
            }
        }
    }

    fn handle_request(&mut self, request: TaskManagerRequest) {
        match request {
            TaskManagerRequest::OpenSelector => {
                self.change_view(TaskManagerView::Selector);
            }
            TaskManagerRequest::OpenEditor(task) => {
                self.change_view(TaskManagerView::Editor);

                self.editor.editing_task = Some(task);
            }
        }
    }
}

impl PanelWidget for TaskManager {
    fn set_focused(&mut self, value: bool) {
        self.focused = value;

        match self.view {
            TaskManagerView::Selector => self.selector.set_focused(value),
            TaskManagerView::Editor => self.editor.set_focused(value),
        }
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_selected(&mut self, value: bool) {
        self.selected = value;

        match self.view {
            TaskManagerView::Selector => self.selector.set_selected(value),
            TaskManagerView::Editor => self.editor.set_selected(value),
        }
    }

    fn is_selected(&self) -> bool {
        self.selected
    }

    fn handle_input(&mut self, key: KeyEvent) {
        let request = match self.view {
            TaskManagerView::Selector => {
                self.selector.handle_input(key);
                self.selector.take_request()
            }
            TaskManagerView::Editor => {
                self.editor.handle_input(key);
                self.editor.take_request()
            }
        };

        let Some(request) = request else { return };

        self.handle_request(request);
    }

    fn get_available_keybinds(&self) -> Line<'static> {
        match self.view {
            TaskManagerView::Selector => self.selector.get_available_keybinds(),
            TaskManagerView::Editor => self.editor.get_available_keybinds(),
        }
    }
}

impl Widget for &mut TaskManager {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self.view {
            TaskManagerView::Selector => self.selector.render(area, buf),
            TaskManagerView::Editor => self.editor.render(area, buf),
        }
    }
}
