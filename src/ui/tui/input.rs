use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;

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

impl Input {
    pub fn from_keyevent(key: KeyEvent) -> Option<Input> {
        use Input as I;
        use KeyCode as K;

        let ret = match key.code {
            K::Up => I::Up,
            K::Down => I::Down,
            K::Left => I::Left,
            K::Right => I::Right,
            K::Enter => I::Enter,
            K::Esc => I::Esc,
            K::Backspace => I::Backspace,
            K::Char(char) => {
                if key.modifiers == KeyModifiers::CONTROL {
                    I::Ctrl(char)
                } else if key.modifiers == KeyModifiers::SHIFT {
                    I::Shift(char)
                } else {
                    I::Char(char)
                }
            }
            _ => return None,
        };
        Some(ret)
    }
}
