use std::path::PathBuf;
use std::time::Duration;

use crate::config::Config;
use crate::config::ConfigError;
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
    // Other
    SaveToDisk,
}

impl SettingsMsg {
    pub fn is_toggle_index(index: u32) -> bool {
        (4..=6).contains(&index)
    }
}

#[derive(Debug)]
pub enum SettingsCmd {
    None,
    SavedToDisk(Result<(), ConfigError>),
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
    type Cmd = SettingsCmd;

    fn update(msg: Self::Msg, mut model: Self::Model) -> (Self::Model, Self::Cmd) {
        use SettingsMsg::*;
        let timer = &mut model.pomodoro.timer;
        let hook = &mut model.pomodoro.hook;
        let sound = &mut model.pomodoro.sound;
        let mut cmd = SettingsCmd::None;
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
            SaveToDisk => cmd = SettingsCmd::SavedToDisk(model.save()),
        }
        (model, cmd)
    }
}
