pub mod router;
pub mod tui;
pub mod update;

pub use update::Update;

#[derive(Debug, thiserror::Error)]
pub enum UiError {
    #[error(transparent)]
    TuiError(#[from] TuiError),
}

use crate::config::Config;
use crate::models::Pomodoro;
use crate::ui::tui::TuiError;

pub struct AppModel {
    pub timer: Pomodoro,
    pub settings: Config,
}

pub trait View {
    type Model;

    fn run(&mut self, model: Self::Model) -> Result<(), UiError>;
}
