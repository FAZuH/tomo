use crate::ui::view::HomeRenderCommand;
use crate::ui::view::HomeView;
use crate::ui::view::SettingsRenderCommand;
use crate::ui::view::SettingsView;
use crate::ui::view::SettingsViewState;
use crate::ui::view::TimerRenderCommand;
use crate::ui::view::TimerView;
use crate::ui::view::TimerViewState;

pub struct TuiHomeView;

impl TuiHomeView {
    pub fn new() -> Self {
        Self
    }
}

impl HomeView for TuiHomeView {
    fn render(&self) -> Vec<HomeRenderCommand> {
        vec![HomeRenderCommand::WelcomeText]
    }
}

pub struct TuiTimerView;

impl TuiTimerView {
    pub fn new() -> Self {
        Self
    }
}

impl TimerView for TuiTimerView {
    fn render(&self, state: TimerViewState) -> Vec<TimerRenderCommand> {
        todo!()
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
