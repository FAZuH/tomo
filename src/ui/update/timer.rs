use std::time::Duration;

use crate::models::Pomodoro;
use crate::ui::Update;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum TimerMsg {
    Add(Duration),
    Subtract(Duration),
    TogglePause,
    SkipSession,
    ResetSession,
    Tick { auto_next: bool },
    NextState,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum TimerCmd {
    None,
    PromptNextSession,
    NextSession,
    SessionContinued,
}

pub struct TimerUpdate {}

impl TimerUpdate {
    pub fn new() -> Self {
        Self {}
    }

    pub fn tick(auto_next: bool, model: &Pomodoro) -> TimerCmd {
        if model.remaining_time().is_zero() {
            if auto_next {
                TimerCmd::NextSession
            } else {
                TimerCmd::PromptNextSession
            }
        } else {
            TimerCmd::None
        }
    }
}

impl Update for TimerUpdate {
    type Msg = TimerMsg;
    type Model = Pomodoro;
    type Cmd = TimerCmd;

    fn update(msg: Self::Msg, model: &mut Self::Model) -> Self::Cmd {
        use TimerMsg::*;
        let mut cmd = TimerCmd::None;
        match msg {
            Add(dur) => model.add(dur),
            Subtract(dur) => model.subtract(dur),
            TogglePause => model.toggle_pause(),
            SkipSession => model.skip(),
            ResetSession => model.reset(),
            NextState => {
                model.skip();
                cmd = TimerCmd::SessionContinued;
            }
            Tick { auto_next } => cmd = Self::tick(auto_next, model),
        }
        cmd
    }
}
