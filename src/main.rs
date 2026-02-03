mod action;
mod app;
mod components;
mod session;
mod storage;
mod theme;
mod tui;

use std::time::Duration;

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use ratatui::layout::{Constraint, Layout};

use crate::action::Action;
use crate::app::{App, AppState, View};
use crate::components::session_list::{display_sessions, SessionFilter, SessionStats};
use crate::components::{HistoryWidget, StatsWidget, TabsWidget, TaskInputWidget, TimerWidget};
use crate::storage::Storage;
use crate::tui::Tui;

/// A Pomodoro timer TUI application
#[derive(Parser, Debug)]
#[command(name = "tomatocrab")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Duration of pomodoro in minutes
    #[arg(short, long, default_value = "25")]
    duration: u32,

    /// Short break duration in minutes
    #[arg(short = 's', long, default_value = "5")]
    short_break: u32,

    /// Long break duration in minutes
    #[arg(short = 'l', long, default_value = "15")]
    long_break: u32,

    /// Number of work sessions before a long break
    #[arg(short = 'n', long, default_value = "4")]
    long_break_interval: u32,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start a new pomodoro timer (default)
    Start {
        /// Duration in minutes
        #[arg(short, long, default_value = "25")]
        duration: u32,

        /// Short break duration in minutes
        #[arg(short = 's', long, default_value = "5")]
        short_break: u32,

        /// Long break duration in minutes
        #[arg(short = 'l', long, default_value = "15")]
        long_break: u32,

        /// Number of work sessions before a long break
        #[arg(short = 'n', long, default_value = "4")]
        long_break_interval: u32,
    },
    /// List past sessions
    List {
        /// Show only today's sessions
        #[arg(long, conflicts_with = "all")]
        today: bool,
        /// Show this week's sessions
        #[arg(long, conflicts_with = "all")]
        week: bool,
        /// Show all sessions
        #[arg(long)]
        all: bool,
    },
    /// Show focus time statistics
    Stats {
        /// Show only today's stats
        #[arg(long, conflicts_with = "all")]
        today: bool,
        /// Show this week's stats
        #[arg(long, conflicts_with = "all")]
        week: bool,
        /// Show all-time stats
        #[arg(long)]
        all: bool,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Start {
            duration,
            short_break,
            long_break,
            long_break_interval,
        }) => run_timer(duration, short_break, long_break, long_break_interval),
        Some(Commands::List { today, week, all }) => {
            let filter = if today {
                SessionFilter::Today
            } else if all {
                SessionFilter::All
            } else if week {
                SessionFilter::Week
            } else {
                // Default to week if no flag specified
                SessionFilter::Week
            };
            list_sessions(filter)
        }
        Some(Commands::Stats { today, week, all }) => {
            let filter = if today {
                SessionFilter::Today
            } else if all {
                SessionFilter::All
            } else if week {
                SessionFilter::Week
            } else {
                SessionFilter::Week
            };
            show_stats(filter)
        }
        None => run_timer(
            cli.duration,
            cli.short_break,
            cli.long_break,
            cli.long_break_interval,
        ),
    }
}

/// Run the timer TUI
fn run_timer(
    duration_minutes: u32,
    short_break_minutes: u32,
    long_break_minutes: u32,
    long_break_interval: u32,
) -> Result<()> {
    let mut tui = Tui::new()?;
    tui.enter()?;

    let mut app = App::new(
        duration_minutes,
        short_break_minutes,
        long_break_minutes,
        long_break_interval,
    )?;

    // Main event loop
    let tick_rate = Duration::from_millis(250);

    while !app.should_quit {
        // Draw the UI
        tui.draw(|frame| {
            let area = frame.area();

            // When entering task, show full-screen task input
            if app.state == AppState::EnteringTask {
                let widget = TaskInputWidget::new(&app);
                widget.render(frame, area);
                return;
            }

            // Otherwise show tabbed interface
            let main_chunks = Layout::vertical([
                Constraint::Length(2),  // Tabs bar
                Constraint::Min(10),    // Content area
            ])
            .split(area);

            // Render tabs bar
            let tabs = TabsWidget::new(app.current_view);
            tabs.render(frame, main_chunks[0]);

            // Render content based on current view
            match app.current_view {
                View::Timer => {
                    let widget = TimerWidget::new(&app);
                    widget.render(frame, main_chunks[1]);
                }
                View::History => {
                    let widget = HistoryWidget::new(&app);
                    widget.render(frame, main_chunks[1]);
                }
                View::Stats => {
                    let widget = StatsWidget::new(&app);
                    widget.render(frame, main_chunks[1]);
                }
            }
        })?;

        // Handle events
        if let Some(action) = tui.poll_event(tick_rate)? {
            app.handle_action(action)?;
        }

        // Send tick action if timer is running
        if app.state == AppState::Running {
            app.handle_action(Action::Tick)?;
        }
    }

    tui.exit()?;
    Ok(())
}

/// List past sessions
fn list_sessions(filter: SessionFilter) -> Result<()> {
    let storage = Storage::new()?;
    let sessions = storage.load_sessions()?;
    display_sessions(&sessions, filter);
    Ok(())
}

/// Show statistics
fn show_stats(filter: SessionFilter) -> Result<()> {
    let storage = Storage::new()?;
    let sessions = storage.load_sessions()?;

    let now = chrono::Local::now();
    let today = now.date_naive();

    let filtered: Vec<_> = sessions
        .iter()
        .filter(|session| {
            let session_date = session
                .started_at
                .with_timezone(&chrono::Local)
                .date_naive();

            match filter {
                SessionFilter::Today => session_date == today,
                SessionFilter::Week => {
                    let week_ago = today - chrono::Duration::days(7);
                    session_date >= week_ago
                }
                SessionFilter::All => true,
            }
        })
        .cloned()
        .collect();

    let filter_label = match filter {
        SessionFilter::Today => "Today",
        SessionFilter::Week => "This Week",
        SessionFilter::All => "All Time",
    };

    println!("Statistics ({})", filter_label);
    println!();

    let stats = SessionStats::from_sessions(&filtered);
    stats.display();

    Ok(())
}
