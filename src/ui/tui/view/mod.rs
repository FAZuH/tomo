pub mod settings;
pub mod timer;

use ratatui::prelude::*;
pub use settings::SettingsState;
pub use settings::TuiSettingsView;
pub use timer::TimerState;
pub use timer::TuiTimerView;

use crate::config::Config;
use crate::models::Pomodoro;
use crate::ui::ConfigCmd;
use crate::ui::ConfigMsg;
use crate::ui::PomodoroCmd;
use crate::ui::PomodoroMsg;
use crate::ui::StatefulViewRef;
use crate::ui::Updateable as _;
use crate::ui::router::Page;
use crate::ui::router::Router;
use crate::ui::tui::model::SettingsCmd;
use crate::ui::tui::model::SettingsModel;
use crate::ui::tui::model::SettingsMsg;
use crate::ui::tui::model::TimerCmd;
use crate::ui::tui::model::TimerModel;
use crate::ui::tui::model::TimerMsg;

pub type Canvas<'a, 'b> = &'a mut Frame<'b>;
type State = TuiState;

pub struct TuiView {
    timer: TuiTimerView,
    settings: TuiSettingsView,
}

impl TuiView {
    pub fn new() -> Self {
        Self {
            timer: TuiTimerView::new(),
            settings: TuiSettingsView::new(),
        }
    }
}
impl<'a> StatefulViewRef<Canvas<'a, '_>> for TuiView {
    type State = State;
    type Result = ();

    fn render_stateful_ref(&self, canvas: Canvas<'a, '_>, state: &mut State) {
        let area = canvas.area();
        let buf = canvas.buffer_mut();
        match state.router.active_page() {
            Some(Page::Timer) => self
                .timer
                .render_stateful_ref((area, buf), &mut state.timer),
            Some(Page::Settings) => self
                .settings
                .render_stateful_ref(canvas, &mut state.settings),
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
