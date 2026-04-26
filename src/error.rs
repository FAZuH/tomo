use crate::cli::CliArgumentError;
use crate::config::ConfigError;
use crate::ui::UiError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Cli(#[from] CliArgumentError),

    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Ui(#[from] UiError),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

pub type Result<T> = std::result::Result<T, Error>;
