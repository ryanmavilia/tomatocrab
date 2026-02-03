/// Actions that can be performed in the application
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Action {
    /// Start a new pomodoro session
    Start,
    /// Pause the current timer
    Pause,
    /// Resume a paused timer
    Resume,
    /// Stop/cancel the current session
    Stop,
    /// Quit the application
    Quit,
    /// Timer tick (internal action)
    Tick,
    /// Character input for task description
    Input(char),
    /// Delete last character
    Backspace,
    /// Confirm input
    Confirm,
    /// Cancel input
    Cancel,
    /// Switch to next tab
    NextTab,
    /// Switch to previous tab
    PrevTab,
    /// Cycle through session filters
    CycleFilter,
    /// Scroll up in list views
    ScrollUp,
    /// Scroll down in list views
    ScrollDown,
    /// No action
    None,
}
