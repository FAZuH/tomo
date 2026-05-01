use std::path::PathBuf;
use std::time::Duration;

use crate::config::Config;
use crate::config::Percentage;
use crate::ui::prelude::*;

pub const SETTINGS_VIEW_ITEMS: u32 = 17;

#[derive(Clone, Debug, PartialEq)]
pub enum ConfigMsg {
    // Timer settings
    AutoStartOnLaunch,
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
    // Alarm path settings
    AlarmPathFocus(Option<PathBuf>),
    AlarmPathShort(Option<PathBuf>),
    AlarmPathLong(Option<PathBuf>),
    // Alarm volume settings
    AlarmVolumeFocus(Percentage),
    AlarmVolumeShort(Percentage),
    AlarmVolumeLong(Percentage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigCmd {
    None,
}

impl Updateable for Config {
    type Msg = ConfigMsg;
    type Cmd = ConfigCmd;

    fn update(&mut self, msg: Self::Msg) -> Self::Cmd {
        use ConfigMsg::*;
        let timer = &mut self.pomodoro.timer;
        let hook = &mut self.pomodoro.hook;
        let alarm = &mut self.pomodoro.alarm;
        let cmd = ConfigCmd::None;
        match msg {
            // Timer
            AutoStartOnLaunch => timer.auto_start_on_launch = !timer.auto_start_on_launch,
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
            // Alarm
            AlarmPathFocus(p) => alarm.focus.path = p,
            AlarmPathShort(p) => alarm.short.path = p,
            AlarmPathLong(p) => alarm.long.path = p,
            AlarmVolumeFocus(v) => alarm.focus.volume = v,
            AlarmVolumeShort(v) => alarm.short.volume = v,
            AlarmVolumeLong(v) => alarm.long.volume = v,
        }
        cmd
    }
}
