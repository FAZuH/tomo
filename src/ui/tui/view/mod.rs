pub mod settings;
pub mod timer;

use ratatui::prelude::*;
pub use settings::SettingsState;
pub use settings::TuiSettingsView;
pub use timer::TimerState;
pub use timer::TuiTimerView;

use crate::config::Config;
use crate::ui::StatefulViewRef;
use crate::ui::router::Page;
use crate::ui::router::Router;
use crate::ui::tui::toasts::ToastHandler;

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
    pub router: Router,
    pub timer: TimerState,
    pub settings: SettingsState,
    pub toast: ToastHandler,
    pub latest_config_save: Option<Config>,
}

impl TuiState {
    pub fn new(
        router: Router,
        timer: TimerState,
        settings: SettingsState,
        toast: ToastHandler,
    ) -> Self {
        Self {
            router,
            timer,
            settings,
            toast,
            latest_config_save: None,
        }
    }

    /// Snapshot current settings.
    ///
    /// Use with [`Self::check_settings_updated`]
    pub fn snapshot_settings(&mut self) {
        self.latest_config_save = Some(self.conf().clone())
    }

    /// Compare current config with when it was latest saved.
    pub fn check_settings_unsaved(&self) -> bool {
        if let Some(last) = &self.latest_config_save {
            return *self.conf() != *last;
        }
        true
    }
}
