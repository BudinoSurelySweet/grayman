use crate::{
    generate_keybinds,
    tui::{
        style::{get_color_by_status, get_style_by_status},
        trait_panel::PanelWidget,
    },
};
use ansi_to_tui::IntoText;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Margin},
    prelude::{Buffer, Rect},
    style::{Color, Stylize},
    text::{Line, Text},
    widgets::{
        Block, BorderType, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Widget, Wrap,
    },
};
use std::collections::VecDeque;

pub enum OutputViewerStatus {
    Off,
    Watching,
    Executing,
}

pub struct OutputViewer {
    pub max_output_len: usize,
    pub status: OutputViewerStatus,

    output: VecDeque<String>,
    scroll: u16,
    focused: bool,
    selected: bool,
}

impl OutputViewer {
    pub fn new() -> Self {
        Self {
            focused: false,
            selected: false,
            output: VecDeque::new(),
            scroll: 0,
            max_output_len: 2000,
            status: OutputViewerStatus::Off,
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

impl PanelWidget for OutputViewer {
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
            self.scroll = self.scroll.saturating_sub(1);
        },

        "Down" ["j/down"]:
        KeyCode::Char('j') | KeyCode::Down => {
            self.scroll = self.scroll.saturating_add(1);
        }
    }
}

impl Widget for &mut OutputViewer {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let (bottom_title, border_type) = match self.status {
            OutputViewerStatus::Off => ("", BorderType::Plain),
            OutputViewerStatus::Watching => (" Watching ", BorderType::HeavyDoubleDashed),
            OutputViewerStatus::Executing => (" Executing ", BorderType::HeavyDoubleDashed),
        };

        let block = Block::bordered()
            .border_style(get_style_by_status(self.selected, self.focused))
            .border_type(border_type)
            .padding(Padding::new(2, 2, 0, 0))
            .title(Line::from(" Output ").alignment(Alignment::Center))
            .title_bottom(
                Line::from(bottom_title)
                    .bg(get_color_by_status(self.selected, self.focused))
                    .fg(Color::Black)
                    .alignment(Alignment::Center),
            );

        let inner_area = block.inner(area);

        let total_lines: u16 = self.output.iter().map(|s| s.lines().count() as u16).sum();
        let viewport_height = inner_area.height;
        let max_scroll = total_lines.saturating_sub(viewport_height);

        // Block the scroll at the max scroll value
        self.scroll = self.scroll.min(max_scroll);

        // Coloring the output
        let text: Text = self
            .output
            .iter()
            .filter_map(|line| line.into_text().ok())
            .flat_map(|parsed_text| parsed_text.lines)
            .collect();

        // Render the output
        Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: false })
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
