use std::io::{stdout, Stdout};
use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use crate::action::Action;

/// Terminal wrapper that handles setup and cleanup
pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    /// Create and initialize a new terminal
    pub fn new() -> Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Ok(Self { terminal })
    }

    /// Enter the TUI mode
    pub fn enter(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    /// Exit the TUI mode and restore terminal
    pub fn exit(&mut self) -> Result<()> {
        self.terminal.show_cursor()?;
        execute!(stdout(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    /// Draw a frame
    pub fn draw<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Frame),
    {
        self.terminal.draw(f)?;
        Ok(())
    }

    /// Poll for events with a timeout
    pub fn poll_event(&self, timeout: Duration) -> Result<Option<Action>> {
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events
                if key.kind != KeyEventKind::Press {
                    return Ok(Some(Action::None));
                }

                let action = match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        Action::Quit
                    }
                    KeyCode::Enter => Action::Confirm,
                    KeyCode::Esc => Action::Cancel,
                    KeyCode::Backspace => Action::Backspace,
                    KeyCode::Tab => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            Action::PrevTab
                        } else {
                            Action::NextTab
                        }
                    }
                    KeyCode::Right => Action::NextTab,
                    KeyCode::Left => Action::PrevTab,
                    KeyCode::Up => Action::ScrollUp,
                    KeyCode::Down => Action::ScrollDown,
                    KeyCode::Char(c) => Action::Input(c), // All chars handled by app state
                    _ => Action::None,
                };

                return Ok(Some(action));
            }
        }
        Ok(None)
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.exit();
    }
}
