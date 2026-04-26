pub mod pages;
pub mod router;
pub mod tui;

pub trait Update {
    type Model;
    type Msg;
    fn update(msg: Self::Msg, model: Self::Model) -> Self::Model;
}

#[derive(Debug, thiserror::Error)]
pub enum UiError {}
