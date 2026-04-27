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
}

pub struct TimerUpdate {}

impl TimerUpdate {
    pub fn new() -> Self {
        Self {}
    }
}

impl Update for TimerUpdate {
    type Msg = TimerMsg;
    type Model = Pomodoro;
    type Cmd = TimerCmd;

    fn update(msg: Self::Msg, mut model: Self::Model) -> (Self::Model, Self::Cmd) {
        use TimerMsg::*;
        let mut cmd = TimerCmd::None;
        match msg {
            Add(dur) => model.add(dur),
            Subtract(dur) => model.subtract(dur),
            TogglePause => model.toggle_pause(),
            SkipSession => model.skip(),
            ResetSession => model.reset(),
            NextState => model.skip(),
            Tick { auto_next } => {
                if model.remaining_time().is_zero() {
                    if auto_next {
                        return Self::update(NextState, model);
                    } else {
                        cmd = TimerCmd::PromptNextSession;
                    }
                }
            }
        }
        (model, cmd)
    }
}
