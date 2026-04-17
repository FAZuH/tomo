use std::path::PathBuf;
use std::time::Duration;

use crate::models::pomodoro::PomodoroState;
use crate::ui::Navigation;

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
pub enum TimerActions {
    Add(Duration),
    Subtract(Duration),
    TogglePause,
    SkipSession,
    ResetSession,

    Navigate(Navigation),
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

pub const SETTINGS_VIEW_ITEMS: u32 = 13;

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
pub enum SettingsActions {
    // Timer settings
    TimerFocus(Duration),
    TimerShort(Duration),
    TimerLong(Duration),
    TimerLongInterval(u32),
    // Toggles
    TimerAutoFocus,
    TimerAutoShort,
    TimerAutoLong,

    // Hook settings
    HookFocus(String),
    HookShort(String),
    HookLong(String),

    // Sound settings
    SoundFocus(Option<PathBuf>),
    SoundShort(Option<PathBuf>),
    SoundLong(Option<PathBuf>),

    Navigate(Navigation),
}

impl SettingsActions {
    pub fn is_toggle(&self) -> bool {
        Self::is_toggle_index(self.index() as u32)
    }

    pub fn is_toggle_index(index: u32) -> bool {
        (4..=6).contains(&index)
    }

    /// Converts [`SettingsActions`] into index.
    ///
    /// Returns [`usize::MAX`] if [`SettingsActions::Navigate`].
    pub fn index(&self) -> usize {
        match self {
            Self::TimerFocus(_) => 0,
            Self::TimerShort(_) => 1,
            Self::TimerLong(_) => 2,
            Self::TimerLongInterval(_) => 3,
            Self::TimerAutoFocus => 4,
            Self::TimerAutoShort => 5,
            Self::TimerAutoLong => 6,
            Self::HookFocus(_) => 7,
            Self::HookShort(_) => 8,
            Self::HookLong(_) => 9,
            Self::SoundFocus(_) => 10,
            Self::SoundShort(_) => 11,
            Self::SoundLong(_) => 12,
            Self::Navigate(_) => usize::MAX,
        }
    }

    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::TimerFocus(Duration::default())),
            1 => Some(Self::TimerShort(Duration::default())),
            2 => Some(Self::TimerLong(Duration::default())),
            3 => Some(Self::TimerLongInterval(0)),
            4 => Some(Self::TimerAutoFocus),
            5 => Some(Self::TimerAutoShort),
            6 => Some(Self::TimerAutoLong),
            7 => Some(Self::HookFocus(String::new())),
            8 => Some(Self::HookShort(String::new())),
            9 => Some(Self::HookLong(String::new())),
            10 => Some(Self::SoundFocus(None)),
            11 => Some(Self::SoundShort(None)),
            12 => Some(Self::SoundLong(None)),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SectionLayout {
    Vertical,
    Horizontal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingsRenderCommand {
    Title,
    Section {
        label: String,
        children: Vec<Self>,
    },
    SubSection {
        label: String,
        layout: SectionLayout,
        children: Vec<Self>,
    },
    Input {
        label: String,
        value: String,
    },
    Checkbox {
        label: String,
        value: bool,
    },
}

impl SettingsRenderCommand {
    pub fn section(label: impl ToString, children: Vec<Self>) -> Self {
        Self::Section {
            label: label.to_string(),
            children,
        }
    }

    pub fn subsection(label: impl ToString, children: Vec<Self>) -> Self {
        Self::SubSection {
            label: label.to_string(),
            layout: SectionLayout::Vertical,
            children,
        }
    }

    pub fn subsection_horizontal(label: impl ToString, children: Vec<Self>) -> Self {
        Self::SubSection {
            label: label.to_string(),
            layout: SectionLayout::Horizontal,
            children,
        }
    }

    pub fn input(label: impl ToString, value: impl ToString) -> Self {
        Self::Input {
            label: label.to_string(),
            value: value.to_string(),
        }
    }

    pub fn checkbox(label: impl ToString, value: bool) -> Self {
        Self::Checkbox {
            label: label.to_string(),
            value,
        }
    }
}
