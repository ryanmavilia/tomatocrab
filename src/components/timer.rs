//! Timer display widget with BigText countdown

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::app::{App, AppState, TimerMode};
use crate::theme::{
    Theme, BORDER, PRIMARY, SUCCESS, TIMER_BREAK, TIMER_FINISHED, TIMER_IDLE, TIMER_LONG_BREAK,
    TIMER_PAUSED, TIMER_RUNNING,
};

/// Widget for displaying the timer
pub struct TimerWidget<'a> {
    app: &'a App,
}

impl<'a> TimerWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }

    /// Render the timer widget
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
            Constraint::Length(2),  // Task description
            Constraint::Length(1),  // Spacer
            Constraint::Length(7),  // Big timer display
            Constraint::Length(1),  // Spacer
            Constraint::Length(3),  // Progress bar with labels
            Constraint::Length(2),  // Status
            Constraint::Min(0),     // Flexible spacer
            Constraint::Length(2),  // Keyboard hints
        ])
        .split(inner_area);

        self.render_task(frame, chunks[0]);
        self.render_big_timer(frame, chunks[2]);
        self.render_progress(frame, chunks[4]);
        self.render_status(frame, chunks[5]);
        self.render_hints(frame, chunks[7]);
    }

    fn render_task(&self, frame: &mut Frame, area: Rect) {
        let task_text = match (&self.app.state, &self.app.timer_mode) {
            (AppState::Idle, _) | (AppState::EnteringTask, _) => {
                if self.app.task_description.is_empty() {
                    "Press ENTER to start a new session".to_string()
                } else {
                    format!("Working on: {}", self.app.task_description)
                }
            }
            (_, TimerMode::ShortBreak) => "Take a short break - stretch, hydrate!".to_string(),
            (_, TimerMode::LongBreak) => "Long break - you've earned it! Rest well.".to_string(),
            (AppState::WorkFinished, _) => {
                format!("Completed: {}", self.app.task_description)
            }
            (AppState::BreakFinished, _) => "Break complete - ready for another session?".to_string(),
            _ => format!("Working on: {}", self.app.task_description),
        };

        let task = Paragraph::new(task_text)
            .style(Theme::subtitle())
            .alignment(Alignment::Center);
        frame.render_widget(task, area);
    }

    fn render_big_timer(&self, frame: &mut Frame, area: Rect) {
        let minutes = self.app.remaining_secs / 60;
        let seconds = self.app.remaining_secs % 60;

        let color = match (&self.app.state, &self.app.timer_mode) {
            (AppState::Running, TimerMode::ShortBreak) => TIMER_BREAK,
            (AppState::Running, TimerMode::LongBreak) => TIMER_LONG_BREAK,
            (AppState::Running, TimerMode::Work) => TIMER_RUNNING,
            (AppState::Paused, _) => TIMER_PAUSED,
            (AppState::WorkFinished, _) => TIMER_FINISHED,
            (AppState::BreakFinished, _) => TIMER_BREAK,
            _ => TIMER_IDLE,
        };

        // Create big ASCII art digits
        let big_text = create_big_time(minutes, seconds);

        let timer = Paragraph::new(big_text)
            .style(ratatui::style::Style::default().fg(color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(timer, area);
    }

    fn render_progress(&self, frame: &mut Frame, area: Rect) {
        let progress = self.app.progress();
        let elapsed = self.app.elapsed_secs();
        let remaining = self.app.remaining_secs;

        // Format times
        let elapsed_str = format!("{:02}:{:02}", elapsed / 60, elapsed % 60);
        let remaining_str = format!("-{:02}:{:02}", remaining / 60, remaining % 60);
        let percent_label = format!("{:.0}%", progress * 100.0);

        // Create layout with labels on sides
        let progress_chunks = Layout::horizontal([
            Constraint::Length(8),  // Elapsed label
            Constraint::Min(10),    // Progress bar
            Constraint::Length(8),  // Remaining label
        ])
        .split(area);

        // Elapsed time label
        let elapsed_widget = Paragraph::new(elapsed_str)
            .style(Theme::muted())
            .alignment(Alignment::Right);
        frame.render_widget(elapsed_widget, progress_chunks[0]);

        // Progress bar
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).border_style(Theme::border()))
            .gauge_style(Theme::progress_gauge())
            .ratio(progress)
            .label(percent_label);
        frame.render_widget(gauge, progress_chunks[1]);

        // Remaining time label
        let remaining_widget = Paragraph::new(remaining_str)
            .style(Theme::muted())
            .alignment(Alignment::Left);
        frame.render_widget(remaining_widget, progress_chunks[2]);
    }

    fn render_status(&self, frame: &mut Frame, area: Rect) {
        let (status_text, style) = match (&self.app.state, &self.app.timer_mode) {
            (AppState::Idle, _) => ("READY", Theme::muted()),
            (AppState::EnteringTask, _) => ("ENTER TASK", Theme::subtitle()),
            (AppState::Running, TimerMode::Work) => (
                "FOCUS TIME",
                ratatui::style::Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD),
            ),
            (AppState::Running, TimerMode::ShortBreak) => (
                "SHORT BREAK",
                ratatui::style::Style::default().fg(TIMER_BREAK).add_modifier(Modifier::BOLD),
            ),
            (AppState::Running, TimerMode::LongBreak) => (
                "LONG BREAK",
                ratatui::style::Style::default().fg(TIMER_LONG_BREAK).add_modifier(Modifier::BOLD),
            ),
            (AppState::Paused, _) => (
                "PAUSED",
                ratatui::style::Style::default().fg(TIMER_PAUSED).add_modifier(Modifier::BOLD),
            ),
            (AppState::WorkFinished, _) => (
                "SESSION COMPLETE!",
                ratatui::style::Style::default().fg(TIMER_FINISHED).add_modifier(Modifier::BOLD),
            ),
            (AppState::BreakFinished, _) => (
                "BREAK OVER",
                ratatui::style::Style::default().fg(TIMER_BREAK).add_modifier(Modifier::BOLD),
            ),
        };

        let status = Paragraph::new(status_text)
            .style(style)
            .alignment(Alignment::Center);
        frame.render_widget(status, area);
    }

    fn render_hints(&self, frame: &mut Frame, area: Rect) {
        let hints = match (&self.app.state, &self.app.timer_mode) {
            (AppState::Idle, _) => vec![
                ("Enter", "Start"),
                ("Tab", "View"),
                ("q", "Quit"),
            ],
            (AppState::EnteringTask, _) => vec![
                ("Enter", "Confirm"),
                ("Esc", "Cancel"),
            ],
            (AppState::Running, TimerMode::Work) => vec![
                ("Space", "Pause"),
                ("r", "Stop"),
                ("Tab", "View"),
                ("q", "Quit"),
            ],
            (AppState::Running, TimerMode::ShortBreak | TimerMode::LongBreak) => vec![
                ("s", "Skip"),
                ("r", "Stop"),
                ("Tab", "View"),
                ("q", "Quit"),
            ],
            (AppState::Paused, _) => vec![
                ("Space", "Resume"),
                ("r", "Stop"),
                ("Tab", "View"),
                ("q", "Quit"),
            ],
            (AppState::WorkFinished, _) => vec![
                ("b", "Break"),
                ("Enter", "New Task"),
                ("s", "Skip"),
                ("q", "Quit"),
            ],
            (AppState::BreakFinished, _) => vec![
                ("Enter", "New Task"),
                ("s", "Idle"),
                ("q", "Quit"),
            ],
        };

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

/// Create big ASCII art time display
fn create_big_time(minutes: u32, seconds: u32) -> String {
    let time_str = format!("{:02}:{:02}", minutes, seconds);

    // 7-segment style ASCII digits
    let digits: Vec<[&str; 5]> = vec![
        // 0
        [
            " ███ ",
            "█   █",
            "█   █",
            "█   █",
            " ███ ",
        ],
        // 1
        [
            "  █  ",
            " ██  ",
            "  █  ",
            "  █  ",
            " ███ ",
        ],
        // 2
        [
            " ███ ",
            "    █",
            " ███ ",
            "█    ",
            "█████",
        ],
        // 3
        [
            "█████",
            "    █",
            " ███ ",
            "    █",
            "█████",
        ],
        // 4
        [
            "█   █",
            "█   █",
            "█████",
            "    █",
            "    █",
        ],
        // 5
        [
            "█████",
            "█    ",
            "█████",
            "    █",
            "█████",
        ],
        // 6
        [
            " ███ ",
            "█    ",
            "█████",
            "█   █",
            " ███ ",
        ],
        // 7
        [
            "█████",
            "    █",
            "   █ ",
            "  █  ",
            "  █  ",
        ],
        // 8
        [
            " ███ ",
            "█   █",
            " ███ ",
            "█   █",
            " ███ ",
        ],
        // 9
        [
            " ███ ",
            "█   █",
            "█████",
            "    █",
            " ███ ",
        ],
    ];

    let colon: [&str; 5] = [
        " ",
        "█",
        " ",
        "█",
        " ",
    ];

    let mut lines: Vec<String> = vec![String::new(); 5];

    for (i, c) in time_str.chars().enumerate() {
        if c == ':' {
            for (line_idx, line) in lines.iter_mut().enumerate() {
                line.push_str(colon[line_idx]);
                line.push(' ');
            }
        } else if let Some(digit) = c.to_digit(10) {
            let digit_art = &digits[digit as usize];
            for (line_idx, line) in lines.iter_mut().enumerate() {
                line.push_str(digit_art[line_idx]);
                if i < time_str.len() - 1 {
                    line.push(' ');
                }
            }
        }
    }

    lines.join("\n")
}
