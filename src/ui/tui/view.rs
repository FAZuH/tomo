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
        let mut ret = Vec::new();

        ret.push(State(s.state));
        ret.push(Timer {
            remaining: s.remaining,
        });
        ret.push(PauseIndicator(!s.running));
        ret.push(Stats {
            remaining: s.remaining,
            total: s.total,
            long_interval: s.long_interval,
            total_sessions: s.total_sessions,
            focus_sessions: s.focus_sessions,
        });
        ret.push(Progress(s.progress_perc));

        ret
    }
}

pub struct TuiSettingsView;

impl TuiSettingsView {
    pub fn new() -> Self {
        Self
    }
}

impl SettingsView for TuiSettingsView {
    fn render(&self, state: SettingsViewState) -> Vec<SettingsRenderCommand> {
        todo!()
    }
}
