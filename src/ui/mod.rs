pub mod router;
pub mod traits;
pub mod tui;
pub mod update;

pub use router::Navigation;
pub use router::Page;
pub use router::Router;
pub use traits::*;
pub use update::SettingsCmd;
pub use update::SettingsMsg;
pub use update::SettingsUpdate;
pub use update::TimerCmd;
pub use update::TimerMsg;
pub use update::TimerUpdate;
pub use update::Update;

use crate::config::Config;
use crate::models::Pomodoro;
use crate::ui::tui::TuiError;

#[derive(Debug, thiserror::Error)]
pub enum UiError {
    #[error(transparent)]
    TuiError(#[from] TuiError),
}

pub struct AppModel {
    pub timer: Pomodoro,
    pub settings: Config,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AppMsg {
    Timer(TimerMsg),
    Settings(SettingsMsg),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppCmd {
    None,
}

impl Updateable for AppModel {
    type Msg = AppMsg;
    type Cmd = AppCmd;

    fn update(&mut self, msg: Self::Msg) -> Self::Cmd {
        match msg {
            AppMsg::Timer(_timer_msg) => todo!(),
            AppMsg::Settings(_settings_msg) => todo!(),
        }
    }
}
