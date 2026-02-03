//! History view widget displaying past sessions in a table

use chrono::Local;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use crate::app::App;
use crate::theme::{Theme, BORDER, PRIMARY};

/// Widget for displaying session history
pub struct HistoryWidget<'a> {
    app: &'a App,
}

impl<'a> HistoryWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }

    /// Render the history widget
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::vertical([
            Constraint::Min(5),     // Table
            Constraint::Length(2),  // Hints
        ])
        .split(area);

        self.render_table(frame, chunks[0]);
        self.render_hints(frame, chunks[1]);
    }

    fn render_table(&self, frame: &mut Frame, area: Rect) {
        let filtered = self.app.filtered_sessions();

        // Create header
        let header_cells = ["Date", "Time", "Task", "Duration", "Status"]
            .iter()
            .map(|h| {
                Cell::from(*h).style(Theme::table_header())
            });
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        // Create rows (reversed to show most recent first)
        let rows: Vec<Row> = filtered
            .iter()
            .rev()
            .enumerate()
            .map(|(idx, session)| {
                let local_time = session.started_at.with_timezone(&Local);
                let date = local_time.format("%Y-%m-%d").to_string();
                let time = local_time.format("%H:%M").to_string();
                let duration = format_duration(session.duration_secs);

                let status_cell = if session.completed {
                    Cell::from("Completed").style(Theme::status_completed())
                } else {
                    Cell::from("Interrupted").style(Theme::status_interrupted())
                };

                // Truncate task if too long
                let task = if session.task.len() > 30 {
                    format!("{}...", &session.task[..27])
                } else {
                    session.task.clone()
                };

                let row_style = if idx == self.selected_index(&filtered) {
                    Theme::table_row_selected()
                } else {
                    Theme::table_row()
                };

                Row::new(vec![
                    Cell::from(date),
                    Cell::from(time),
                    Cell::from(task),
                    Cell::from(duration),
                    status_cell,
                ])
                .style(row_style)
            })
            .collect();

        let title = format!(" Session History ({}) ", self.app.filter_label());

        let table = Table::new(
            rows,
            [
                Constraint::Length(12),  // Date
                Constraint::Length(8),   // Time
                Constraint::Min(20),     // Task
                Constraint::Length(10),  // Duration
                Constraint::Length(12),  // Status
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(ratatui::style::Style::default().fg(BORDER))
                .title(title)
                .title_style(
                    ratatui::style::Style::default()
                        .fg(PRIMARY)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .row_highlight_style(Theme::table_row_selected());

        // Render with selection state
        let mut state = TableState::default();
        state.select(Some(self.selected_index(&filtered)));
        frame.render_stateful_widget(table, area, &mut state);
    }

    fn selected_index(&self, filtered: &[&crate::session::Session]) -> usize {
        // Reverse the index since we display in reverse order
        let len = filtered.len();
        if len == 0 {
            0
        } else {
            len.saturating_sub(1).saturating_sub(self.app.history_selected)
        }
    }

    fn render_hints(&self, frame: &mut Frame, area: Rect) {
        let hints = vec![
            ("Tab", "Switch View"),
            ("f", "Filter"),
            ("Up/Down", "Navigate"),
            ("q", "Quit"),
        ];

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

/// Format duration in seconds to human readable format
fn format_duration(secs: u32) -> String {
    let minutes = secs / 60;
    let seconds = secs % 60;
    format!("{}:{:02}", minutes, seconds)
}
