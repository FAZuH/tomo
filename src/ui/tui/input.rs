use std::time::Duration;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;

use crate::ui::Input;
use crate::ui::Navigation;
use crate::ui::Page;
use crate::ui::tui::renderer::TuiSettingsRenderer;
use crate::ui::view::SettingsActions;
use crate::ui::view::TimerActions;

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

pub struct TimerInputMapper;

impl TimerInputMapper {
    pub fn new() -> Self {
        Self
    }
}

impl InputMapper<Input, TimerActions> for TimerInputMapper {
    fn into_action(&mut self, input: Input) -> Option<TimerActions> {
        use Input::*;
        use TimerActions::*;
        let ret = match input {
            Left => Subtract(Duration::from_secs(30)),
            Down => Subtract(Duration::from_secs(60)),
            Right => Add(Duration::from_secs(30)),
            Up => Add(Duration::from_secs(60)),
            Char(' ') => TogglePause,
            Enter => SkipSession,
            Backspace => ResetSession,
            Char('q') => Navigate(Navigation::Quit),
            Char('s') => Navigate(Navigation::GoTo(Page::Settings)),
            _ => return None,
        };
        Some(ret)
    }
}

/// Platform-specific input mapper for settings (stateless, just maps to domain actions)
pub struct SettingsInputMapper;

impl InputMapper<Input, SettingsActions> for SettingsInputMapper {
    fn into_action(&mut self, _input: Input) -> Option<SettingsActions> {
        // Settings input is handled directly in TuiRunner
        // This mapper is only used for actions that bypass navigation (e.g., direct quit)
        None
    }
}

/// Commit the current edit from the settings renderer state to a domain action
pub fn commit_settings_edit(settings: &mut TuiSettingsRenderer) -> Option<SettingsActions> {
    use SettingsActions::*;

    let selected_idx = settings.selected_idx();
    let value = settings.edit_buffer().to_string();
    settings.cancel_editing();

    let action = match selected_idx {
        // Timer settings (0-6)
        0 => TimerFocus(parse_duration_minutes(&value)),
        1 => TimerShort(parse_duration_minutes(&value)),
        2 => TimerLong(parse_duration_minutes(&value)),
        3 => TimerLongInterval(value.parse().unwrap_or(4)),
        4 => TimerAutoFocus(parse_bool(&value)),
        5 => TimerAutoShort(parse_bool(&value)),
        6 => TimerAutoLong(parse_bool(&value)),
        // Hook settings (7-9)
        7 => HookFocus(value),
        8 => HookShort(value),
        9 => HookLong(value),
        // Sound settings (10-12)
        10 => SoundFocus(parse_path(&value)),
        11 => SoundShort(parse_path(&value)),
        12 => SoundLong(parse_path(&value)),
        _ => return None,
    };

    Some(action)
}

fn parse_duration_minutes(s: &str) -> Duration {
    s.parse::<u64>()
        .map(|m| Duration::from_secs(m * 60))
        .unwrap_or(Duration::from_secs(25 * 60))
}

fn parse_bool(s: &str) -> bool {
    matches!(s.to_lowercase().as_str(), "true" | "yes" | "1" | "y")
}

fn parse_path(s: &str) -> Option<std::path::PathBuf> {
    if s.is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(s))
    }
}

pub trait InputMapper<I, A> {
    #[allow(clippy::wrong_self_convention)]
    fn into_action(&mut self, input: I) -> Option<A>;
}
