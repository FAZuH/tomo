pub mod pages;
pub mod router;
pub mod tui;

pub trait Update {
    type Model;
    type Msg;
    type Cmd;
    fn update(msg: Self::Msg, model: Self::Model) -> (Self::Model, Self::Cmd);
}

#[derive(Debug, thiserror::Error)]
pub enum UiError {}
