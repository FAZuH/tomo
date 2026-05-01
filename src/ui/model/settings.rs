use std::fmt::Display;

use tui_widgets::prompts::FocusState;
use tui_widgets::prompts::State;
use tui_widgets::prompts::TextState;
use tui_widgets::scrollview::ScrollViewState;

use crate::config::pomodoro::PomodoroConfig;
use crate::ui::prelude::*;
use crate::ui::tui::view::settings::SettingsPrompt;
use crate::ui::update::config::SETTINGS_VIEW_ITEMS;

static SETTINGS_SECTIONS: u32 = 3;

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsMsg {
    SelectUp,
    SelectDown,
    SectionPrev,
    SectionNext,
    SectionSelect(u32),
    ScrollUp,
    ScrollDown,
    // StartEditing(PomodoroConfig),
    CancelEditing,
    TakeEditValue,
    SetUnsavedChanges(bool),
    SetShowKeybinds(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsCmd {
    None,
    EditValue(Option<String>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SettingsItem {
    // Timer settings
    AutoStartOnLaunch,
    TimerFocus,
    TimerShort,
    TimerLong,
    TimerLongInterval,
    // Toggles
    TimerAutoFocus,
    TimerAutoShort,
    TimerAutoLong,

    // Hook settings
    HookFocus,
    HookShort,
    HookLong,

    // Alarm path settings
    AlarmPathFocus,
    AlarmPathShort,
    AlarmPathLong,
    // Alarm volume settings
    AlarmVolumeFocus,
    AlarmVolumeShort,
    AlarmVolumeLong,
}

impl SettingsItem {
    pub fn index(&self) -> u32 {
        use SettingsItem::*;
        match self {
            TimerFocus => 0,
            TimerShort => 1,
            TimerLong => 2,
            TimerLongInterval => 3,
            AutoStartOnLaunch => 4,
            TimerAutoFocus => 5,
            TimerAutoShort => 6,
            TimerAutoLong => 7,
            HookFocus => 8,
            HookShort => 9,
            HookLong => 10,
            AlarmPathFocus => 11,
            AlarmPathShort => 12,
            AlarmPathLong => 13,
            AlarmVolumeFocus => 14,
            AlarmVolumeShort => 15,
            AlarmVolumeLong => 16,
        }
    }

    pub fn from_index(idx: u32) -> Option<Self> {
        use SettingsItem::*;
        let ret = match idx {
            0 => TimerFocus,
            1 => TimerShort,
            2 => TimerLong,
            3 => TimerLongInterval,
            4 => AutoStartOnLaunch,
            5 => TimerAutoFocus,
            6 => TimerAutoShort,
            7 => TimerAutoLong,
            8 => HookFocus,
            9 => HookShort,
            10 => HookLong,
            11 => AlarmPathFocus,
            12 => AlarmPathShort,
            13 => AlarmPathLong,
            14 => AlarmVolumeFocus,
            15 => AlarmVolumeShort,
            16 => AlarmVolumeLong,
            _ => return None,
        };
        Some(ret)
    }

    pub fn label_long(&self) -> &'static str {
        match self.index() {
            0 => "Focus Duration",
            1 => "Short Break Duration",
            2 => "Long Break Duration",
            3 => "Long Break Interval",

            8 => "Focus Hook Command",
            9 => "Short Break Hook Command",
            10 => "Long Break Hook Command",

            11 => "Focus Alarm Sound File Path",
            12 => "Short Break Alarm Sound File Path",
            13 => "Long Break Alarm Sound File Path",

            14 => "Focus Alarm Volume",
            15 => "Short Break Alarm Volume",
            16 => "Long Break Alarm Volume",
            _ => panic!("label called on invalid item"),
        }
    }

    pub fn label(&self) -> &'static str {
        match self.index() {
            0 => "Focus",
            1 => "Short Break",
            2 => "Long Break",
            3 => "Long Break Interval",

            4 => "Auto-start on Launch",
            5 => "Focus",
            6 => "Short Break",
            7 => "Long Break",

            8 => "Focus",
            9 => "Short Break",
            10 => "Long Break",

            11 => "Focus",
            12 => "Short Break",
            13 => "Long Break",

            14 => "Focus",
            15 => "Short Break",
            16 => "Long Break",
            _ => panic!("label called on invalid item"),
        }
    }

    pub fn section(&self) -> SettingsSection {
        SettingsSection::from_item_index(self.index()).unwrap()
    }

    pub fn is_toggle(&self) -> bool {
        Self::toggles().contains(self)
    }

    pub fn is_percentage(&self) -> bool {
        Self::percentages().contains(self)
    }

    fn toggles() -> Vec<Self> {
        use SettingsItem::*;
        vec![
            TimerAutoFocus,
            TimerAutoShort,
            TimerAutoLong,
            AutoStartOnLaunch,
        ]
    }

    fn percentages() -> Vec<Self> {
        use SettingsItem::*;
        vec![AlarmVolumeFocus, AlarmVolumeLong, AlarmVolumeShort]
    }
}

impl Display for SettingsItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label_long())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsSection {
    Timer,
    Hook,
    Alarm,
}

impl SettingsSection {
    pub fn from_item_index(idx: u32) -> Option<Self> {
        use SettingsSection::*;
        let ret = match idx {
            0..=7 => Timer,
            8..=10 => Hook,
            11..=16 => Alarm,
            _ => return None,
        };
        Some(ret)
    }

    pub fn from_index(idx: u32) -> Option<Self> {
        use SettingsSection::*;
        let ret = match idx {
            0 => Timer,
            1 => Hook,
            2 => Alarm,
            _ => return None,
        };
        Some(ret)
    }

    pub fn index(&self) -> u32 {
        use SettingsSection::*;
        match self {
            Timer => 0,
            Hook => 1,
            Alarm => 2,
        }
    }

    pub fn item_begin_idx(&self) -> u32 {
        match self.index() {
            0 => 0,
            1 => 7,
            2 => 10,
            _ => panic!("label called on invalid section"),
        }
    }

    pub fn label(&self) -> &'static str {
        match self.index() {
            0 => "Timer",
            1 => "Hook",
            2 => "Alarm",
            _ => panic!("label called on invalid section"),
        }
    }
}

impl From<SettingsItem> for SettingsSection {
    fn from(value: SettingsItem) -> Self {
        value.section()
    }
}

pub struct SettingsModel {
    selected: SettingsItem,
    scroll_state: ScrollViewState,
    prompt: Option<SettingsPrompt>,
    has_unsaved_changes: bool,
    show_keybinds: bool,
}

impl Updateable for SettingsModel {
    type Msg = SettingsMsg;
    type Cmd = SettingsCmd;

    fn update(&mut self, msg: Self::Msg) -> Self::Cmd {
        use SettingsMsg::*;
        let mut cmd = SettingsCmd::None;

        match msg {
            SelectUp => self.select_up(),
            SelectDown => self.select_down(),
            SectionPrev => self.prev_section(),
            SectionNext => self.next_section(),
            SectionSelect(idx) => self.select_section(SettingsSection::from_index(idx).unwrap()),
            ScrollUp => self.scroll_up(),
            ScrollDown => self.scroll_down(),
            CancelEditing => self.cancel_editing(),
            TakeEditValue => {
                cmd = SettingsCmd::EditValue(
                    self.prompt.take().map(|v| v.text_state.value().to_string()),
                );
            }
            SetUnsavedChanges(v) => self.has_unsaved_changes = v,
            SetShowKeybinds(v) => self.show_keybinds = v,
        }

        cmd
    }
}

impl SettingsModel {
    pub fn new() -> Self {
        Self {
            selected: SettingsItem::TimerFocus,
            scroll_state: ScrollViewState::default(),
            prompt: None,
            has_unsaved_changes: false,
            show_keybinds: false,
        }
    }

    pub fn take_edit_value(&mut self) -> String {
        if let SettingsCmd::EditValue(Some(v)) = self.update(SettingsMsg::TakeEditValue) {
            v
        } else {
            String::new()
        }
    }

    pub fn prompt_state_mut(&mut self) -> Option<&mut SettingsPrompt> {
        self.prompt.as_mut()
    }

    pub fn scroll_state_mut(&mut self) -> &mut ScrollViewState {
        &mut self.scroll_state
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.has_unsaved_changes
    }

    /// Get current selection index
    pub fn selected(&self) -> SettingsItem {
        self.selected
    }

    /// Check if currently editing
    pub fn is_editing(&self) -> bool {
        self.prompt.is_some()
    }

    pub fn show_keybinds(&self) -> bool {
        self.show_keybinds
    }

    pub fn toggle_keybinds(&mut self) {
        let new = !self.show_keybinds;
        self.update(SettingsMsg::SetShowKeybinds(new));
    }

    pub fn start_editing_for_field(&mut self, config: &PomodoroConfig) {
        let alarm = &config.alarm;
        let hook = &config.hook;
        let timer = &config.timer;
        use SettingsItem::*;

        let mut value = match self.selected {
            TimerFocus => format!("{}", timer.focus.as_secs() / 60),
            TimerShort => format!("{}", timer.short.as_secs() / 60),
            TimerLong => format!("{}", timer.long.as_secs() / 60),
            TimerLongInterval => format!("{}", timer.long_interval),
            HookFocus => hook.focus.clone(),
            HookShort => hook.short.clone(),
            HookLong => hook.long.clone(),
            AlarmPathFocus => alarm.focus.path(),
            AlarmPathShort => alarm.short.path(),
            AlarmPathLong => alarm.long.path(),
            AlarmVolumeFocus => alarm.focus.volume(),
            AlarmVolumeShort => alarm.short.volume(),
            AlarmVolumeLong => alarm.long.volume(),
            AutoStartOnLaunch | TimerAutoFocus | TimerAutoShort | TimerAutoLong => return,
        };

        if self.selected.is_percentage() {
            value = value[..value.len() - 1].to_string();
        }

        let value_len = value.len();
        let mut text_state = TextState::new()
            .with_focus(FocusState::Focused)
            .with_value(value);
        *State::position_mut(&mut text_state) = value_len;

        self.prompt = Some(SettingsPrompt {
            text_state,
            label: self.selected().to_string(),
        });
    }

    /// Select item up
    fn select_up(&mut self) {
        let idx = self
            .selected
            .index()
            .saturating_sub(1)
            .clamp(0, SETTINGS_VIEW_ITEMS - 1); // 13 items total
        self.selected = SettingsItem::from_index(idx).unwrap();
    }

    /// Select item down
    fn select_down(&mut self) {
        let idx = self
            .selected
            .index()
            .saturating_add(1)
            .clamp(0, SETTINGS_VIEW_ITEMS - 1); // 13 items total
        self.selected = SettingsItem::from_index(idx).unwrap();
    }

    fn prev_section(&mut self) {
        let idx = (self.selected.section().index() + SETTINGS_SECTIONS - 1) % SETTINGS_SECTIONS;
        self.selected =
            SettingsItem::from_index(SettingsSection::from_index(idx).unwrap().item_begin_idx())
                .unwrap();
    }

    fn next_section(&mut self) {
        let idx = (self.selected.section().index() + 1) % SETTINGS_SECTIONS;
        self.selected =
            SettingsItem::from_index(SettingsSection::from_index(idx).unwrap().item_begin_idx())
                .unwrap();
    }

    fn select_section(&mut self, section: SettingsSection) {
        self.selected = SettingsItem::from_index(section.item_begin_idx()).unwrap();
    }

    /// Scroll up by one row
    fn scroll_up(&mut self) {
        self.scroll_state.scroll_up();
    }

    /// Scroll down by one row
    fn scroll_down(&mut self) {
        self.scroll_state.scroll_down();
    }

    /// Cancel editing
    fn cancel_editing(&mut self) {
        self.prompt = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_idx() {
        use SettingsItem::*;
        use SettingsSection::*;
        assert_eq!(TimerFocus.section(), Timer);
        assert_eq!(TimerAutoLong.section(), Timer);

        assert_eq!(HookFocus.section(), Hook);
        assert_eq!(HookLong.section(), Hook);

        assert_eq!(AlarmPathFocus.section(), Alarm);
        assert_eq!(AlarmVolumeLong.section(), Alarm);
    }
}
