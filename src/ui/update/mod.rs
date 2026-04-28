pub mod settings;
pub mod timer;

pub trait Update {
    type Model;
    type Msg;
    type Cmd;
    fn update(msg: Self::Msg, model: Self::Model) -> (Self::Model, Self::Cmd);
}
