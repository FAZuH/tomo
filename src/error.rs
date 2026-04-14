use crate::cli::CliArgumentError;
use crate::config::ConfigError;
use crate::models::pomodoro::PomodoroError;
use crate::ui::app::AppBuildError;
use crate::ui::error::UiError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Cli(#[from] CliArgumentError),

    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Ui(#[from] UiError),

    #[error(transparent)]
    Pomodoro(#[from] PomodoroError),

    #[error(transparent)]
    Build(#[from] AppBuildError),
}

pub type Result<T> = std::result::Result<T, Error>;
