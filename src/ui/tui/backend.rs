use std::io::Stderr;
use std::ops::Deref;
use std::ops::DerefMut;

use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use ratatui::crossterm;
use ratatui::prelude::*;

use crate::ui::tui::TuiError;

/// A wrapper around the terminal that enables raw mode on creation and disables it on drop.
pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stderr>>,
}

impl Tui {
    pub fn new() -> Result<Self, TuiError> {
        let terminal = Self::init()?;
        Ok(Self { terminal })
    }

    fn init() -> Result<Terminal<CrosstermBackend<Stderr>>, TuiError> {
        let buffer = std::io::stderr();
        let mut backend = CrosstermBackend::new(buffer);

        execute!(backend, EnterAlternateScreen)?;

        crossterm::terminal::enable_raw_mode()?;
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(terminal)
    }

    fn cleanup(&mut self) -> Result<(), TuiError> {
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;

        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }
}

impl Deref for Tui {
    type Target = Terminal<CrosstermBackend<Stderr>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.cleanup().unwrap();
    }
}
