use notify_rust::Notification;

use crate::models::pomodoro::PomodoroState;

pub fn notify_pomodoro(state: PomodoroState) {
    let (summary, body) = match state {
        PomodoroState::Focus => ("Time to focus!", "Your break is over. Get back to work."),
        PomodoroState::LongBreak => ("Long break time!", "You've earned it. Take a long break."),
        PomodoroState::ShortBreak => ("Short break time!", "Take a quick breather."),
    };

    if let Err(e) = Notification::new().summary(summary).body(body).show() {
        log::error!("{e}")
    }
}
