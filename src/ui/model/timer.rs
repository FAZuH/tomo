use crate::ui::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerMsg {
    SetPromptNextSession(bool),
    SetShowKeybinds(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerCmd {
    None,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct TimerModel {
    prompt_transition: bool,
    show_keybinds: bool,
}

impl Updateable for TimerModel {
    type Msg = TimerMsg;
    type Cmd = TimerCmd;

    fn update(&mut self, msg: Self::Msg) -> Self::Cmd {
        use TimerMsg::*;
        match msg {
            SetPromptNextSession(v) => self.prompt_transition = v,
            SetShowKeybinds(v) => self.show_keybinds = v,
        }
        TimerCmd::None
    }
}

impl TimerModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prompt_transition(&self) -> bool {
        self.prompt_transition
    }

    pub fn show_keybinds(&self) -> bool {
        self.show_keybinds
    }

    pub fn toggle_keybinds(&mut self) {
        let new = !self.show_keybinds;
        self.update(TimerMsg::SetShowKeybinds(new));
    }
}
