use std::fmt::Display;

use tui_widgets::prompts::FocusState;
use tui_widgets::prompts::State;
use tui_widgets::prompts::TextState;
use tui_widgets::scrollview::ScrollViewState;

use crate::config::pomodoro::PomodoroConfig;
use crate::ui::Updateable;
use crate::ui::tui::view::settings::SettingsPrompt;
use crate::ui::update::config::SETTINGS_VIEW_ITEMS;

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsMsg {
    SelectUp,
    SelectDown,
    ScrollUp,
    ScrollDown,
    // StartEditing(PomodoroConfig),
    CancelEditing,
    TakeEditValue,
    SetUnsavedChanges(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsCmd {
    None,
    EditValue(Option<String>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsItem {
    // Timer settings
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
            TimerAutoFocus => 4,
            TimerAutoShort => 5,
            TimerAutoLong => 6,
            HookFocus => 7,
            HookShort => 8,
            HookLong => 9,
            AlarmPathFocus => 10,
            AlarmPathShort => 11,
            AlarmPathLong => 12,
            AlarmVolumeFocus => 13,
            AlarmVolumeShort => 14,
            AlarmVolumeLong => 15,
        }
    }

    pub fn from_index(idx: u32) -> Option<Self> {
        use SettingsItem::*;
        let ret = match idx {
            0 => TimerFocus,
            1 => TimerShort,
            2 => TimerLong,
            3 => TimerLongInterval,
            4 => TimerAutoFocus,
            5 => TimerAutoShort,
            6 => TimerAutoLong,
            7 => HookFocus,
            8 => HookShort,
            9 => HookLong,
            10 => AlarmPathFocus,
            11 => AlarmPathShort,
            12 => AlarmPathLong,
            13 => AlarmVolumeFocus,
            14 => AlarmVolumeShort,
            15 => AlarmVolumeLong,
            _ => return None,
        };
        Some(ret)
    }

    pub fn label_long(&self) -> &'static str {
        match self.index() {
            0 => "Focus",
            1 => "Short Break",
            2 => "Long Break",
            3 => "Long Break Interval",

            7 => "Focus Hook",
            8 => "Short Break Hook",
            9 => "Long Break Hook",

            10 => "Focus Alarm",
            11 => "Short Break Alarm",
            12 => "Long Break Alarm",

            13 => "Focus Alarm Volume",
            14 => "Short Break Alarm Volume",
            15 => "Long Break Alarm Volume",
            _ => panic!("label called on invalid item"),
        }
    }

    pub fn label(&self) -> &'static str {
        match self.index() {
            0 => "Focus",
            1 => "Short Break",
            2 => "Long Break",
            3 => "Long Break Interval",

            4 => "Focus",
            5 => "Short Break",
            6 => "Long Break",

            7 => "Focus",
            8 => "Short Break",
            9 => "Long Break",

            10 => "Focus",
            11 => "Short Break",
            12 => "Long Break",

            13 => "Focus",
            14 => "Short Break",
            15 => "Long Break",
            _ => panic!("label called on invalid item"),
        }
    }

    pub fn section(&self) -> SettingsSection {
        SettingsSection::from_index(self.index()).unwrap()
    }

    pub fn is_toggle(&self) -> bool {
        Self::toggles().contains(self)
    }

    fn toggles() -> Vec<Self> {
        use SettingsItem::*;
        vec![TimerAutoFocus, TimerAutoShort, TimerAutoLong]
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
    pub fn from_index(idx: u32) -> Option<Self> {
        use SettingsSection::*;
        let ret = match idx {
            0..=6 => Timer,
            7..=9 => Hook,
            10..=15 => Alarm,
            _ => return None,
        };
        Some(ret)
    }

    pub fn label(&self) -> &'static str {
        use SettingsSection::*;
        match self {
            Timer => "Timer",
            Hook => "Hook",
            Alarm => "Alarm",
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
            ScrollUp => self.scroll_up(),
            ScrollDown => self.scroll_down(),
            CancelEditing => self.cancel_editing(),
            TakeEditValue => {
                cmd = SettingsCmd::EditValue(
                    self.prompt.take().map(|v| v.text_state.value().to_string()),
                );
            }
            SetUnsavedChanges(v) => self.has_unsaved_changes = v,
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

    pub fn start_editing_for_field(&mut self, config: &PomodoroConfig) {
        let alarm = &config.alarm;
        let hook = &config.hook;
        let timer = &config.timer;
        let value = match self.selected.index() {
            0 => format!("{}", timer.focus.as_secs() / 60),
            1 => format!("{}", timer.short.as_secs() / 60),
            2 => format!("{}", timer.long.as_secs() / 60),
            3 => format!("{}", timer.long_interval),
            7 => hook.focus.clone(),
            8 => hook.short.clone(),
            9 => hook.long.clone(),
            10 => alarm.focus.path(),
            11 => alarm.short.path(),
            12 => alarm.long.path(),
            13 => alarm.focus.path(),
            14 => alarm.short.path(),
            15 => alarm.long.path(),
            _ => return, // Cannot edit toggles or out of bounds
        };

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

    /// Move selection up
    fn select_up(&mut self) {
        let idx = self
            .selected
            .index()
            .saturating_sub(1)
            .clamp(0, SETTINGS_VIEW_ITEMS - 1); // 13 items total
        self.selected = SettingsItem::from_index(idx).unwrap();
    }

    /// Move selection down
    fn select_down(&mut self) {
        let idx = self
            .selected
            .index()
            .saturating_add(1)
            .clamp(0, SETTINGS_VIEW_ITEMS - 1); // 13 items total
        self.selected = SettingsItem::from_index(idx).unwrap();
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
