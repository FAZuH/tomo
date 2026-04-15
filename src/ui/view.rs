use std::time::Duration;

use crate::models::pomodoro::PomodoroState;
use crate::ui::FromInput;
use crate::ui::Input;

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
    Add1Min,
    Add5Min,

    // backward
    Sub1Min,
    Sub5Min,

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
            Left => Sub1Min,
            Down => Sub5Min,
            Right => Add1Min,
            Up => Add5Min,
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
    pub focus: Duration,
    pub short: Duration,
    pub long: Duration,
    pub long_interval: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingsRenderCommand {
    SettingsHeader,
    SettingsField { label: String, value: String },
}
