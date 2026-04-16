use std::path::PathBuf;
use std::time::Duration;

use crate::models::pomodoro::PomodoroState;
use crate::ui::FromInput;
use crate::ui::Input;
use crate::ui::Navigation;
use crate::ui::Page;

#[derive(Clone, Debug, PartialEq)]
pub enum RenderCommand {
    Timer(Vec<TimerRenderCommand>),
    Settings(Vec<SettingsRenderCommand>),
}

pub trait TimerView {
    fn render(&self, state: TimerViewState) -> Vec<TimerRenderCommand>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimerViewState {
    pub remaining: Duration,
    pub total: Duration,
    pub state: PomodoroState,
    pub running: bool,
    pub long_interval: u32,
    pub total_sessions: u32,
    pub focus_sessions: u32,
    pub progress_perc: f64,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum TimerViewActions {
    // forward
    Add30Sec,
    Add1Min,

    // backward
    Sub30Sec,
    Sub1Min,

    // session
    TogglePause,
    SkipSession,
    ResetSession,

    GoSettings,
    Quit,
}

impl FromInput for TimerViewActions {
    fn from_input(input: Input) -> Option<Self> {
        use Input::*;
        use TimerViewActions::*;
        let ret = match input {
            Left => Sub30Sec,
            Down => Sub1Min,
            Right => Add30Sec,
            Up => Add1Min,
            Char(' ') => TogglePause,
            Enter => SkipSession,
            Backspace => ResetSession,
            Char('q') => Quit,
            _ => return None,
        };
        Some(ret)
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum TimerRenderCommand {
    State(PomodoroState),
    Timer {
        remaining: Duration,
    },
    PauseIndicator(bool),
    Stats {
        remaining: Duration,
        total: Duration,
        long_interval: u32,
        total_sessions: u32,
        focus_sessions: u32,
    },
    Progress(f64),
}

pub trait SettingsView {
    fn render(&self, state: SettingsViewState) -> Vec<SettingsRenderCommand>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsViewState {
    pub timer_focus: Duration,
    pub timer_short: Duration,
    pub timer_long: Duration,
    pub timer_long_interval: u32,
    pub timer_auto_focus: bool,
    pub timer_auto_short: bool,
    pub timer_auto_long: bool,
    pub hook_focus: String,
    pub hook_short: String,
    pub hook_long: String,
    pub sound_focus: Option<PathBuf>,
    pub sound_short: Option<PathBuf>,
    pub sound_long: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingsViewActions {
    SelectDown,
    SelectUp,
    EditSelection,

    Navigate(Navigation),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingsRenderCommand {
    SettingsHeader,
    SettingsField { label: String, value: String },
}
