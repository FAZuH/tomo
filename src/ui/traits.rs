use crate::ui::UiError;

pub trait Runner {
    fn run(&mut self) -> Result<(), UiError>;
}

pub trait Updateable {
    type Msg;
    type Cmd;
    fn update(&mut self, msg: Self::Msg) -> Self::Cmd;
}
