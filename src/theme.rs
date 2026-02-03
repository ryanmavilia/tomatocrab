//! Centralized color palette and style helpers for the TUI
//!
//! Tomato-inspired color scheme for a polished Pomodoro experience.

use ratatui::style::{Color, Modifier, Style};

/// Primary tomato red color
pub const PRIMARY: Color = Color::Rgb(231, 76, 60);

/// Accent gold color
pub const ACCENT: Color = Color::Rgb(243, 156, 18);

/// Success green color
pub const SUCCESS: Color = Color::Rgb(39, 174, 96);

/// Warning orange color
pub const WARNING: Color = Color::Rgb(230, 126, 34);

/// Background dark color
pub const BG_DARK: Color = Color::Rgb(30, 30, 30);

/// Surface color for cards/panels
pub const SURFACE: Color = Color::Rgb(45, 45, 45);

/// Muted text color
pub const TEXT_MUTED: Color = Color::Rgb(127, 140, 141);

/// Bright text color
pub const TEXT_BRIGHT: Color = Color::Rgb(236, 240, 241);

/// Border color
pub const BORDER: Color = Color::Rgb(52, 73, 94);

/// Highlight/selection color
pub const HIGHLIGHT: Color = Color::Rgb(52, 152, 219);

/// Timer running color
pub const TIMER_RUNNING: Color = SUCCESS;

/// Timer paused color
pub const TIMER_PAUSED: Color = ACCENT;

/// Timer finished color
pub const TIMER_FINISHED: Color = Color::Cyan;

/// Timer idle color
pub const TIMER_IDLE: Color = TEXT_MUTED;

/// Timer short break color (teal)
pub const TIMER_BREAK: Color = Color::Rgb(26, 188, 156);

/// Timer long break color (blue)
pub const TIMER_LONG_BREAK: Color = Color::Rgb(52, 152, 219);

/// Style helpers for consistent UI styling
pub struct Theme;

impl Theme {
    /// Title style (bold primary)
    pub fn title() -> Style {
        Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD)
    }

    /// Subtitle style (accent)
    pub fn subtitle() -> Style {
        Style::default().fg(ACCENT)
    }

    /// Muted text style
    pub fn muted() -> Style {
        Style::default().fg(TEXT_MUTED)
    }

    /// Bright text style
    pub fn bright() -> Style {
        Style::default().fg(TEXT_BRIGHT)
    }

    /// Success style (green)
    pub fn success() -> Style {
        Style::default().fg(SUCCESS)
    }

    /// Warning style (orange)
    pub fn warning() -> Style {
        Style::default().fg(WARNING)
    }

    /// Border style
    pub fn border() -> Style {
        Style::default().fg(BORDER)
    }

    /// Active border style
    pub fn border_active() -> Style {
        Style::default().fg(PRIMARY)
    }

    /// Highlight style for selections
    pub fn highlight() -> Style {
        Style::default().fg(HIGHLIGHT).add_modifier(Modifier::BOLD)
    }

    /// Key hint style (for keyboard shortcuts)
    pub fn key_hint() -> Style {
        Style::default().fg(HIGHLIGHT).add_modifier(Modifier::BOLD)
    }

    /// Key action style (description of what key does)
    pub fn key_action() -> Style {
        Style::default().fg(TEXT_MUTED)
    }

    /// Tab active style
    pub fn tab_active() -> Style {
        Style::default()
            .fg(PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    /// Tab inactive style
    pub fn tab_inactive() -> Style {
        Style::default().fg(TEXT_MUTED)
    }

    /// Stat card value style
    pub fn stat_value() -> Style {
        Style::default()
            .fg(TEXT_BRIGHT)
            .add_modifier(Modifier::BOLD)
    }

    /// Stat card label style
    pub fn stat_label() -> Style {
        Style::default().fg(TEXT_MUTED)
    }

    /// Progress gauge style
    pub fn progress_gauge() -> Style {
        Style::default().fg(PRIMARY).bg(SURFACE)
    }

    /// Table header style
    pub fn table_header() -> Style {
        Style::default()
            .fg(ACCENT)
            .add_modifier(Modifier::BOLD)
    }

    /// Table row normal style
    pub fn table_row() -> Style {
        Style::default().fg(TEXT_BRIGHT)
    }

    /// Table row selected style
    pub fn table_row_selected() -> Style {
        Style::default()
            .fg(TEXT_BRIGHT)
            .bg(Color::Rgb(60, 60, 60))
            .add_modifier(Modifier::BOLD)
    }

    /// Session completed status style
    pub fn status_completed() -> Style {
        Style::default().fg(SUCCESS)
    }

    /// Session interrupted status style
    pub fn status_interrupted() -> Style {
        Style::default().fg(WARNING)
    }

    /// Sparkline style
    pub fn sparkline() -> Style {
        Style::default().fg(SUCCESS)
    }

    /// Bar chart style
    pub fn bar_chart() -> Style {
        Style::default().fg(PRIMARY)
    }
}
