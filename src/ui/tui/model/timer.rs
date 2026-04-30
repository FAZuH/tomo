use crate::ui::Updateable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerMsg {
    SetPromptNextSession(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerCmd {
    None,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct TimerModel {
    prompt_transition: bool,
}

impl Updateable for TimerModel {
    type Msg = TimerMsg;
    type Cmd = TimerCmd;

    fn update(&mut self, msg: Self::Msg) -> Self::Cmd {
        use TimerMsg::*;
        match msg {
            SetPromptNextSession(v) => self.prompt_transition = v,
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
}
