use crate::config::Config;
use crate::models::pomodoro::PomodoroError;
use crate::models::Pomodoro;
use crate::ui::view::SettingsRenderCommand;
use crate::ui::view::SettingsView;
use crate::ui::view::SettingsViewActions;
use crate::ui::view::SettingsViewState;
use crate::ui::view::TimerRenderCommand;
use crate::ui::view::TimerView;
use crate::ui::view::TimerViewActions;
use crate::ui::view::TimerViewState;
use crate::ui::Navigation;

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
            Add(dur) => self.model.add(dur),
            Subtract(dur) => self.model.subtract(dur),
            TogglePause => self.model.toggle_pause(),
            SkipSession => self.model.skip(),
            ResetSession => self.model.reset(),
            Navigate(nav) => return Ok(nav),
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
}

impl SettingsController {
    pub fn new(view: Box<dyn SettingsView>, config: Config) -> Self {
        Self { view, config }
    }

    pub fn handle(&mut self, action: SettingsViewActions) -> Result<Navigation, PomodoroError> {
        use SettingsViewActions::*;
        match action {
            // Timer settings
            TimerFocus(d) => self.config.pomodoro.timer.focus = d,
            TimerShort(d) => self.config.pomodoro.timer.short = d,
            TimerLong(d) => self.config.pomodoro.timer.long = d,
            TimerLongInterval(n) => self.config.pomodoro.timer.long_interval = n,
            TimerAutoFocus(v) => self.config.pomodoro.timer.auto_focus = v,
            TimerAutoShort(v) => self.config.pomodoro.timer.auto_short = v,
            TimerAutoLong(v) => self.config.pomodoro.timer.auto_long = v,
            // Hook settings
            HookFocus(s) => self.config.pomodoro.hook.focus = s,
            HookShort(s) => self.config.pomodoro.hook.short = s,
            HookLong(s) => self.config.pomodoro.hook.long = s,
            // Sound settings
            SoundFocus(p) => self.config.pomodoro.sound.focus = p,
            SoundShort(p) => self.config.pomodoro.sound.short = p,
            SoundLong(p) => self.config.pomodoro.sound.long = p,
            Navigate(nav) => return Ok(nav),
        }
        Ok(Navigation::Stay)
    }

    pub fn render(&self) -> Vec<SettingsRenderCommand> {
        let timer = self.config.pomodoro.timer.clone();
        let hook = self.config.pomodoro.hook.clone();
        let sound = self.config.pomodoro.sound.clone();
        let state = SettingsViewState {
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
        };
        self.view.render(state)
    }
}
