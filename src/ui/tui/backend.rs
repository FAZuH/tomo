use std::io::Stderr;
use std::ops::Deref;
use std::ops::DerefMut;

use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
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
        Self::set_panic_hook();
        Ok(Self { terminal })
    }

    fn init() -> Result<Terminal<CrosstermBackend<Stderr>>, TuiError> {
        color_eyre::install().map_err(|e| TuiError::InitializeError(e.to_string()))?;
        let buffer = std::io::stderr();
        let mut backend = CrosstermBackend::new(buffer);

        execute!(backend, EnterAlternateScreen)?;
        execute!(backend, EnableMouseCapture)?;

        crossterm::terminal::enable_raw_mode()?;
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(terminal)
    }

    fn cleanup() {
        let _ = execute!(std::io::stderr(), LeaveAlternateScreen, DisableMouseCapture,);
        let _ = crossterm::terminal::disable_raw_mode();
    }

    fn set_panic_hook() {
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            Self::cleanup();
            hook(info);
        }));
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
        Self::cleanup();
    }
}
