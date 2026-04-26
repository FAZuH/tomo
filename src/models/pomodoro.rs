use std::time::Duration;
use std::time::Instant;

use PomodoroState::*;

#[derive(Clone, Debug)]
pub struct Pomodoro {
    // Session config
    focus: Duration,
    long_break: Duration,
    short_break: Duration,

    long_interval: u32,

    // Aggregate session stats
    total_sessions: u32,
    focus_sessions: u32,
    /// Accumulated running time from previous segments (before current anchor).
    accumulated: Duration,

    // Session data
    running: bool,
    state: PomodoroState,

    /// Anchor instant for the current running segment.
    /// Always set when running, None when paused.
    anchor: Option<Instant>,
    /// Remaining time frozen at pause, or session duration at start/reset.
    /// When running, actual remaining is `frozen_remaining - anchor.elapsed()`.
    frozen_remaining: Duration,
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

    /// Starts the Pomodoro session timer.
    ///
    /// # Errors
    ///
    /// Returns [`PomodoroError::Running`] if the session is running.
    pub fn start(&mut self) -> Result<(), PomodoroError> {
        self.check_not_running()?;
        self.running = true;
        self.reset_time();
        Ok(())
    }

    /// Adds given duration to session's remaining time.
    pub fn add(&mut self, duration: Duration) {
        self.frozen_remaining += duration;
    }

    /// Subtracts given duration from session's remaining time.
    pub fn subtract(&mut self, duration: Duration) {
        self.frozen_remaining = self.frozen_remaining.saturating_sub(duration);
    }

    /// Freezes remaining time and disables running state.
    ///
    /// # Errors
    ///
    /// Returns [`PomodoroError::NotRunning`] if the session is not running.
    pub fn pause(&mut self) -> Result<(), PomodoroError> {
        self.check_running()?;
        self.running = false;
        self.frozen_remaining = self.remaining_time();
        if let Some(anchor) = self.anchor {
            self.accumulated += anchor.elapsed();
        }
        self.anchor = None;
        Ok(())
    }

    /// Resumes timer.
    ///
    /// # Errors
    ///
    /// Returns [`PomodoroError::Running`] if session is running.
    pub fn resume(&mut self) -> Result<(), PomodoroError> {
        self.check_not_running()?;
        self.running = true;
        self.anchor = Some(Instant::now());
        Ok(())
    }

    /// Toggles paused state.
    pub fn toggle_pause(&mut self) {
        // Guaranteed to not panic
        if self.running {
            self.pause().unwrap()
        } else {
            self.resume().unwrap()
        }
    }

    /// Skips to the next session.
    pub fn skip(&mut self) {
        self.next_state();
        self.reset_time();
    }

    /// Resets timer of current running session.
    pub fn reset(&mut self) {
        self.reset_time();
    }

    pub fn state(&self) -> PomodoroState {
        self.state
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn focus_sessions(&self) -> u32 {
        self.focus_sessions
    }

    pub fn total_sessions(&self) -> u32 {
        self.total_sessions
    }

    pub fn started_at(&self) -> Option<Instant> {
        self.anchor
    }

    pub fn total_time(&self) -> Duration {
        self.accumulated + self.anchor.map_or(Duration::ZERO, |a| a.elapsed())
    }

    pub fn remaining_time(&self) -> Duration {
        match self.anchor {
            Some(anchor) => self.frozen_remaining.saturating_sub(anchor.elapsed()),
            None => self.frozen_remaining,
        }
    }

    pub fn long_interval(&self) -> u32 {
        self.long_interval
    }

    /// Gets session duration based on current state.
    pub fn session_duration(&self) -> Duration {
        match self.state {
            Focus => self.focus,
            LongBreak => self.long_break,
            ShortBreak => self.short_break,
        }
    }

    fn reset_time(&mut self) {
        if let Some(anchor) = self.anchor {
            self.accumulated += anchor.elapsed();
        }
        self.frozen_remaining = self.session_duration();
        self.anchor = if self.running {
            Some(Instant::now())
        } else {
            None
        };
    }

    /// Returns [`PomodoroError::NotRunning`] if session is not running yet.
    fn check_running(&self) -> Result<(), PomodoroError> {
        if !self.is_running() {
            return Err(PomodoroError::NotRunning);
        }
        Ok(())
    }

    /// Returns [`PomodoroError::Running`] if session is running.
    fn check_not_running(&self) -> Result<(), PomodoroError> {
        if self.is_running() {
            return Err(PomodoroError::Running);
        }
        Ok(())
    }

    /// Sets session state based on previous state.
    fn next_state(&mut self) {
        self.total_sessions += 1;
        match self.state {
            Focus => {
                self.focus_sessions += 1;
                if self.focus_sessions.is_multiple_of(self.long_interval) {
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
            state: PomodoroState::Focus,
            focus: Duration::from_mins(25),
            long_break: Duration::from_mins(15),
            short_break: Duration::from_mins(5),
            long_interval: 3,
            focus_sessions: 0,
            total_sessions: 0,
            accumulated: Duration::ZERO,
            running: false,
            frozen_remaining: Duration::from_mins(25),
            anchor: None,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum PomodoroState {
    Focus,
    LongBreak,
    ShortBreak,
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
        let mut pomo = Pomodoro {
            state: ShortBreak,
            ..Default::default()
        };

        pomo.next_state();
        assert_eq!(pomo.state(), Focus)
    }

    #[test]
    fn test_next_state_short_break() {
        let mut pomo = Pomodoro::default();

        pomo.next_state();
        assert_eq!(pomo.state(), ShortBreak)
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
        assert_eq!(pomo.state(), LongBreak)
    }

    #[test]
    fn test_running_checks() {
        let mut pomo = Pomodoro {
            running: false,
            ..Default::default()
        };

        assert_eq!(pomo.check_running(), Err(PomodoroError::NotRunning));

        pomo.running = true;
        assert!(matches!(
            pomo.check_not_running(),
            Err(PomodoroError::Running)
        ));
    }

    #[test]
    fn test_skip() {
        let mut pomo = Pomodoro {
            focus_sessions: 1,
            total_sessions: 1,
            state: Focus,
            running: true,
            anchor: None,
            ..Default::default()
        };

        pomo.skip();

        assert_eq!((pomo.focus_sessions(), pomo.total_sessions()), (2, 2));
        assert_eq!(pomo.state(), ShortBreak);

        assert_ne!(pomo.started_at(), None);
        let diff = pomo
            .session_duration()
            .saturating_sub(pomo.remaining_time());
        assert!(diff < Duration::from_secs(1));
    }

    #[test]
    fn test_session_counts() {
        // 0 0
        let mut pomo = Pomodoro {
            focus_sessions: 0,
            total_sessions: 0,
            state: Focus,
            long_interval: 2,
            ..Default::default()
        };
        pomo.start().unwrap();

        // 1 1
        pomo.skip();
        assert_eq!((pomo.focus_sessions(), pomo.total_sessions()), (1, 1));
        // 1 2
        pomo.skip();
        assert_eq!((pomo.focus_sessions(), pomo.total_sessions()), (1, 2));
        // 2 3
        pomo.skip();
        assert_eq!((pomo.focus_sessions(), pomo.total_sessions()), (2, 3));
    }

    #[test]
    fn test_pause_resume() {
        let mut pomo = Pomodoro::default();

        pomo.start().unwrap();

        pomo.pause().unwrap();
        assert!(!pomo.is_running());
        assert_eq!(pomo.started_at(), None);

        pomo.resume().unwrap();
        assert!(pomo.is_running());
        assert_ne!(pomo.started_at(), None);
    }

    #[test]
    fn test_add() {
        let mut pomo = Pomodoro {
            frozen_remaining: Duration::from_secs(67),
            running: false,
            ..Default::default()
        };

        pomo.add(Duration::from_secs(2));
        assert_eq!(pomo.remaining_time(), Duration::from_secs(69));

        pomo.subtract(Duration::from_secs(2));
        assert_eq!(pomo.remaining_time(), Duration::from_secs(67));
    }

    #[test]
    fn test_update() {
        let past = Instant::now().checked_sub(Duration::from_secs(1)).unwrap();

        let pomo = Pomodoro {
            frozen_remaining: Duration::from_secs(67),
            anchor: Some(past),
            accumulated: Duration::ZERO,
            running: true,
            ..Default::default()
        };

        let remaining = pomo.remaining_time().as_secs();
        assert!(remaining <= 66, "Expected <= 66, got {}", remaining);
        assert!(remaining >= 65, "Expected >= 65, got {}", remaining);

        let total = pomo.total_time().as_secs();
        assert!(total >= 1, "Expected total_time >= 1, got {}", total);
    }

    #[test]
    fn test_pause_resume_accumulates_total_time() {
        let mut pomo = Pomodoro::default();
        pomo.start().unwrap();

        std::thread::sleep(Duration::from_millis(50));
        pomo.pause().unwrap();

        let total_after_pause = pomo.total_time();
        assert!(total_after_pause >= Duration::from_millis(40));

        pomo.resume().unwrap();
        std::thread::sleep(Duration::from_millis(50));
        pomo.pause().unwrap();

        let total_after_second_pause = pomo.total_time();
        assert!(total_after_second_pause >= total_after_pause + Duration::from_millis(40));
    }
}
