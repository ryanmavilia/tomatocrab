//! Task input widget for entering task descriptions

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::theme::{Theme, ACCENT, BORDER, HIGHLIGHT, PRIMARY, TEXT_BRIGHT};

/// Widget for entering task description
pub struct TaskInputWidget<'a> {
    app: &'a App,
}

impl<'a> TaskInputWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }

    /// Render the task input widget
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Main container with border
        let outer_block = Block::default()
            .borders(Borders::ALL)
            .border_style(ratatui::style::Style::default().fg(BORDER))
            .title(" TOMATOCRAB ")
            .title_style(
                ratatui::style::Style::default()
                    .fg(PRIMARY)
                    .add_modifier(Modifier::BOLD),
            )
            .title_alignment(Alignment::Center);

        let inner_area = outer_block.inner(area);
        frame.render_widget(outer_block, area);

        let chunks = Layout::vertical([
            Constraint::Min(2),     // Spacer
            Constraint::Length(2),  // Prompt
            Constraint::Length(1),  // Spacer
            Constraint::Length(3),  // Input field
            Constraint::Min(2),     // Spacer
            Constraint::Length(2),  // Hints
        ])
        .split(inner_area);

        self.render_prompt(frame, chunks[1]);
        self.render_input(frame, chunks[3]);
        self.render_hints(frame, chunks[5]);
    }

    fn render_prompt(&self, frame: &mut Frame, area: Rect) {
        let prompt = Paragraph::new("What are you working on?")
            .style(Theme::subtitle())
            .alignment(Alignment::Center);
        frame.render_widget(prompt, area);
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        // Center the input box
        let input_width = area.width.min(60);
        let horizontal_padding = (area.width.saturating_sub(input_width)) / 2;

        let centered_area = Rect {
            x: area.x + horizontal_padding,
            y: area.y,
            width: input_width,
            height: area.height,
        };

        // Blinking cursor effect
        let input_text = format!("{}|", self.app.task_description);

        let input = Paragraph::new(input_text)
            .style(ratatui::style::Style::default().fg(TEXT_BRIGHT))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(ratatui::style::Style::default().fg(HIGHLIGHT))
                    .title(" Task Description ")
                    .title_style(
                        ratatui::style::Style::default()
                            .fg(ACCENT)
                            .add_modifier(Modifier::BOLD),
                    ),
            );
        frame.render_widget(input, centered_area);
    }

    fn render_hints(&self, frame: &mut Frame, area: Rect) {
        let hints = vec![("Enter", "Start Timer"), ("Esc", "Cancel")];

        let hint_spans: Vec<Span> = hints
            .iter()
            .enumerate()
            .flat_map(|(i, (key, action))| {
                let mut spans = vec![
                    Span::styled(format!("[{}]", key), Theme::key_hint()),
                    Span::raw(" "),
                    Span::styled(*action, Theme::key_action()),
                ];
                if i < hints.len() - 1 {
                    spans.push(Span::raw("  "));
                }
                spans
            })
            .collect();

        let hints_paragraph = Paragraph::new(Line::from(hint_spans)).alignment(Alignment::Center);
        frame.render_widget(hints_paragraph, area);
    }
}
