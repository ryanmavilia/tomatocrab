use std::time::Instant;

use chrono::{Datelike, Local, NaiveDate, Utc};
use color_eyre::eyre::Result;

use crate::action::Action;
use crate::components::session_list::SessionFilter;
use crate::session::Session;
use crate::storage::Storage;

/// What kind of timer is currently active
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimerMode {
    #[default]
    Work,
    ShortBreak,
    LongBreak,
}

/// The current state of the application
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    /// Waiting for user to start a session
    Idle,
    /// Entering task description
    EnteringTask,
    /// Timer is running
    Running,
    /// Timer is paused
    Paused,
    /// Work session completed, offer break option
    WorkFinished,
    /// Break completed, ready for new work
    BreakFinished,
}

/// The current view/tab being displayed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Timer,
    History,
    Stats,
}

impl View {
    /// Get view index for tab display
    pub fn index(&self) -> usize {
        match self {
            View::Timer => 0,
            View::History => 1,
            View::Stats => 2,
        }
    }

    /// Get next view (wrapping)
    pub fn next(&self) -> Self {
        match self {
            View::Timer => View::History,
            View::History => View::Stats,
            View::Stats => View::Timer,
        }
    }

    /// Get previous view (wrapping)
    pub fn prev(&self) -> Self {
        match self {
            View::Timer => View::Stats,
            View::History => View::Timer,
            View::Stats => View::History,
        }
    }
}

/// Main application state
pub struct App {
    /// Current state of the app
    pub state: AppState,
    /// Whether the app should quit
    pub should_quit: bool,
    /// Total duration of current timer in seconds
    pub total_duration_secs: u32,
    /// Remaining time in seconds
    pub remaining_secs: u32,
    /// Current task description
    pub task_description: String,
    /// When the current session started
    session_start: Option<Instant>,
    /// Time when paused (for calculating elapsed time)
    pause_start: Option<Instant>,
    /// Storage for persistence
    storage: Storage,
    /// When the pomodoro was started (for session record)
    pomodoro_started_at: Option<chrono::DateTime<Utc>>,
    /// Current view/tab
    pub current_view: View,
    /// Current session filter
    pub session_filter: SessionFilter,
    /// Cached sessions for history/stats views
    pub sessions_cache: Vec<Session>,
    /// Currently selected row in history view
    pub history_selected: usize,
    /// Current timer mode (work, short break, long break)
    pub timer_mode: TimerMode,
    /// Original work duration in seconds
    pub work_duration_secs: u32,
    /// Short break duration in seconds
    pub short_break_secs: u32,
    /// Long break duration in seconds
    pub long_break_secs: u32,
    /// Number of work sessions before a long break
    pub sessions_until_long_break: u32,
    /// Work sessions completed since last long break
    pub work_sessions_completed: u32,
}

impl App {
    /// Create a new application
    pub fn new(
        duration_minutes: u32,
        short_break_minutes: u32,
        long_break_minutes: u32,
        long_break_interval: u32,
    ) -> Result<Self> {
        let duration_secs = duration_minutes * 60;
        let storage = Storage::new()?;
        let sessions_cache = storage.load_sessions().unwrap_or_default();

        Ok(Self {
            state: AppState::Idle,
            should_quit: false,
            total_duration_secs: duration_secs,
            remaining_secs: duration_secs,
            task_description: String::new(),
            session_start: None,
            pause_start: None,
            storage,
            pomodoro_started_at: None,
            current_view: View::Timer,
            session_filter: SessionFilter::Week,
            sessions_cache,
            history_selected: 0,
            timer_mode: TimerMode::Work,
            work_duration_secs: duration_secs,
            short_break_secs: short_break_minutes * 60,
            long_break_secs: long_break_minutes * 60,
            sessions_until_long_break: long_break_interval,
            work_sessions_completed: 0,
        })
    }

    /// Handle an action and update state
    pub fn handle_action(&mut self, action: Action) -> Result<()> {
        match (&self.state, &action) {
            // Tab navigation (available in non-input states)
            (AppState::Idle | AppState::Running | AppState::Paused | AppState::WorkFinished | AppState::BreakFinished, Action::NextTab) => {
                self.next_view();
            }
            (AppState::Idle | AppState::Running | AppState::Paused | AppState::WorkFinished | AppState::BreakFinished, Action::PrevTab) => {
                self.prev_view();
            }

            // Scroll in history view
            (_, Action::ScrollUp) => {
                if self.current_view == View::History && self.history_selected > 0 {
                    self.history_selected -= 1;
                }
            }
            (_, Action::ScrollDown) => {
                if self.current_view == View::History {
                    let max = self.filtered_sessions().len().saturating_sub(1);
                    if self.history_selected < max {
                        self.history_selected += 1;
                    }
                }
            }

            // Idle state
            (AppState::Idle, Action::Confirm) => {
                if self.current_view == View::Timer {
                    self.state = AppState::EnteringTask;
                    self.task_description.clear();
                }
            }
            (AppState::Idle, Action::Input(c)) => {
                // Handle special keys
                match c {
                    'q' | 'Q' => {
                        self.should_quit = true;
                    }
                    'f' | 'F' => {
                        if matches!(self.current_view, View::History | View::Stats) {
                            self.cycle_filter();
                        } else if self.current_view == View::Timer {
                            // Start entering task with this character
                            self.state = AppState::EnteringTask;
                            self.task_description.clear();
                            self.task_description.push(*c);
                        }
                    }
                    _ => {
                        if self.current_view == View::Timer {
                            self.state = AppState::EnteringTask;
                            self.task_description.clear();
                            self.task_description.push(*c);
                        }
                    }
                }
            }

            // Entering task state
            (AppState::EnteringTask, Action::Input(c)) => {
                self.task_description.push(*c);
            }
            (AppState::EnteringTask, Action::Backspace) => {
                self.task_description.pop();
            }
            (AppState::EnteringTask, Action::Confirm) => {
                if !self.task_description.trim().is_empty() {
                    self.start_work_timer();
                }
            }
            (AppState::EnteringTask, Action::Cancel) => {
                self.task_description.clear();
                self.state = AppState::Idle;
            }

            // Running state
            (AppState::Running, Action::Input(c)) => {
                match c {
                    ' ' => {
                        // Only allow pause during work sessions
                        if self.timer_mode == TimerMode::Work {
                            self.pause_start = Some(Instant::now());
                            self.state = AppState::Paused;
                        }
                    }
                    's' | 'S' => {
                        // Skip - during break, skip remaining break time
                        if self.timer_mode != TimerMode::Work {
                            self.state = AppState::BreakFinished;
                        }
                    }
                    'r' | 'R' => {
                        // Only save if it's a work session
                        if self.timer_mode == TimerMode::Work {
                            self.save_current_session(false)?;
                        }
                        self.reset();
                    }
                    'q' | 'Q' => {
                        if self.timer_mode == TimerMode::Work {
                            self.save_current_session(false)?;
                        }
                        self.should_quit = true;
                    }
                    'f' | 'F' => {
                        if matches!(self.current_view, View::History | View::Stats) {
                            self.cycle_filter();
                        }
                    }
                    _ => {}
                }
            }
            (AppState::Running, Action::Tick) => {
                self.update_timer();
            }

            // Paused state (only for work sessions)
            (AppState::Paused, Action::Input(c)) => {
                match c {
                    ' ' => {
                        // Resume - adjust session_start to account for pause duration
                        if let (Some(pause_start), Some(session_start)) =
                            (self.pause_start, self.session_start)
                        {
                            let pause_duration = pause_start.elapsed();
                            self.session_start = Some(session_start + pause_duration);
                        }
                        self.pause_start = None;
                        self.state = AppState::Running;
                    }
                    'r' | 'R' => {
                        self.save_current_session(false)?;
                        self.reset();
                    }
                    'q' | 'Q' => {
                        self.save_current_session(false)?;
                        self.should_quit = true;
                    }
                    'f' | 'F' => {
                        if matches!(self.current_view, View::History | View::Stats) {
                            self.cycle_filter();
                        }
                    }
                    _ => {}
                }
            }

            // Work Finished state - offer break option
            (AppState::WorkFinished, Action::Input(c)) => {
                match c {
                    'b' | 'B' => {
                        // Start break (short or long based on completed sessions)
                        self.start_break();
                    }
                    's' | 'S' => {
                        // Skip break, go to idle
                        self.reset();
                    }
                    'q' | 'Q' => {
                        self.should_quit = true;
                    }
                    'f' | 'F' => {
                        if matches!(self.current_view, View::History | View::Stats) {
                            self.cycle_filter();
                        }
                    }
                    _ => {}
                }
            }
            (AppState::WorkFinished, Action::Confirm) => {
                // Skip break, start new task entry
                self.timer_mode = TimerMode::Work;
                self.total_duration_secs = self.work_duration_secs;
                self.remaining_secs = self.work_duration_secs;
                self.state = AppState::EnteringTask;
                self.task_description.clear();
            }

            // Break Finished state
            (AppState::BreakFinished, Action::Confirm) => {
                // Start new task entry
                self.timer_mode = TimerMode::Work;
                self.total_duration_secs = self.work_duration_secs;
                self.remaining_secs = self.work_duration_secs;
                self.state = AppState::EnteringTask;
                self.task_description.clear();
            }
            (AppState::BreakFinished, Action::Input(c)) => {
                match c {
                    's' | 'S' => {
                        // Go to idle
                        self.reset();
                    }
                    'q' | 'Q' => {
                        self.should_quit = true;
                    }
                    'f' | 'F' => {
                        if matches!(self.current_view, View::History | View::Stats) {
                            self.cycle_filter();
                        }
                    }
                    _ => {}
                }
            }

            _ => {}
        }

        Ok(())
    }

    /// Switch to next view
    pub fn next_view(&mut self) {
        self.current_view = self.current_view.next();
        if self.current_view != View::Timer {
            self.refresh_sessions();
        }
        self.history_selected = 0;
    }

    /// Switch to previous view
    pub fn prev_view(&mut self) {
        self.current_view = self.current_view.prev();
        if self.current_view != View::Timer {
            self.refresh_sessions();
        }
        self.history_selected = 0;
    }

    /// Cycle through session filters
    pub fn cycle_filter(&mut self) {
        self.session_filter = match self.session_filter {
            SessionFilter::Today => SessionFilter::Week,
            SessionFilter::Week => SessionFilter::All,
            SessionFilter::All => SessionFilter::Today,
        };
        self.history_selected = 0;
    }

    /// Refresh sessions cache from storage
    pub fn refresh_sessions(&mut self) {
        self.sessions_cache = self.storage.load_sessions().unwrap_or_default();
    }

    /// Get filtered sessions based on current filter
    pub fn filtered_sessions(&self) -> Vec<&Session> {
        let now = Local::now();
        let today = now.date_naive();

        self.sessions_cache
            .iter()
            .filter(|session| {
                let session_date = session.started_at.with_timezone(&Local).date_naive();

                match self.session_filter {
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

    /// Get daily focus time in seconds for the past 7 days
    /// Returns a vector of 7 values (oldest to newest)
    pub fn daily_focus_data(&self) -> Vec<u64> {
        let now = Local::now();
        let today = now.date_naive();

        (0..7)
            .rev()
            .map(|days_ago| {
                let target_date = today - chrono::Duration::days(days_ago);
                self.focus_time_for_date(target_date)
            })
            .collect()
    }

    /// Get focus time in seconds for a specific date
    fn focus_time_for_date(&self, date: NaiveDate) -> u64 {
        self.sessions_cache
            .iter()
            .filter(|session| {
                session.started_at.with_timezone(&Local).date_naive() == date
            })
            .map(|session| session.duration_secs as u64)
            .sum()
    }

    /// Get weekly focus data with day labels (for bar chart)
    /// Returns (day_label, focus_seconds) for the past 7 days
    pub fn weekly_bar_data(&self) -> Vec<(&'static str, u64)> {
        let now = Local::now();
        let today = now.date_naive();

        (0..7)
            .rev()
            .map(|days_ago| {
                let target_date = today - chrono::Duration::days(days_ago);
                let day_label = match target_date.weekday() {
                    chrono::Weekday::Mon => "Mon",
                    chrono::Weekday::Tue => "Tue",
                    chrono::Weekday::Wed => "Wed",
                    chrono::Weekday::Thu => "Thu",
                    chrono::Weekday::Fri => "Fri",
                    chrono::Weekday::Sat => "Sat",
                    chrono::Weekday::Sun => "Sun",
                };
                (day_label, self.focus_time_for_date(target_date))
            })
            .collect()
    }

    /// Start a work timer
    fn start_work_timer(&mut self) {
        self.timer_mode = TimerMode::Work;
        self.total_duration_secs = self.work_duration_secs;
        self.remaining_secs = self.work_duration_secs;
        self.session_start = Some(Instant::now());
        self.pomodoro_started_at = Some(Utc::now());
        self.state = AppState::Running;
    }

    /// Start a break timer (short or long based on completed sessions)
    fn start_break(&mut self) {
        // Determine if this should be a long break
        if self.work_sessions_completed >= self.sessions_until_long_break {
            self.timer_mode = TimerMode::LongBreak;
            self.total_duration_secs = self.long_break_secs;
            self.work_sessions_completed = 0; // Reset counter after long break
        } else {
            self.timer_mode = TimerMode::ShortBreak;
            self.total_duration_secs = self.short_break_secs;
        }
        self.remaining_secs = self.total_duration_secs;
        self.session_start = Some(Instant::now());
        self.state = AppState::Running;
    }

    /// Update the timer based on elapsed time
    fn update_timer(&mut self) {
        if let Some(start) = self.session_start {
            let elapsed = start.elapsed().as_secs() as u32;
            if elapsed >= self.total_duration_secs {
                self.remaining_secs = 0;

                if self.timer_mode == TimerMode::Work {
                    // Work session completed - save and offer break
                    self.state = AppState::WorkFinished;
                    self.work_sessions_completed += 1;
                    let _ = self.save_current_session(true);
                    self.refresh_sessions();
                } else {
                    // Break completed - NOT saved to history
                    self.state = AppState::BreakFinished;
                }
            } else {
                self.remaining_secs = self.total_duration_secs - elapsed;
            }
        }
    }

    /// Reset the app to idle state
    fn reset(&mut self) {
        self.state = AppState::Idle;
        self.timer_mode = TimerMode::Work;
        self.total_duration_secs = self.work_duration_secs;
        self.remaining_secs = self.work_duration_secs;
        self.task_description.clear();
        self.session_start = None;
        self.pause_start = None;
        self.pomodoro_started_at = None;
        self.refresh_sessions();
    }

    /// Save the current session
    fn save_current_session(&mut self, completed: bool) -> Result<()> {
        if let Some(started_at) = self.pomodoro_started_at {
            if !self.task_description.trim().is_empty() {
                let duration_secs = self.total_duration_secs - self.remaining_secs;
                let session = Session::new(
                    self.task_description.clone(),
                    started_at,
                    duration_secs,
                    completed,
                );
                self.storage.save_session(session)?;
            }
        }
        Ok(())
    }

    /// Get elapsed seconds
    pub fn elapsed_secs(&self) -> u32 {
        self.total_duration_secs - self.remaining_secs
    }

    /// Get progress as a ratio (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        self.elapsed_secs() as f64 / self.total_duration_secs as f64
    }

    /// Get storage reference
    #[allow(dead_code)]
    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    /// Get filter label
    pub fn filter_label(&self) -> &'static str {
        match self.session_filter {
            SessionFilter::Today => "Today",
            SessionFilter::Week => "This Week",
            SessionFilter::All => "All Time",
        }
    }
}
