pub mod input;
pub mod renderer;
pub mod runner;
pub mod view;

pub use runner::TuiRunner;

use crate::ui::app::AppError;

#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    App(#[from] AppError),
}
