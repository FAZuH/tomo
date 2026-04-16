use std::time::Duration;

use crate::config::Config;
use crate::models::Pomodoro;
use crate::models::pomodoro::PomodoroError;
use crate::ui::Navigation;
use crate::ui::Page;
use crate::ui::view::SettingsRenderCommand;
use crate::ui::view::SettingsView;
use crate::ui::view::SettingsViewActions;
use crate::ui::view::SettingsViewState;
use crate::ui::view::TimerRenderCommand;
use crate::ui::view::TimerView;
use crate::ui::view::TimerViewActions;
use crate::ui::view::TimerViewState;

pub struct TimerController {
    view: Box<dyn TimerView>,
    model: Pomodoro,
}

impl TimerController {
    pub fn new(view: Box<dyn TimerView>, model: Pomodoro) -> Self {
        Self { view, model }
    }

    pub fn handle(&mut self, action: TimerViewActions) -> Result<Navigation, PomodoroError> {
        use TimerViewActions::*;
        match action {
            Add30Sec => self.model.add(Duration::from_secs(30)),
            Add1Min => self.model.add(Duration::from_mins(1)),
            Sub30Sec => self.model.subtract(Duration::from_secs(30)),
            Sub1Min => self.model.subtract(Duration::from_mins(1)),
            TogglePause => self.model.toggle_pause(),
            SkipSession => self.model.skip(),
            ResetSession => self.model.reset(),
            GoSettings => return Ok(Navigation::GoTo(Page::Settings)),
            Quit => return Ok(Navigation::Quit),
        }
        Ok(Navigation::Stay)
    }

    pub fn tick(&mut self) {
        self.model.update();
    }

    pub fn render(&self) -> Vec<TimerRenderCommand> {
        let state = TimerViewState::from(&self.model);
        self.view.render(state)
    }
}

impl From<&Pomodoro> for TimerViewState {
    fn from(v: &Pomodoro) -> Self {
        let remaining = v.remaining_time();
        let progress_perc = 1.0 - (remaining.as_secs_f64() / v.session_duration().as_secs_f64());
        Self {
            remaining,
            total: v.session_duration(),
            state: v.state(),
            running: v.is_running(),
            long_interval: v.long_interval(),
            total_sessions: v.total_sessions(),
            focus_sessions: v.focus_sessions(),
            progress_perc,
        }
    }
}

pub struct SettingsController {
    view: Box<dyn SettingsView>,
    config: Config,
    curr_selection_idx: u32,
}

impl SettingsController {
    pub fn new(view: Box<dyn SettingsView>, config: Config) -> Self {
        Self {
            view,
            config,
            curr_selection_idx: 0,
        }
    }

    pub fn handle(&mut self, action: SettingsViewActions) -> Result<Navigation, PomodoroError> {
        use SettingsViewActions::*;
        match action {
            SelectDown => self.curr_selection_idx -= 1,
            SelectUp => self.curr_selection_idx += 1,
            EditSelection => todo!(),
            Navigate(nav) => return Ok(nav),
        }
        Ok(Navigation::Stay)
    }

    pub fn render(&self) -> Vec<SettingsRenderCommand> {
        let state = SettingsViewState::from(&self.config);
        self.view.render(state)
    }
}

impl From<&Config> for SettingsViewState {
    fn from(value: &Config) -> Self {
        let timer = value.pomodoro.timer.clone();
        let hook = value.pomodoro.hook.clone();
        let sound = value.pomodoro.sound.clone();
        Self {
            timer_focus: timer.focus,
            timer_short: timer.short,
            timer_long: timer.long,
            timer_long_interval: timer.long_interval,
            timer_auto_focus: timer.auto_focus,
            timer_auto_short: timer.auto_short,
            timer_auto_long: timer.auto_long,
            hook_focus: hook.focus,
            hook_short: hook.short,
            hook_long: hook.long,
            sound_focus: sound.focus,
            sound_short: sound.short,
            sound_long: sound.long,
        }
    }
}
