pub mod app;
pub mod controller;
pub mod error;
pub mod tui;
pub mod view;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    Timer,
    Settings,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Navigation {
    Quit,
    Stay,
    GoTo(Page),
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Input {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Esc,
    Backspace,
    Char(char),
    Ctrl(char),
    Shift(char),
}

trait FromInput: Sized {
    fn from_input(input: Input) -> Option<Self>;
}
