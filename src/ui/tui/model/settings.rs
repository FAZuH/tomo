use tui_widgets::prompts::FocusState;
use tui_widgets::prompts::State;
use tui_widgets::prompts::TextState;
use tui_widgets::scrollview::ScrollViewState;

use crate::config::pomodoro::PomodoroConfig;
use crate::ui::Updateable;
use crate::ui::tui::view::settings::SettingsPrompt;
use crate::ui::update::settings::SETTINGS_VIEW_ITEMS;

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

pub struct SettingsModel {
    selected_idx: u32,
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
            scroll_state: ScrollViewState::default(),
            selected_idx: 0,
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
    pub fn selected_idx(&self) -> u32 {
        self.selected_idx
    }

    /// Check if currently editing
    pub fn is_editing(&self) -> bool {
        self.prompt.is_some()
    }

    pub fn start_editing_for_field(&mut self, config: &PomodoroConfig) {
        let alarm = &config.alarm;
        let hook = &config.hook;
        let timer = &config.timer;
        let (label, value) = match self.selected_idx {
            0 => ("Focus", format!("{}", timer.focus.as_secs() / 60)),
            1 => ("Short Break", format!("{}", timer.short.as_secs() / 60)),
            2 => ("Long Break", format!("{}", timer.long.as_secs() / 60)),
            3 => ("Long Break Interval", format!("{}", timer.long_interval)),
            7 => ("Focus Hook", hook.focus.clone()),
            8 => ("Short Break Hook", hook.short.clone()),
            9 => ("Long Break Hook", hook.long.clone()),
            10 => ("Focus Alarm", alarm.focus.path()),
            11 => ("Short Break Alarm", alarm.short.path()),
            12 => ("Long Break Alarm", alarm.long.path()),
            13 => ("Focus Alarm Volume", alarm.focus.path()),
            14 => ("Short Break Alarm Volume", alarm.short.path()),
            15 => ("Long Break Alarm Volume", alarm.long.path()),
            _ => return, // Cannot edit toggles or out of bounds
        };

        let value_len = value.len();
        let mut text_state = TextState::new()
            .with_focus(FocusState::Focused)
            .with_value(value);
        *State::position_mut(&mut text_state) = value_len;

        self.prompt = Some(SettingsPrompt {
            text_state,
            label: label.to_string(),
        });
    }

    /// Move selection up
    fn select_up(&mut self) {
        self.selected_idx = self
            .selected_idx
            .saturating_sub(1)
            .clamp(0, SETTINGS_VIEW_ITEMS - 1); // 13 items total
    }

    /// Move selection down
    fn select_down(&mut self) {
        self.selected_idx = self
            .selected_idx
            .saturating_add(1)
            .clamp(0, SETTINGS_VIEW_ITEMS - 1);
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
