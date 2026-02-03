//! Statistics dashboard with stat cards, sparklines, and bar charts

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Paragraph, Sparkline},
    Frame,
};

use crate::app::App;
use crate::components::session_list::SessionStats;
use crate::theme::{Theme, ACCENT, BORDER, PRIMARY, SUCCESS, TEXT_BRIGHT};

/// Widget for displaying statistics dashboard
pub struct StatsWidget<'a> {
    app: &'a App,
}

impl<'a> StatsWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }

    /// Render the stats widget
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::vertical([
            Constraint::Length(5),  // Stat cards row
            Constraint::Length(6),  // Sparkline
            Constraint::Min(8),     // Bar chart
            Constraint::Length(2),  // Hints
        ])
        .split(area);

        self.render_stat_cards(frame, chunks[0]);
        self.render_sparkline(frame, chunks[1]);
        self.render_bar_chart(frame, chunks[2]);
        self.render_hints(frame, chunks[3]);
    }

    fn render_stat_cards(&self, frame: &mut Frame, area: Rect) {
        let filtered: Vec<_> = self.app.filtered_sessions().into_iter().cloned().collect();
        let stats = SessionStats::from_sessions(&filtered);

        // Create 4 equal-width columns
        let card_chunks = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

        // Sessions card
        self.render_stat_card(
            frame,
            card_chunks[0],
            &stats.total_sessions.to_string(),
            "Sessions",
        );

        // Completion rate card
        let completion_pct = if stats.total_sessions > 0 {
            format!(
                "{:.0}%",
                stats.completed_sessions as f64 / stats.total_sessions as f64 * 100.0
            )
        } else {
            "0%".to_string()
        };
        self.render_stat_card(frame, card_chunks[1], &completion_pct, "Complete");

        // Focus time card
        let focus_time = format_duration_short(stats.total_focus_time_secs);
        self.render_stat_card(frame, card_chunks[2], &focus_time, "Focus");

        // Average duration card
        let avg_duration = format_duration_short(stats.average_duration_secs);
        self.render_stat_card(frame, card_chunks[3], &avg_duration, "Average");
    }

    fn render_stat_card(&self, frame: &mut Frame, area: Rect, value: &str, label: &str) {
        let inner_chunks = Layout::vertical([
            Constraint::Length(2), // Value
            Constraint::Length(1), // Label
        ])
        .margin(1)
        .split(area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(ratatui::style::Style::default().fg(BORDER));
        frame.render_widget(block, area);

        let value_widget = Paragraph::new(value)
            .style(Theme::stat_value())
            .alignment(Alignment::Center);
        frame.render_widget(value_widget, inner_chunks[0]);

        let label_widget = Paragraph::new(label)
            .style(Theme::stat_label())
            .alignment(Alignment::Center);
        frame.render_widget(label_widget, inner_chunks[1]);
    }

    fn render_sparkline(&self, frame: &mut Frame, area: Rect) {
        let daily_data = self.app.daily_focus_data();

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(ratatui::style::Style::default().fg(BORDER))
                    .title(" 7-Day Trend ")
                    .title_style(
                        ratatui::style::Style::default()
                            .fg(ACCENT)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .data(&daily_data)
            .style(Theme::sparkline());

        frame.render_widget(sparkline, area);
    }

    fn render_bar_chart(&self, frame: &mut Frame, area: Rect) {
        let weekly_data = self.app.weekly_bar_data();

        // Convert to minutes for better display
        let bars: Vec<Bar> = weekly_data
            .iter()
            .map(|(label, secs)| {
                let minutes = *secs / 60;
                Bar::default()
                    .value(minutes)
                    .label(Line::from(*label))
                    .style(ratatui::style::Style::default().fg(PRIMARY))
                    .value_style(
                        ratatui::style::Style::default()
                            .fg(TEXT_BRIGHT)
                            .add_modifier(Modifier::BOLD),
                    )
            })
            .collect();

        let bar_chart = BarChart::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(ratatui::style::Style::default().fg(BORDER))
                    .title(" Weekly Activity (minutes) ")
                    .title_style(
                        ratatui::style::Style::default()
                            .fg(ACCENT)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .data(BarGroup::default().bars(&bars))
            .bar_width(5)
            .bar_gap(2)
            .bar_style(ratatui::style::Style::default().fg(PRIMARY))
            .value_style(
                ratatui::style::Style::default()
                    .fg(TEXT_BRIGHT)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(bar_chart, area);
    }

    fn render_hints(&self, frame: &mut Frame, area: Rect) {
        let hints = vec![
            ("Tab", "Switch View"),
            ("f", "Filter"),
            ("q", "Quit"),
        ];

        let filter_info = Span::styled(
            format!("Filter: {} ", self.app.filter_label()),
            ratatui::style::Style::default().fg(SUCCESS),
        );

        let hint_spans: Vec<Span> = std::iter::once(filter_info)
            .chain(std::iter::once(Span::raw(" | ")))
            .chain(hints.iter().enumerate().flat_map(|(i, (key, action))| {
                let mut spans = vec![
                    Span::styled(format!("[{}]", key), Theme::key_hint()),
                    Span::raw(" "),
                    Span::styled(*action, Theme::key_action()),
                ];
                if i < hints.len() - 1 {
                    spans.push(Span::raw("  "));
                }
                spans
            }))
            .collect();

        let hints_paragraph = Paragraph::new(Line::from(hint_spans)).alignment(Alignment::Center);
        frame.render_widget(hints_paragraph, area);
    }
}

/// Format duration to short format (e.g., "2h 15m" or "45m")
fn format_duration_short(secs: u32) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;

    if hours > 0 {
        format!("{}h{}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        "0m".to_string()
    }
}
