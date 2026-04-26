use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use crossterm::event::Event;
use crossterm::event::{self};
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use ratatui::prelude::*;

use crate::config::Config;
use crate::models::Pomodoro;
use crate::ui::Update;
use crate::ui::pages::settings::SettingsMsg;
use crate::ui::pages::settings::SettingsUpdate;
use crate::ui::pages::timer::TimerMsg;
use crate::ui::pages::timer::TimerUpdate;
use crate::ui::router::Navigation;
use crate::ui::router::Page;
use crate::ui::router::Router;
use crate::ui::tui::TuiError;
use crate::ui::tui::input::Input;
use crate::ui::tui::renderer::TuiRenderer;

pub struct TuiView {
    router: Router,
    pomodoro: Pomodoro,
    config: Config,
    should_quit: bool,
    renderer: TuiRenderer,
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    needs_redraw: bool,
}

impl TuiView {
    pub fn new(config: Config, pomodoro: Pomodoro) -> Result<Self, TuiError> {
        let renderer = TuiRenderer::new();
        let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

        Ok(Self {
            router: Router::new(Page::Timer),
            pomodoro,
            config,
            should_quit: false,
            renderer,
            terminal,
            needs_redraw: true,
        })
    }

    pub fn run(&mut self) -> Result<(), TuiError> {
        enable_raw_mode().map_err(TuiError::from)?;
        execute!(std::io::stdout(), EnterAlternateScreen).map_err(TuiError::from)?;

        let res = self.run_loop();

        self.cleanup();
        res
    }

    fn cleanup(&mut self) {
        // Unconditionally ignore errors for cleanup
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }

    fn run_loop(&mut self) -> Result<(), TuiError> {
        let mut last_redraw = Instant::now();
        let redraw_rate = Duration::from_secs(1);

        while !self.should_quit {
            let now = Instant::now();
            if now.duration_since(last_redraw) >= redraw_rate {
                last_redraw = now;
                self.needs_redraw = true;
            }

            if self.needs_redraw {
                self.render_terminal()?;
                self.needs_redraw = false;
            }

            if let Some(input) = Self::get_input()? {
                self.handle_key_event(input)?;
                self.needs_redraw = true;
            }
            sleep(Duration::from_millis(10));
        }
        Ok(())
    }

    fn get_input() -> Result<Option<Input>, TuiError> {
        if event::poll(Duration::from_millis(10))?
            && let Event::Key(key) = event::read()?
        {
            Ok(Input::from_keyevent(key))
        } else {
            Ok(None)
        }
    }

    fn render_terminal(&mut self) -> Result<(), TuiError> {
        self.terminal.draw(|f| {
            self.renderer
                .flush(f, &self.router, &self.pomodoro, &self.config)
        })?;
        Ok(())
    }

    fn handle_key_event(&mut self, input: Input) -> Result<(), TuiError> {
        // Handle settings input directly on the renderer
        match self.router.active_page() {
            Some(Page::Settings) => self.handle_settings(input)?,
            Some(Page::Timer) => self.handle_timer(input)?,
            None => self.should_quit = true,
        }
        Ok(())
    }

    fn handle_timer(&mut self, input: Input) -> Result<(), TuiError> {
        use Input::*;
        use TimerMsg::*;

        match input {
            Left | Char('h') => {
                self.pomodoro =
                    TimerUpdate::update(Subtract(Duration::from_secs(30)), self.pomodoro.clone());
            }
            Down | Char('j') => {
                self.pomodoro =
                    TimerUpdate::update(Subtract(Duration::from_secs(60)), self.pomodoro.clone());
            }
            Right | Char('l') => {
                self.pomodoro =
                    TimerUpdate::update(Add(Duration::from_secs(30)), self.pomodoro.clone());
            }
            Up | Char('k') => {
                self.pomodoro =
                    TimerUpdate::update(Add(Duration::from_secs(60)), self.pomodoro.clone());
            }
            Char(' ') => {
                self.pomodoro = TimerUpdate::update(TogglePause, self.pomodoro.clone());
            }
            Enter => {
                self.pomodoro = TimerUpdate::update(SkipSession, self.pomodoro.clone());
            }
            Backspace => {
                self.pomodoro = TimerUpdate::update(ResetSession, self.pomodoro.clone());
            }
            Char('q') => self.router.navigate(Navigation::Quit),
            Char('s') => self.router.navigate(Navigation::GoTo(Page::Settings)),
            _ => {}
        }
        Ok(())
    }

    /// Handle settings page input directly, mutating renderer state
    fn handle_settings(&mut self, input: Input) -> Result<(), TuiError> {
        let settings = &mut self.renderer.settings;

        // When editing, handle text input
        if settings.is_editing() {
            return self.handle_settings_edit(input);
        }

        // When navigating, handle navigation input
        use Input::*;
        match input {
            Up | Char('k') => settings.select_up(),
            Down | Char('j') => settings.select_down(),
            Enter => {
                if SettingsMsg::is_toggle_index(settings.selected_idx()) {
                    self.save_settings()?
                } else {
                    settings.start_editing()
                }
            }
            Char(' ') if SettingsMsg::is_toggle_index(settings.selected_idx()) => {
                self.save_settings()?
            }
            Esc => self.router.navigate(Navigation::GoTo(Page::Timer)),
            Char('q') => self.quit(),
            _ => {}
        }

        Ok(())
    }

    fn handle_settings_edit(&mut self, input: Input) -> Result<(), TuiError> {
        let settings = &mut self.renderer.settings;

        use Input::*;
        match input {
            Esc => settings.cancel_editing(),
            Enter => self.save_settings()?,
            Backspace => settings.pop_char(),
            Char(c) if c.is_ascii_digit() || c == ':' => {
                settings.push_char(c);
            }
            _ => {}
        }

        Ok(())
    }

    fn save_settings(&mut self) -> Result<(), TuiError> {
        let settings = &mut self.renderer.settings;
        let selected_idx = settings.selected_idx();
        let value = settings.edit_buffer().to_string();
        settings.cancel_editing();

        let msg = match selected_idx {
            // Timer settings (0-6)
            0 => Some(SettingsMsg::TimerFocus(parse_duration_minutes(&value))),
            1 => Some(SettingsMsg::TimerShort(parse_duration_minutes(&value))),
            2 => Some(SettingsMsg::TimerLong(parse_duration_minutes(&value))),
            3 => Some(SettingsMsg::TimerLongInterval(value.parse().unwrap_or(4))),
            4 => Some(SettingsMsg::TimerAutoFocus),
            5 => Some(SettingsMsg::TimerAutoShort),
            6 => Some(SettingsMsg::TimerAutoLong),
            // Hook settings (7-9)
            7 => Some(SettingsMsg::HookFocus(value)),
            8 => Some(SettingsMsg::HookShort(value)),
            9 => Some(SettingsMsg::HookLong(value)),
            // Sound settings (10-12)
            10 => Some(SettingsMsg::SoundFocus(parse_path(&value))),
            11 => Some(SettingsMsg::SoundShort(parse_path(&value))),
            12 => Some(SettingsMsg::SoundLong(parse_path(&value))),
            _ => None,
        };

        if let Some(m) = msg {
            self.config = SettingsUpdate::update(m, self.config.clone());
        }

        Ok(())
    }

    fn quit(&mut self) {
        self.should_quit = true
    }
}

fn parse_duration_minutes(s: &str) -> Duration {
    s.parse::<u64>()
        .map(|m| Duration::from_secs(m * 60))
        .unwrap_or(Duration::from_secs(25 * 60))
}

fn parse_path(s: &str) -> Option<std::path::PathBuf> {
    if s.is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(s))
    }
}
