pub mod settings;
pub mod timer;

use ratatui::prelude::*;
pub use settings::SettingsState;
pub use settings::TuiSettingsView;
pub use timer::TimerState;
pub use timer::TuiTimerView;

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
        }
    }
}
