use crate::tui::trait_panel::PanelWidget;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Margin},
    prelude::{Buffer, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{
        Block, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget,
        Widget,
    },
};
use std::collections::VecDeque;

pub struct RightPanel {
    pub max_output_len: usize,

    output: VecDeque<String>,
    scroll: u16,
    focused: bool,
    selected: bool,
}

impl RightPanel {
    pub fn new() -> Self {
        Self {
            focused: false,
            selected: false,
            output: VecDeque::new(),
            scroll: 0,
            max_output_len: 2000,
        }
    }

    pub fn update_output(&mut self, s: String) {
        if self.output.len() > self.max_output_len {
            self.output.pop_front();
        }

        self.output.push_back(s);
        self.scroll = u16::MAX;
    }

    pub fn clear_output(&mut self) {
        self.output.clear();
    }
}

impl PanelWidget for RightPanel {
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
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll = self.scroll.saturating_add(1);
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

impl Widget for &mut RightPanel {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered()
            .title(Line::from(" Console ").alignment(Alignment::Center))
            .border_style(self.get_style())
            .padding(Padding::new(2, 2, 0, 0));

        let inner_area = block.inner(area);

        let total_lines: u16 = self.output.iter().map(|s| s.lines().count() as u16).sum();
        let viewport_height = inner_area.height;
        let max_scroll = total_lines.saturating_sub(viewport_height);

        // Block the scroll at the max scroll value
        self.scroll = self.scroll.min(max_scroll);

        let text_lines: Vec<Line> = self.output.iter().map(|s| Line::from(s.as_str())).collect();

        // Render the outputs
        Paragraph::new(text_lines)
            .block(block)
            .scroll((self.scroll, 0))
            .render(area, buf);

        // Render the scrollbar
        if total_lines > viewport_height {
            let mut scrollbar_state = ScrollbarState::default()
                .content_length(max_scroll as usize)
                .position(self.scroll as usize);

            StatefulWidget::render(
                Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("▲"))
                    .end_symbol(Some("▼")),
                inner_area.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                buf,
                &mut scrollbar_state,
            );
        }
    }
}
