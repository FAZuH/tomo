use crate::ui::app::AppBuildError;
use crate::ui::app::AppError;
use crate::ui::tui::TuiError;

#[derive(Debug, thiserror::Error)]
pub enum UiError {
    #[error(transparent)]
    Tui(#[from] TuiError),

    #[error(transparent)]
    App(#[from] AppError),

    #[error(transparent)]
    Build(#[from] AppBuildError),
}
