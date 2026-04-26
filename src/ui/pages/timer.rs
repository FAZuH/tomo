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

    fn update(msg: Self::Msg, mut model: Self::Model) -> Self::Model {
        use TimerMsg::*;
        match msg {
            Add(dur) => model.add(dur),
            Subtract(dur) => model.subtract(dur),
            TogglePause => model.toggle_pause(),
            SkipSession => model.skip(),
            ResetSession => model.reset(),
        }
        model
    }
}
