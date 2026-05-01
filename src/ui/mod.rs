pub mod router;
pub mod traits;
pub mod tui;
pub mod update;

pub use router::Navigation;
pub use router::Page;
pub use router::Router;
pub use traits::*;
pub use update::ConfigCmd;
pub use update::ConfigMsg;
pub use update::PomodoroCmd;
pub use update::PomodoroMsg;

use crate::ui::tui::TuiError;

#[derive(Debug, thiserror::Error)]
pub enum UiError {
    #[error(transparent)]
    TuiError(#[from] TuiError),
}
