use std::time::Duration;

use crate::config::Config;
use crate::models::Pomodoro;
use crate::models::pomodoro::PomodoroError;
use crate::ui::Navigation;
use crate::ui::Page;
use crate::ui::view::HomeRenderCommand;
use crate::ui::view::HomeView;
use crate::ui::view::HomeViewActions;
use crate::ui::view::SettingsRenderCommand;
use crate::ui::view::SettingsView;
use crate::ui::view::SettingsViewState;
use crate::ui::view::TimerRenderCommand;
use crate::ui::view::TimerView;
use crate::ui::view::TimerViewActions;
use crate::ui::view::TimerViewState;

pub struct HomeController {
    view: Box<dyn HomeView>,
}

impl HomeController {
    pub fn new(view: Box<dyn HomeView>) -> Self {
        Self { view }
    }

    pub fn handle(&self, action: HomeViewActions) -> Navigation {
        match action {
            HomeViewActions::GoToTimer => Navigation::GoTo(Page::Timer),
            HomeViewActions::GoToSettings => Navigation::GoTo(Page::Settings),
            HomeViewActions::Quit => Navigation::Quit,
        }
    }

    pub fn render(&self) -> Vec<HomeRenderCommand> {
        self.view.render()
    }
}

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
            Add1Min => self.model.add(Duration::from_mins(1))?,
            Add5Min => self.model.add(Duration::from_mins(5))?,
            Sub1Min => self.model.subtract(Duration::from_mins(1))?,
            Sub5Min => self.model.subtract(Duration::from_mins(5))?,
            TogglePause => self.model.toggle_pause(),
            SkipSession => self.model.skip()?,
            ResetSession => self.model.reset()?,
            GoHome => return Ok(Navigation::GoTo(Page::Home)),
            GoSettings => return Ok(Navigation::GoTo(Page::Settings)),
            Quit => return Ok(Navigation::Quit),
        }
        Ok(Navigation::Stay)
    }

    pub fn tick(&mut self) -> Result<(), PomodoroError> {
        self.model.tick()
    }

    pub fn render(&self) -> Vec<TimerRenderCommand> {
        let state = TimerViewState::from(&self.model);
        self.view.render(state)
    }
}

impl From<&Pomodoro> for TimerViewState {
    fn from(value: &Pomodoro) -> Self {
        Self {
            remaining: value.remaining_time().unwrap_or(value.session_duration()),
            total: value.session_duration(),
            state: value.state(),
            paused: !value.is_running(),
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

    pub fn render(&self) -> Vec<SettingsRenderCommand> {
        let state = SettingsViewState::from(&self.config);
        self.view.render(state)
    }
}

impl From<&Config> for SettingsViewState {
    fn from(value: &Config) -> Self {
        let timer = value.pomodoro.timer.clone();
        Self {
            focus: timer.focus,
            short: timer.short,
            long: timer.long,
            long_interval: timer.long_interval,
        }
    }
}
