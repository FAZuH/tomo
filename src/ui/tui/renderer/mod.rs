pub mod settings;
pub mod timer;

use ratatui::prelude::*;

use crate::config::Config;
use crate::models::pomodoro::Pomodoro;
use crate::ui::router::Page;
use crate::ui::router::Router;
use crate::ui::tui::renderer::settings::TuiSettingsRenderer;
use crate::ui::tui::renderer::timer::TuiTimerRenderer;

pub struct TuiRenderer {
    pub timer: TuiTimerRenderer,
    pub settings: TuiSettingsRenderer,
}

impl TuiRenderer {
    pub fn new() -> Self {
        Self {
            timer: TuiTimerRenderer::new(),
            settings: TuiSettingsRenderer::new(),
        }
    }

    pub fn flush(
        &mut self,
        frame: &mut Frame,
        router: &Router,
        pomodoro: &Pomodoro,
        config: &Config,
    ) {
        match router.active_page() {
            Some(Page::Timer) => self.timer.render(frame, pomodoro),
            Some(Page::Settings) => self.settings.render(frame, config),
            None => {}
        }
    }
}
