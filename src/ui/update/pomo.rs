use std::time::Duration;

use crate::models::Pomodoro;
use crate::ui::prelude::*;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum PomodoroMsg {
    Add(Duration),
    Subtract(Duration),
    TogglePause,
    SkipSession,
    ResetSession,
    Tick { auto_next: bool },
    NextState,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum PomodoroCmd {
    None,
    PromptNextSession,
    NextSession,
    SessionContinued,
}

impl Updateable for Pomodoro {
    type Msg = PomodoroMsg;
    type Cmd = PomodoroCmd;

    fn update(&mut self, msg: Self::Msg) -> Self::Cmd {
        use PomodoroMsg::*;
        let mut cmd = PomodoroCmd::None;
        match msg {
            Add(dur) => self.add(dur),
            Subtract(dur) => self.subtract(dur),
            TogglePause => self.toggle_pause(),
            SkipSession => self.skip(),
            ResetSession => self.reset(),
            NextState => {
                self.skip();
                cmd = PomodoroCmd::SessionContinued;
            }
            Tick { auto_next } => cmd = self.tick(auto_next),
        }
        cmd
    }
}

impl Pomodoro {
    pub fn tick(&mut self, auto_next: bool) -> PomodoroCmd {
        if self.remaining_time().is_zero() {
            if auto_next {
                PomodoroCmd::NextSession
            } else {
                PomodoroCmd::PromptNextSession
            }
        } else {
            PomodoroCmd::None
        }
    }
}
