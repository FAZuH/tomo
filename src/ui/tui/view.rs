use crate::ui::view::SettingsRenderCommand;
use crate::ui::view::SettingsView;
use crate::ui::view::SettingsViewState;
use crate::ui::view::TimerRenderCommand;
use crate::ui::view::TimerView;
use crate::ui::view::TimerViewState;

pub struct TuiTimerView;

impl TuiTimerView {
    pub fn new() -> Self {
        Self
    }
}

impl TimerView for TuiTimerView {
    fn render(&self, s: TimerViewState) -> Vec<TimerRenderCommand> {
        use TimerRenderCommand::*;

        vec![
            State(s.state),
            Timer {
                remaining: s.remaining,
            },
            PauseIndicator(!s.running),
            Stats {
                remaining: s.remaining,
                total: s.total,
                long_interval: s.long_interval,
                total_sessions: s.total_sessions,
                focus_sessions: s.focus_sessions,
            },
            Progress(s.progress_perc),
        ]
    }
}

#[derive(Default)]
pub struct TuiSettingsView;

impl TuiSettingsView {
    pub fn new() -> Self {
        Self
    }
}

impl SettingsView for TuiSettingsView {
    #[rustfmt::skip]
    fn render(&self, state: SettingsViewState) -> Vec<SettingsRenderCommand> {
        use SettingsRenderCommand as S;

        vec![
            S::Title,
            S::section("󰔛 Pomodoro Timer", vec![
                S::subsection("Time (minutes)", vec![
                    S::input("Focus", state.timer_focus.as_secs() / 60),
                    S::input("Short Break", state.timer_short.as_secs() / 60),
                    S::input("Long Break", state.timer_long.as_secs() / 60),
                ]),
                S::input("Long Break Interval", state.timer_long_interval),
                S::subsection("Auto Start", vec![
                    S::checkbox("Auto Start Focus", state.timer_auto_focus),
                    S::checkbox("Auto Start Short Break", state.timer_auto_short),
                    S::checkbox("Auto Start Long Break", state.timer_auto_long),
                ]),
            ]),
            S::section("󰛢 Command Hooks", vec![
                S::input("Focus Hook", &state.hook_focus),
                S::input("Short Break Hook", &state.hook_short),
                S::input("Long Break Hook", &state.hook_long),
            ]),
            S::section("󰕾 Sounds", vec![
                S::input("Focus Sound", state.sound_focus.as_ref().map(|p| p.display().to_string()).unwrap_or_default()),
                S::input("Short Break Sound", state.sound_short.as_ref().map(|p| p.display().to_string()).unwrap_or_default()),
                S::input("Long Break Sound", state.sound_long.as_ref().map(|p| p.display().to_string()).unwrap_or_default()),
            ]),
        ]
    }
}
