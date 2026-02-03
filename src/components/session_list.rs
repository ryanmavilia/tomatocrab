use chrono::Local;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::session::Session;

/// Filter for session list
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionFilter {
    Today,
    Week,
    All,
}

/// Widget for displaying past sessions (TUI view, kept for future use)
#[allow(dead_code)]
pub struct SessionListWidget<'a> {
    sessions: &'a [Session],
    filter: SessionFilter,
}

#[allow(dead_code)]
impl<'a> SessionListWidget<'a> {
    pub fn new(sessions: &'a [Session], filter: SessionFilter) -> Self {
        Self { sessions, filter }
    }

    /// Filter sessions based on the current filter
    fn filtered_sessions(&self) -> Vec<&Session> {
        let now = Local::now();
        let today = now.date_naive();

        self.sessions
            .iter()
            .filter(|session| {
                let session_date = session.started_at.with_timezone(&Local).date_naive();

                match self.filter {
                    SessionFilter::Today => session_date == today,
                    SessionFilter::Week => {
                        let week_ago = today - chrono::Duration::days(7);
                        session_date >= week_ago
                    }
                    SessionFilter::All => true,
                }
            })
            .collect()
    }

    /// Render the session list
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let filtered = self.filtered_sessions();

        let header_cells = ["Date", "Time", "Task", "Duration", "Status"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let rows = filtered.iter().rev().map(|session| {
            let local_time = session.started_at.with_timezone(&Local);
            let date = local_time.format("%Y-%m-%d").to_string();
            let time = local_time.format("%H:%M").to_string();
            let duration = format_duration(session.duration_secs);
            let status = if session.completed {
                Cell::from("Completed").style(Style::default().fg(Color::Green))
            } else {
                Cell::from("Interrupted").style(Style::default().fg(Color::Yellow))
            };

            Row::new(vec![
                Cell::from(date),
                Cell::from(time),
                Cell::from(session.task.clone()),
                Cell::from(duration),
                status,
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(12),
                Constraint::Length(8),
                Constraint::Min(20),
                Constraint::Length(10),
                Constraint::Length(12),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Sessions ({}) ", self.filter_label())),
        );

        frame.render_widget(table, area);
    }

    fn filter_label(&self) -> &'static str {
        match self.filter {
            SessionFilter::Today => "Today",
            SessionFilter::Week => "This Week",
            SessionFilter::All => "All Time",
        }
    }
}

/// Format duration in seconds to human readable format
fn format_duration(secs: u32) -> String {
    let minutes = secs / 60;
    let seconds = secs % 60;
    format!("{}:{:02}", minutes, seconds)
}

/// Calculate and display statistics
pub struct SessionStats {
    pub total_sessions: usize,
    pub completed_sessions: usize,
    pub interrupted_sessions: usize,
    pub total_focus_time_secs: u32,
    pub average_duration_secs: u32,
}

impl SessionStats {
    pub fn from_sessions(sessions: &[Session]) -> Self {
        let total_sessions = sessions.len();
        let completed_sessions = sessions.iter().filter(|s| s.completed).count();
        let interrupted_sessions = total_sessions - completed_sessions;
        let total_focus_time_secs: u32 = sessions.iter().map(|s| s.duration_secs).sum();
        let average_duration_secs = if total_sessions > 0 {
            total_focus_time_secs / total_sessions as u32
        } else {
            0
        };

        Self {
            total_sessions,
            completed_sessions,
            interrupted_sessions,
            total_focus_time_secs,
            average_duration_secs,
        }
    }

    pub fn display(&self) {
        println!("Session Statistics");
        println!("==================");
        println!("Total Sessions:      {}", self.total_sessions);
        println!("Completed:           {} ({:.1}%)",
            self.completed_sessions,
            if self.total_sessions > 0 {
                self.completed_sessions as f64 / self.total_sessions as f64 * 100.0
            } else {
                0.0
            }
        );
        println!("Interrupted:         {}", self.interrupted_sessions);
        println!("Total Focus Time:    {}", format_duration_long(self.total_focus_time_secs));
        println!("Average Duration:    {}", format_duration_long(self.average_duration_secs));
    }
}

/// Format duration for CLI output
fn format_duration_long(secs: u32) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Display sessions in CLI format
pub fn display_sessions(sessions: &[Session], filter: SessionFilter) {
    let now = Local::now();
    let today = now.date_naive();

    let filtered: Vec<&Session> = sessions
        .iter()
        .filter(|session| {
            let session_date = session.started_at.with_timezone(&Local).date_naive();

            match filter {
                SessionFilter::Today => session_date == today,
                SessionFilter::Week => {
                    let week_ago = today - chrono::Duration::days(7);
                    session_date >= week_ago
                }
                SessionFilter::All => true,
            }
        })
        .collect();

    if filtered.is_empty() {
        println!("No sessions found.");
        return;
    }

    let filter_label = match filter {
        SessionFilter::Today => "Today",
        SessionFilter::Week => "This Week",
        SessionFilter::All => "All Time",
    };

    println!("Sessions ({})", filter_label);
    println!("{}", "=".repeat(60));
    println!("{:<12} {:<8} {:<24} {:<10} {}", "Date", "Time", "Task", "Duration", "Status");
    println!("{}", "-".repeat(60));

    for session in filtered.iter().rev() {
        let local_time = session.started_at.with_timezone(&Local);
        let date = local_time.format("%Y-%m-%d").to_string();
        let time = local_time.format("%H:%M").to_string();
        let duration = format_duration(session.duration_secs);
        let status = if session.completed { "Completed" } else { "Interrupted" };
        let task = if session.task.len() > 22 {
            format!("{}...", &session.task[..19])
        } else {
            session.task.clone()
        };

        println!("{:<12} {:<8} {:<24} {:<10} {}", date, time, task, duration, status);
    }
}
