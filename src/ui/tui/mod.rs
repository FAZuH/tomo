pub mod backend;
pub mod renderer;
pub mod toasts;
pub mod view;

pub use view::TuiView;

#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("error during initialization: {0}")]
    InitializeError(String),
}
