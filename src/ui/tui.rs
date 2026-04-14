pub mod renderer;
pub mod runner;
pub mod view;

pub use runner::TuiRunner;

#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
