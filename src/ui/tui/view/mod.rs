pub mod settings;
pub mod timer;

use ratatui::prelude::*;
pub use settings::SettingsState;
pub use settings::TuiSettingsRenderer;
pub use timer::TimerState;
pub use timer::TuiTimerRenderer;

use crate::config::Config;
use crate::models::Pomodoro;
use crate::ui::ConfigCmd;
use crate::ui::ConfigMsg;
use crate::ui::PomodoroCmd;
use crate::ui::PomodoroMsg;
use crate::ui::Updateable as _;
use crate::ui::router::Page;
use crate::ui::router::Router;
use crate::ui::tui::model::SettingsCmd;
use crate::ui::tui::model::SettingsModel;
use crate::ui::tui::model::SettingsMsg;
use crate::ui::tui::model::TimerCmd;
use crate::ui::tui::model::TimerModel;
use crate::ui::tui::model::TimerMsg;

pub struct TuiRenderer {}

impl TuiRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl StatefulWidget for TuiRenderer {
    type State = TuiState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        match state.router.active_page() {
            Some(Page::Timer) => TuiTimerRenderer::new().render(area, buf, &mut state.timer),
            Some(Page::Settings) => {
                TuiSettingsRenderer::new().render(area, buf, &mut state.settings)
            }
            None => {}
        }
    }
}

pub struct TuiState {
    router: Router,
    timer: TimerState,
    settings: SettingsState,
}

impl TuiState {
    pub fn new(router: Router, timer: TimerState, settings: SettingsState) -> Self {
        Self {
            router,
            timer,
            settings,
        }
    }
}

impl TuiState {
    // Router
    pub fn router(&self) -> &Router {
        &self.router
    }
    pub fn router_mut(&mut self) -> &mut Router {
        &mut self.router
    }
    // pub fn update_router(&mut self, msg: RouterMsg) -> RouterCmd { self.router.update(msg) }

    // Timer
    pub fn timer(&self) -> &TimerModel {
        &self.timer.model
    }
    pub fn timer_mut(&mut self) -> &mut TimerModel {
        &mut self.timer.model
    }
    pub fn update_timer(&mut self, msg: TimerMsg) -> TimerCmd {
        self.timer.model.update(msg)
    }

    pub fn pomo(&self) -> &Pomodoro {
        &self.timer.pomo
    }
    pub fn pomo_mut(&mut self) -> &mut Pomodoro {
        &mut self.timer.pomo
    }
    pub fn update_pomo(&mut self, msg: PomodoroMsg) -> PomodoroCmd {
        self.timer.pomo.update(msg)
    }

    // Settings
    pub fn settings(&self) -> &SettingsModel {
        &self.settings.model
    }
    pub fn settings_mut(&mut self) -> &mut SettingsModel {
        &mut self.settings.model
    }
    pub fn update_settings(&mut self, msg: SettingsMsg) -> SettingsCmd {
        self.settings.model.update(msg)
    }

    pub fn conf(&self) -> &Config {
        &self.settings.conf
    }
    pub fn conf_mut(&mut self) -> &mut Config {
        &mut self.settings.conf
    }
    pub fn update_conf(&mut self, msg: ConfigMsg) -> ConfigCmd {
        self.settings.conf.update(msg)
    }
}
