use std::time::Duration;

use crate::models::pomodoro::PomodoroState;
use crate::ui::FromInput;
use crate::ui::Input;
use crate::ui::Page;

#[derive(Clone, Debug, PartialEq)]
pub enum RenderCommand {
    Home(Vec<HomeRenderCommand>),
    Timer(Vec<TimerRenderCommand>),
    Settings(Vec<SettingsRenderCommand>),
}

pub trait HomeView {
    fn render(&self) -> Vec<HomeRenderCommand>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HomeRenderCommand {
    WelcomeText,
    NavButton(String, Page),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HomeViewActions {
    GoToTimer,
    GoToSettings,
    Quit,
}

pub trait TimerView {
    fn render(&self, state: TimerViewState) -> Vec<TimerRenderCommand>;
}

pub struct TimerViewState {
    pub remaining: Duration,
    pub total: Duration,
    pub state: PomodoroState,
    pub paused: bool,
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

    GoHome,
    GoSettings,
    Quit,
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum TimerRenderCommand {
    TimerDisplay {
        remaining: Duration,
        total: Duration,
    },
    SessionLabel(PomodoroState),
    PauseIndicator(bool),
    ProgressBar(f64),
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
            Backspace => ResetSession,
            Enter => SkipSession,
            _ => return None,
        };
        Some(ret)
    }
}

impl FromInput for HomeViewActions {
    fn from_input(input: Input) -> Option<Self> {
        use HomeViewActions::*;
        use Input::*;
        let ret = match input {
            Right => GoToSettings,
            Enter => GoToTimer,
            Esc => Quit,
            Char('q') => Quit,
            _ => return None,
        };
        Some(ret)
    }
}
