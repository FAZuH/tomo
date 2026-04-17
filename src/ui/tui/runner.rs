use std::thread::sleep;
use std::time::Duration;

use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use crossterm::event::{self};
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use ratatui::prelude::*;

use crate::config::Config;
use crate::models::Pomodoro;
use crate::ui::Input;
use crate::ui::Navigation;
use crate::ui::Page;
use crate::ui::app::App;
use crate::ui::app::AppBuildError;
use crate::ui::app::InputMapper;
use crate::ui::tui::TuiError;
use crate::ui::tui::renderer::TuiRenderer;
use crate::ui::tui::view::TuiSettingsView;
use crate::ui::tui::view::TuiTimerView;
use crate::ui::view::SettingsViewActions;
use crate::ui::view::TimerViewActions;

pub struct TuiRunner {
    app: App<Input>,
}

impl TuiRunner {
    pub fn new(config: Config, pomodoro: Pomodoro) -> Result<Self, AppBuildError> {
        let app = App::builder()
            .pomodoro(pomodoro)
            .config(config)
            .timer_view(Box::new(TuiTimerView::new()))
            .settings_view(Box::new(TuiSettingsView::new()))
            .timer_inputmap(Box::new(TimerInputMapper::new()))
            .settings_inputmap(Box::new(SettingsInputMapper))
            .build()?;

        Ok(Self { app })
    }

    pub fn run(&mut self) -> Result<(), TuiError> {
        enable_raw_mode().map_err(TuiError::from)?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(TuiError::from)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout)).map_err(TuiError::from)?;

        let mut renderer = TuiRenderer::new();

        let res = self.run_loop(&mut terminal, &mut renderer);

        // Unconditionally ignore errors for cleanup
        let _ = disable_raw_mode();
        let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
        let _ = terminal.show_cursor();

        res
    }

    fn run_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
        renderer: &mut TuiRenderer,
    ) -> Result<(), TuiError> {
        loop {
            self.app.tick()?;

            let cmds = self.app.render();
            terminal
                .draw(|f| renderer.flush(f, cmds))
                .map_err(TuiError::from)?;

            if event::poll(Duration::from_millis(100)).map_err(TuiError::from)?
                && let Event::Key(key) = event::read().map_err(TuiError::from)?
                && let Some(input) = Input::from_keyevent(key)
            {
                // Handle settings input directly on the renderer
                if matches!(self.app.active_page(), Page::Settings) {
                    self.handle_settings_input(input, renderer)?;
                } else {
                    let nav = self.app.handle(input)?;
                    if matches!(nav, Navigation::Quit) {
                        break;
                    }
                    self.app.navigate(nav);
                }
            }
            sleep(Duration::from_millis(100));
        }
        Ok(())
    }

    /// Handle settings page input directly, mutating renderer state
    fn handle_settings_input(
        &mut self,
        input: Input,
        renderer: &mut TuiRenderer,
    ) -> Result<(), TuiError> {
        use Input::*;

        let settings = &mut renderer.settings;

        // When editing, handle text input
        if settings.is_editing() {
            return self.handle_settings_edit_input(input, settings);
        }

        // When navigating, handle navigation input
        match input {
            Up | Char('k') => {
                settings.select_up();
            }
            Down | Char('j') => {
                settings.select_down();
            }
            Enter => {
                settings.start_editing();
            }
            Esc => {
                self.app
                    .navigate(Navigation::GoTo(Page::Timer));
            }
            Char('q') => {
                return Err(TuiError::from(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "quit",
                )));
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_settings_edit_input(
        &mut self,
        input: Input,
        settings: &mut crate::ui::tui::renderer::TuiSettingsRenderer,
    ) -> Result<(), TuiError> {
        use Input::*;

        match input {
            Esc => {
                settings.cancel_editing();
            }
            Enter => {
                if let Some(action) = commit_settings_edit(settings) {
                    let nav = self.app.handle_settings_action(action)?;
                    if matches!(nav, Navigation::Quit) {
                        return Err(TuiError::from(std::io::Error::new(
                            std::io::ErrorKind::Interrupted,
                            "quit",
                        )));
                    }
                    self.app.navigate(nav);
                }
            }
            Backspace => {
                settings.pop_char();
            }
            Char(c) if c.is_ascii_digit() || c == ':' => {
                settings.push_char(c);
            }
            _ => {}
        }

        Ok(())
    }
}

impl From<&Config> for Pomodoro {
    fn from(value: &Config) -> Self {
        let timer = value.pomodoro.timer.clone();
        Self::new(timer.focus, timer.long, timer.short, timer.long_interval)
    }
}

impl Input {
    fn from_keyevent(key: KeyEvent) -> Option<Input> {
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

impl InputMapper<Input, TimerViewActions> for TimerInputMapper {
    fn into_action(&mut self, input: Input) -> Option<TimerViewActions> {
        use Input::*;
        use TimerViewActions::*;
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

impl InputMapper<Input, SettingsViewActions> for SettingsInputMapper {
    fn into_action(&mut self, _input: Input) -> Option<SettingsViewActions> {
        // Settings input is handled directly in TuiRunner
        // This mapper is only used for actions that bypass navigation (e.g., direct quit)
        None
    }
}

/// Commit the current edit from the settings renderer state to a domain action
fn commit_settings_edit(
    settings: &mut crate::ui::tui::renderer::TuiSettingsRenderer,
) -> Option<SettingsViewActions> {
    use SettingsViewActions::*;

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
