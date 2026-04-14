use std::time::Duration;
use std::time::Instant;

use PomodoroState::*;

#[derive(Clone, Debug)]
pub struct Pomodoro {
    focus: Duration,
    long_break: Duration,
    short_break: Duration,

    long_interval: u32,

    running: bool,
    state: PomodoroState,

    focus_sessions: u32,

    started_at: Option<Instant>,
    remaining_time: Option<Duration>,
}

impl Pomodoro {
    pub fn new(
        focus: Duration,
        long_break: Duration,
        short_break: Duration,
        long_interval: u32,
    ) -> Self {
        Self {
            focus,
            long_break,
            short_break,
            long_interval,
            ..Default::default()
        }
    }

    pub fn start(&mut self) -> Result<(), PomodoroError> {
        self.check_not_running()?;
        self.running = true;
        self.reset_time();
        Ok(())
    }

    pub fn tick(&mut self) -> Result<(), PomodoroError> {
        if let (Some(started_at), Some(remaining)) = (self.started_at, self.remaining_time) {
            let elapsed = started_at.elapsed();
            let remaining = remaining.saturating_sub(elapsed);
            self.remaining_time = Some(remaining);
        }
        Ok(())
    }

    /// Adds given duration to session's remaining time.
    ///
    /// # Errors
    ///
    /// Returns [`PomodoroError::NotRunning`] if the session is not running.
    pub fn add(&mut self, duration: Duration) -> Result<(), PomodoroError> {
        self.check_running()?;
        if let Some(remaining) = self.remaining_time.as_mut() {
            *remaining += duration;
        } else {
            return Err(PomodoroError::UnexpectedState(
                "Remaining is None when pomodoro is running".to_string(),
            ));
        }
        Ok(())
    }

    /// Subtracts given duration from session's remaining time.
    ///
    /// # Errors
    ///
    /// Returns [`PomodoroError::Running`] if session is running.
    pub fn subtract(&mut self, duration: Duration) -> Result<(), PomodoroError> {
        self.check_running()?;
        if let Some(remaining) = self.remaining_time.as_mut() {
            *remaining -= duration;
        } else {
            return Err(PomodoroError::UnexpectedState(
                "Remaining is None when pomodoro is running".to_string(),
            ));
        }
        Ok(())
    }

    /// Ticks timer state and disables running state.
    ///
    /// # Errors
    ///
    /// Returns [`PomodoroError::NotRunning`] if the session is not running.
    pub fn pause(&mut self) -> Result<(), PomodoroError> {
        self.check_running()?;

        self.running = false;
        self.tick()?;
        self.started_at = None;

        Ok(())
    }

    /// Resumes timer state.
    ///
    /// # Errors
    ///
    /// Returns [`PomodoroError::Running`] if session is running.
    pub fn resume(&mut self) -> Result<(), PomodoroError> {
        self.check_not_running()?;

        self.running = true;
        self.started_at = Some(Instant::now());

        Ok(())
    }

    /// Toggles paused state.
    pub fn toggle_pause(&mut self) {
        if self.running {
            self.pause().unwrap()
        } else {
            self.resume().unwrap()
        }
    }

    /// Skips to the next session.
    ///
    /// # Errors
    ///
    /// Returns [`PomodoroError::NotRunning`] if the session is not running.
    pub fn skip(&mut self) -> Result<(), PomodoroError> {
        self.check_running()?;
        self.next_state();
        self.reset_time();
        Ok(())
    }

    /// Resets timer of current running session.
    ///
    /// # Errors
    ///
    /// Returns [`PomodoroError::NotRunning`] if the session is not running.
    pub fn reset(&mut self) -> Result<(), PomodoroError> {
        self.check_running()?;
        self.reset_time();
        Ok(())
    }

    pub fn state(&self) -> PomodoroState {
        self.state.clone()
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn focus_sessions(&self) -> u32 {
        self.focus_sessions
    }

    pub fn started_at(&self) -> Option<Instant> {
        self.started_at.clone()
    }

    pub fn remaining_time(&self) -> Option<Duration> {
        self.remaining_time.clone()
    }

    /// Gets session duration based on current state.
    pub fn session_duration(&self) -> Duration {
        let duration = match self.state {
            Focus => self.focus,
            LongBreak => self.long_break,
            ShortBreak => self.short_break,
        };
        duration
    }

    fn reset_time(&mut self) {
        self.started_at = Some(Instant::now());
        self.remaining_time = Some(self.session_duration());
    }

    /// Returns [`PomodoroError::NotRunning`] if session is not running yet.
    fn check_running(&self) -> Result<(), PomodoroError> {
        if !self.running {
            return Err(PomodoroError::NotRunning);
        }
        Ok(())
    }

    /// Returns [`PomodoroError::Running`] if session is running.
    fn check_not_running(&self) -> Result<(), PomodoroError> {
        if self.running {
            return Err(PomodoroError::Running);
        }
        Ok(())
    }

    /// Sets session state based on previous state.
    fn next_state(&mut self) {
        match self.state {
            Focus => {
                self.focus_sessions += 1;
                if self.focus_sessions % self.long_interval == 0 {
                    self.state = LongBreak;
                } else {
                    self.state = ShortBreak;
                }
            }
            _ => self.state = Focus,
        }
    }
}

impl Default for Pomodoro {
    fn default() -> Self {
        Self {
            state: Default::default(),
            focus: Duration::from_mins(25),
            long_break: Duration::from_mins(15),
            short_break: Duration::from_mins(5),
            long_interval: 3,
            focus_sessions: 0,
            running: false,
            remaining_time: None,
            started_at: None,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum PomodoroState {
    Focus,
    LongBreak,
    ShortBreak,
}

impl Default for PomodoroState {
    fn default() -> Self {
        Self::Focus
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PomodoroError {
    #[error("Session is running.")]
    Running,

    #[error("Session is not running")]
    NotRunning,

    #[error("Unexpected state {0}")]
    UnexpectedState(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_state_focus() {
        let mut pomo = Pomodoro::default();
        pomo.state = ShortBreak;

        pomo.next_state();
        assert_eq!(pomo.state, Focus)
    }

    #[test]
    fn test_next_state_short_break() {
        let mut pomo = Pomodoro::default();

        pomo.next_state();
        assert_eq!(pomo.state, ShortBreak)
    }

    #[test]
    fn test_next_state_long_break() {
        let mut pomo = Pomodoro::default();

        // Short break
        pomo.next_state();
        // Focus
        pomo.next_state();
        // Short break
        pomo.next_state();
        // Focus
        pomo.next_state();

        // Long Break
        pomo.next_state();
        assert_eq!(pomo.state, LongBreak)
    }

    #[test]
    fn test_running_checks() {
        let mut pomo = Pomodoro::default();

        pomo.running = false;
        assert_eq!(pomo.check_running(), Err(PomodoroError::NotRunning));

        pomo.running = true;
        assert!(matches!(
            pomo.check_not_running(),
            Err(PomodoroError::Running)
        ));
    }

    #[test]
    fn test_skip() {
        let mut pomo = Pomodoro::default();
        pomo.focus_sessions = 1;
        pomo.state = Focus;
        pomo.running = true;
        pomo.started_at = None;
        pomo.remaining_time = None;

        pomo.skip().unwrap();

        assert_eq!(pomo.focus_sessions, 2);
        assert_ne!(pomo.started_at, None);
        assert_ne!(pomo.remaining_time, None);
    }

    #[test]
    fn test_pause_resume() {
        let mut pomo = Pomodoro::default();

        pomo.start().unwrap();

        pomo.pause().unwrap();
        assert!(!pomo.running);
        assert_eq!(pomo.started_at, None);

        pomo.resume().unwrap();
        assert!(pomo.running);
        assert_ne!(pomo.started_at, None);
    }

    #[test]
    fn test_add() {
        let mut pomo = Pomodoro::default();
        pomo.remaining_time = Some(Duration::from_secs(67));
        pomo.running = true;

        pomo.add(Duration::from_secs(2)).unwrap();
        assert_eq!(pomo.remaining_time, Some(Duration::from_secs(69)));

        pomo.subtract(Duration::from_secs(2)).unwrap();
        assert_eq!(pomo.remaining_time, Some(Duration::from_secs(67)));
    }
}
