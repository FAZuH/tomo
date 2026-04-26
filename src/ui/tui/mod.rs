pub mod input;
pub mod renderer;
pub mod view;

pub use view::TuiView;

#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Error during initialization: {0}")]
    InitializeError(String),
}
