use notify_rust::Notification;
use notify_rust::{self};

use crate::models::pomodoro::State;

impl From<State> for Notification {
    fn from(value: State) -> Self {
        let (summary, body) = match value {
            State::Focus => ("Time to focus!", "Your break is over. Get back to work."),
            State::LongBreak => ("Long break time!", "You've earned it. Take a long break."),
            State::ShortBreak => ("Short break time!", "Take a quick breather."),
        };

        let mut ret = Notification::new();
        ret.summary(summary).body(body);
        ret
    }
}

pub fn notify(notifiable: impl Into<Notification>) -> Result<(), notify_rust::error::Error> {
    notifiable.into().show()?;
    Ok(())
}
