use std::path::PathBuf;
use std::time::Duration;

use crate::config::Config;
use crate::ui::Update;

pub const SETTINGS_VIEW_ITEMS: u32 = 13;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingsMsg {
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
}

impl SettingsMsg {
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

pub struct SettingsUpdate {}

impl SettingsUpdate {
    pub fn new() -> Self {
        Self {}
    }
}

impl Update for SettingsUpdate {
    type Msg = SettingsMsg;
    type Model = Config;

    fn update(msg: Self::Msg, mut model: Self::Model) -> Self::Model {
        use SettingsMsg::*;
        let timer = &mut model.pomodoro.timer;
        let hook = &mut model.pomodoro.hook;
        let sound = &mut model.pomodoro.sound;
        match msg {
            // Timer
            TimerFocus(d) => timer.focus = d,
            TimerShort(d) => timer.short = d,
            TimerLong(d) => timer.long = d,
            TimerLongInterval(n) => timer.long_interval = n,
            TimerAutoFocus => timer.auto_focus = !timer.auto_focus,
            TimerAutoShort => timer.auto_short = !timer.auto_short,
            TimerAutoLong => timer.auto_long = !timer.auto_long,
            // Hook
            HookFocus(s) => hook.focus = s,
            HookShort(s) => hook.short = s,
            HookLong(s) => hook.long = s,
            // Sound
            SoundFocus(p) => sound.focus = p,
            SoundShort(p) => sound.short = p,
            SoundLong(p) => sound.long = p,
        }
        model
    }
}
