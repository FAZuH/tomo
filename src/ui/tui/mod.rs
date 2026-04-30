pub mod backend;
pub mod renderer;
pub mod runner;
pub mod toasts;

pub use runner::TuiRunner;

#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("error during initialization: {0}")]
    InitializeError(String),
}
