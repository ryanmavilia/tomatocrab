use std::fs;
use std::path::PathBuf;

use color_eyre::eyre::{Context, Result};
use directories::ProjectDirs;

use crate::session::Session;

/// Manages persistence of sessions to disk
pub struct Storage {
    data_path: PathBuf,
}

impl Storage {
    /// Create a new storage instance
    pub fn new() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("com", "tomatocrab", "tomatocrab")
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine data directory"))?;

        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir).wrap_err("Failed to create data directory")?;

        let data_path = data_dir.join("sessions.json");

        Ok(Self { data_path })
    }

    /// Load all sessions from disk
    pub fn load_sessions(&self) -> Result<Vec<Session>> {
        if !self.data_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.data_path).wrap_err("Failed to read sessions file")?;

        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        let sessions: Vec<Session> =
            serde_json::from_str(&content).wrap_err("Failed to parse sessions file")?;

        Ok(sessions)
    }

    /// Save a session to disk
    pub fn save_session(&self, session: Session) -> Result<()> {
        let mut sessions = self.load_sessions()?;
        sessions.push(session);

        let content = serde_json::to_string_pretty(&sessions).wrap_err("Failed to serialize sessions")?;

        fs::write(&self.data_path, content).wrap_err("Failed to write sessions file")?;

        Ok(())
    }

    /// Get the path where sessions are stored
    #[allow(dead_code)]
    pub fn data_path(&self) -> &PathBuf {
        &self.data_path
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new().expect("Failed to initialize storage")
    }
}
