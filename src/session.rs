use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a completed or interrupted Pomodoro session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique identifier for the session
    pub id: Uuid,
    /// Description of the task worked on
    pub task: String,
    /// When the session started
    pub started_at: DateTime<Utc>,
    /// Duration in seconds that was actually worked
    pub duration_secs: u32,
    /// Whether the session ran its full intended duration
    pub completed: bool,
}

impl Session {
    /// Create a new session
    pub fn new(task: String, started_at: DateTime<Utc>, duration_secs: u32, completed: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            task,
            started_at,
            duration_secs,
            completed,
        }
    }
}
