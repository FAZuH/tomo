use std::fmt::Display;
use std::path::PathBuf;
use std::time::Duration;

use strum::EnumCount;
use strum::EnumDiscriminants;
use strum::EnumIter;
use strum::EnumMessage;
use strum::EnumProperty;
use strum::FromRepr;
use strum::IntoStaticStr;
use strum::VariantArray;

use crate::config::Config;
use crate::config::PomodoroConfig;
use crate::config::Volume;
use crate::ui::prelude::*;

#[derive(
    Clone, Debug, PartialEq, EnumDiscriminants, EnumCount, EnumMessage, FromRepr, EnumProperty,
)]
#[strum_discriminants(derive(PartialOrd, Ord, FromRepr, EnumCount, EnumIter, VariantArray,))]
#[strum_discriminants(name(SettingsItem))]
pub enum ConfigMsg {
    // Timer section
    #[strum(
        message = "Focus",
        detailed_message = "Focus Duration",
        props(
            description = "Duration of the main focus session in minutes.",
            type = "duration"
        )
    )]
    TimerFocus(Duration),
    #[strum(
        message = "Short Break",
        detailed_message = "Short Break Duration",
        props(
            description = "Duration of a short break in minutes.",
            type = "duration"
        )
    )]
    TimerShort(Duration),
    #[strum(
        message = "Long Break",
        detailed_message = "Long Break Duration",
        props(
            description = "Duration of a long break in minutes.",
            type = "duration"
        )
    )]
    TimerLong(Duration),
    #[strum(
        message = "Long Break Interval",
        detailed_message = "Long Break Interval",
        props(
            description = "Number of focus sessions before a long break.",
            type = "number"
        )
    )]
    TimerLongInterval(u32),
    #[strum(
        message = "Auto-start on Launch",
        props(
            description = "Automatically start the timer when the application launches.",
            type = "toggle"
        )
    )]
    AutoStartOnLaunch,
    #[strum(
        message = "Focus",
        props(
            description = "Automatically start the next focus session after a break.",
            type = "toggle"
        )
    )]
    TimerAutoFocus,
    #[strum(
        message = "Short Break",
        props(
            description = "Automatically start a short break after a focus session.",
            type = "toggle"
        )
    )]
    TimerAutoShort,
    #[strum(
        message = "Long Break",
        props(
            description = "Automatically start a long break when the interval is reached.",
            type = "toggle"
        )
    )]
    TimerAutoLong,

    // Hook section
    #[strum(
        message = "Focus",
        detailed_message = "Focus Hook Command",
        props(
            description = "Shell command to execute when a focus session starts.",
            type = "string"
        )
    )]
    HookFocus(String),
    #[strum(
        message = "Short Break",
        detailed_message = "Short Break Hook Command",
        props(
            description = "Shell command to execute when a short break starts.",
            type = "string"
        )
    )]
    HookShort(String),
    #[strum(
        message = "Long Break",
        detailed_message = "Long Break Hook Command",
        props(
            description = "Shell command to execute when a long break starts.",
            type = "string"
        )
    )]
    HookLong(String),

    // Alarm section
    #[strum(
        message = "Focus",
        detailed_message = "Focus Alarm Sound File Path",
        props(
            description = "Path to the audio file played when a focus session ends.",
            type = "path"
        )
    )]
    AlarmPathFocus(Option<PathBuf>),
    #[strum(
        message = "Short Break",
        detailed_message = "Short Break Alarm Sound File Path",
        props(
            description = "Path to the audio file played when a short break ends.",
            type = "path"
        )
    )]
    AlarmPathShort(Option<PathBuf>),
    #[strum(
        message = "Long Break",
        detailed_message = "Long Break Alarm Sound File Path",
        props(
            description = "Path to the audio file played when a long break ends.",
            type = "path"
        )
    )]
    AlarmPathLong(Option<PathBuf>),
    #[strum(
        message = "Focus",
        detailed_message = "Focus Alarm Volume",
        props(
            description = "Volume level (0-100) for the focus alarm.",
            type = "percentage"
        )
    )]
    AlarmVolumeFocus(Volume),
    #[strum(
        message = "Short Break",
        detailed_message = "Short Break Alarm Volume",
        props(
            description = "Volume level (0-100) for the short break alarm.",
            type = "percentage"
        )
    )]
    AlarmVolumeShort(Volume),
    #[strum(
        message = "Long Break",
        detailed_message = "Long Break Alarm Volume",
        props(
            description = "Volume level (0-100) for the long break alarm.",
            type = "percentage"
        )
    )]
    AlarmVolumeLong(Volume),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigCmd {
    None,
}

impl Updateable<ConfigMsg, ConfigCmd> for Config {
    fn update(&mut self, msg: ConfigMsg) -> Vec<ConfigCmd> {
        use ConfigMsg::*;
        let timer = &mut self.pomodoro.timer;
        let hook = &mut self.pomodoro.hook;
        let alarm = &mut self.pomodoro.alarm;
        let cmd = vec![ConfigCmd::None];
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

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsMsg<'a> {
    /// Carries a ConfigMsg constructed by the view layer (post-split flow).
    ApplyEdit(ConfigMsg),
    StartEdit(&'a PomodoroConfig),
    CancelEditing,
    SaveConfig,
    SaveEdit,
    ScrollDown,
    ScrollUp,
    SectionNext,
    SectionPrev,
    SectionSelect(u32),
    SelectDown,
    SelectUp,
    SetShowKeybinds(bool),
    ToggleShowKeybinds,
    SelectForCopy,
    CopyValue(&'a PomodoroConfig),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsCmd {
    SaveEdit(ConfigMsg),
    SaveConfig,
    ShowToast { message: String, r#type: ToastType },
}

impl SettingsItem {
    pub fn index(&self) -> u32 {
        *self as u32
    }

    pub fn from_index(idx: u32) -> Option<Self> {
        Self::from_repr(idx as usize)
    }

    pub fn label_long(&self) -> &'static str {
        self.config_msg().get_detailed_message().unwrap()
    }

    pub fn description(&self) -> &'static str {
        self.config_msg().get_str("description").unwrap_or("")
    }

    fn config_msg(&self) -> ConfigMsg {
        ConfigMsg::from_repr(self.index() as usize).unwrap()
    }

    pub fn label(&self) -> &'static str {
        self.config_msg().get_message().unwrap()
    }

    pub fn section(&self) -> SettingsSection {
        SettingsSection::from_item_index(self.index()).unwrap()
    }

    pub fn is_toggle(&self) -> bool {
        self.config_msg().get_str("type") == Some("toggle")
    }

    pub fn is_percentage(&self) -> bool {
        self.config_msg().get_str("type") == Some("percentage")
    }

    pub fn is_path(&self) -> bool {
        self.config_msg().get_str("type") == Some("path")
    }
}

impl Display for SettingsItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label_long())
    }
}

impl Display for ConfigMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", SettingsItem::from(self).label_long())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumCount, FromRepr, IntoStaticStr, VariantArray)]
pub enum SettingsSection {
    Timer,
    Hook,
    Alarm,
}

impl SettingsSection {
    pub fn from_item(item: SettingsItem) -> Self {
        use SettingsItem::*;
        match item {
            AutoStartOnLaunch | TimerFocus | TimerShort | TimerLong | TimerLongInterval
            | TimerAutoFocus | TimerAutoShort | TimerAutoLong => Self::Timer,
            HookFocus | HookShort | HookLong => Self::Hook,
            AlarmPathFocus | AlarmPathShort | AlarmPathLong | AlarmVolumeFocus
            | AlarmVolumeShort | AlarmVolumeLong => Self::Alarm,
        }
    }

    pub fn from_item_index(idx: u32) -> Option<Self> {
        SettingsItem::from_repr(idx as usize).map(Self::from_item)
    }
    pub fn item_begin_idx(&self) -> u32 {
        use SettingsItem::*;
        match self {
            SettingsSection::Timer => TimerFocus as u32,
            SettingsSection::Hook => HookFocus as u32,
            SettingsSection::Alarm => AlarmPathFocus as u32,
        }
    }

    pub fn from_index(idx: u32) -> Option<Self> {
        Self::from_repr(idx as usize)
    }

    pub fn index(&self) -> u32 {
        *self as u32
    }

    pub fn label(&self) -> &'static str {
        self.into()
    }

    pub fn items(&self) -> &[SettingsItem] {
        use SettingsItem::*;
        match self {
            SettingsSection::Timer => {
                &SettingsItem::VARIANTS[TimerFocus as usize..=TimerAutoLong as usize]
            }
            SettingsSection::Hook => {
                &SettingsItem::VARIANTS[HookFocus as usize..=HookLong as usize]
            }
            SettingsSection::Alarm => {
                &SettingsItem::VARIANTS[AlarmPathFocus as usize..=AlarmVolumeLong as usize]
            }
        }
    }
}

impl From<SettingsItem> for SettingsSection {
    fn from(value: SettingsItem) -> Self {
        value.section()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Error,
    Warning,
    Success,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn settings_item_props() {
        assert_eq!(SettingsItem::TimerFocus.label(), "Focus");
        assert_eq!(SettingsItem::TimerFocus.label_long(), "Focus Duration");
        assert_eq!(
            SettingsItem::TimerFocus.description(),
            "Duration of the main focus session in minutes."
        );
    }

    #[test]
    fn item_toggle() {
        assert!(SettingsItem::AutoStartOnLaunch.is_toggle());
        assert!(SettingsItem::TimerAutoFocus.is_toggle());
        assert!(!SettingsItem::TimerFocus.is_toggle());
    }

    #[test]
    fn item_percentage() {
        assert!(SettingsItem::AlarmVolumeFocus.is_percentage());
        assert!(SettingsItem::AlarmVolumeShort.is_percentage());
        assert!(!SettingsItem::TimerFocus.is_percentage());
    }

    #[test]
    fn item_path() {
        assert!(SettingsItem::AlarmPathFocus.is_path());
        assert!(SettingsItem::AlarmPathLong.is_path());
        assert!(!SettingsItem::TimerFocus.is_path());
    }

    #[test]
    fn item_index_roundtrip() {
        assert_eq!(
            SettingsItem::from_index(SettingsItem::TimerFocus.index()),
            Some(SettingsItem::TimerFocus)
        );
        assert_eq!(
            SettingsItem::from_index(SettingsItem::AlarmVolumeLong.index()),
            Some(SettingsItem::AlarmVolumeLong)
        );
    }

    #[test]
    fn item_section() {
        assert_eq!(SettingsItem::TimerFocus.section(), SettingsSection::Timer);
        assert_eq!(SettingsItem::HookFocus.section(), SettingsSection::Hook);
        assert_eq!(
            SettingsItem::AlarmPathFocus.section(),
            SettingsSection::Alarm
        );
    }

    #[test]
    fn item_display() {
        assert_eq!(SettingsItem::TimerFocus.to_string(), "Focus Duration");
    }

    #[test]
    fn config_msg_display() {
        assert_eq!(
            ConfigMsg::TimerFocus(Duration::from_secs(60)).to_string(),
            "Focus Duration"
        );
    }

    #[test]
    fn section_from_item() {
        assert_eq!(
            SettingsSection::from_item(SettingsItem::TimerShort),
            SettingsSection::Timer
        );
        assert_eq!(
            SettingsSection::from_item(SettingsItem::HookLong),
            SettingsSection::Hook
        );
    }

    #[test]
    fn section_item_begin_idx() {
        assert_eq!(
            SettingsSection::Timer.item_begin_idx(),
            SettingsItem::TimerFocus.index()
        );
        assert_eq!(
            SettingsSection::Hook.item_begin_idx(),
            SettingsItem::HookFocus.index()
        );
        assert_eq!(
            SettingsSection::Alarm.item_begin_idx(),
            SettingsItem::AlarmPathFocus.index()
        );
    }

    #[test]
    fn section_label() {
        assert_eq!(SettingsSection::Timer.label(), "Timer");
        assert_eq!(SettingsSection::Hook.label(), "Hook");
        assert_eq!(SettingsSection::Alarm.label(), "Alarm");
    }

    #[test]
    fn section_items_length() {
        assert_eq!(SettingsSection::Timer.items().len(), 8);
        assert_eq!(SettingsSection::Hook.items().len(), 3);
        assert_eq!(SettingsSection::Alarm.items().len(), 6);
    }

    #[test]
    fn config_update_toggle() {
        let mut config = Config::default();
        let auto_start = config.pomodoro.timer.auto_start_on_launch;

        config.update(ConfigMsg::AutoStartOnLaunch);

        assert_ne!(config.pomodoro.timer.auto_start_on_launch, auto_start);
    }

    #[test]
    fn config_update_duration() {
        let mut config = Config::default();
        let new_dur = Duration::from_secs(1800);

        config.update(ConfigMsg::TimerFocus(new_dur));

        assert_eq!(config.pomodoro.timer.focus, new_dur);
    }

    #[test]
    fn config_update_string() {
        let mut config = Config::default();

        config.update(ConfigMsg::HookFocus("notify-send".into()));

        assert_eq!(config.pomodoro.hook.focus, "notify-send");
    }

    #[test]
    fn config_update_returns_none() {
        let mut config = Config::default();
        let cmds = config.update(ConfigMsg::TimerAutoFocus);

        assert_eq!(cmds[0], ConfigCmd::None);
    }

    #[test]
    fn section_from_item_index() {
        assert_eq!(
            SettingsSection::from_item_index(SettingsItem::TimerFocus.index()),
            Some(SettingsSection::Timer)
        );
        assert_eq!(
            SettingsSection::from_item_index(SettingsItem::HookShort.index()),
            Some(SettingsSection::Hook)
        );
    }

    #[test]
    fn section_from_invalid_index() {
        assert_eq!(SettingsSection::from_item_index(999), None);
    }

    #[test]
    fn section_index_roundtrip() {
        assert_eq!(
            SettingsSection::from_index(SettingsSection::Timer.index()),
            Some(SettingsSection::Timer)
        );
    }
}
