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
use crate::config::Percentage;
use crate::models::Pomodoro;
use crate::models::pomodoro::PomodoroState;
use crate::services::SoundService;
use crate::ui::Update;
use crate::ui::pages::settings::SettingsCmd;
use crate::ui::pages::settings::SettingsMsg;
use crate::ui::pages::settings::SettingsUpdate;
use crate::ui::pages::timer::TimerCmd;
use crate::ui::pages::timer::TimerMsg;
use crate::ui::pages::timer::TimerUpdate;
use crate::ui::router::Navigation;
use crate::ui::router::Page;
use crate::ui::router::Router;
use crate::ui::tui::TuiError;
use crate::ui::tui::input::Input;
use crate::ui::tui::renderer::TuiRenderer;

type Sound = Box<dyn SoundService<SoundType = PomodoroState>>;

pub struct TuiView {
    router: Router,
    pomodoro: Pomodoro,
    config: Config,
    latest_config_save: Config,
    should_quit: bool,
    renderer: TuiRenderer,
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    needs_redraw: bool,
    sound: Sound,
}

impl TuiView {
    pub fn new(config: Config, pomodoro: Pomodoro, sound: Sound) -> Result<Self, TuiError> {
        let renderer = TuiRenderer::new();
        let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

        Ok(Self {
            router: Router::new(Page::Timer),
            pomodoro,
            latest_config_save: config.clone(),
            config,
            should_quit: false,
            renderer,
            terminal,
            needs_redraw: true,
            sound,
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
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(1000);

        while !self.should_quit {
            let now = Instant::now();
            if now.duration_since(last_tick) >= tick_rate {
                last_tick = now;
                self.needs_redraw = true;
            }

            if self.needs_redraw {
                self.tick()?;
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

    fn tick(&mut self) -> Result<(), TuiError> {
        self.sound.set_sound(self.pomodoro.state());
        self.render_terminal()?;
        let (model, cmd) = TimerUpdate::update(
            TimerMsg::Tick {
                auto_next: self.should_auto_next(),
            },
            self.pomodoro.clone(),
        );
        self.pomodoro = model;

        self.handle_timer_cmd(cmd);
        Ok(())
    }

    fn handle_timer_cmd(&mut self, cmd: TimerCmd) {
        match cmd {
            TimerCmd::None => {}
            TimerCmd::PromptNextSession => {
                self.renderer.timer.set_prompt_next_session(true);
                self.play_sound();
            }
            TimerCmd::NextSession => {
                self.next_session();
                self.play_sound();
            }
            TimerCmd::ContinuedSession => {}
        }
    }

    fn play_sound(&mut self) {
        // TODO: Handle errs
        if !self.sound.is_playing() {
            self.sound.play().unwrap();
        }
    }

    fn next_session(&mut self) {
        (self.pomodoro, _) = TimerUpdate::update(TimerMsg::NextState, self.pomodoro.clone());
    }

    fn should_auto_next(&self) -> bool {
        let timer = &self.config.pomodoro.timer;
        match self.pomodoro.state() {
            PomodoroState::Focus => timer.auto_focus,
            PomodoroState::LongBreak => timer.auto_long,
            PomodoroState::ShortBreak => timer.auto_short,
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

        if self.renderer.timer.prompt_next_session() {
            return self.handle_timer_nextstate_prompt(input);
        }

        match input {
            Left | Char('h') => {
                (self.pomodoro, _) =
                    TimerUpdate::update(Subtract(Duration::from_secs(30)), self.pomodoro.clone());
            }
            Down | Char('j') => {
                (self.pomodoro, _) =
                    TimerUpdate::update(Subtract(Duration::from_secs(60)), self.pomodoro.clone());
            }
            Right | Char('l') => {
                (self.pomodoro, _) =
                    TimerUpdate::update(Add(Duration::from_secs(30)), self.pomodoro.clone());
            }
            Up | Char('k') => {
                (self.pomodoro, _) =
                    TimerUpdate::update(Add(Duration::from_secs(60)), self.pomodoro.clone());
            }
            Char(' ') => {
                (self.pomodoro, _) = TimerUpdate::update(TogglePause, self.pomodoro.clone());
            }
            Enter => {
                (self.pomodoro, _) = TimerUpdate::update(SkipSession, self.pomodoro.clone());
            }
            Backspace => {
                (self.pomodoro, _) = TimerUpdate::update(ResetSession, self.pomodoro.clone());
            }
            Char('q') => self.quit(),
            Char('s') => self.router.navigate(Navigation::GoTo(Page::Settings)),
            _ => {}
        }
        Ok(())
    }

    fn handle_timer_nextstate_prompt(&mut self, input: Input) -> Result<(), TuiError> {
        use Input::*;

        match input {
            Enter | Char('y') => {
                self.next_session();
                self.renderer.timer.set_prompt_next_session(false);
            }
            Esc | Char('n') => self.quit(),
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
                    self.update_settings()
                } else {
                    settings.start_editing()
                }
            }
            Char('s') => self.save_settings(),
            Char(' ') if SettingsMsg::is_toggle_index(settings.selected_idx()) => {
                self.update_settings()
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
            Enter => self.update_settings(),
            Backspace => settings.pop_char(),
            Char(c) if c.is_ascii_digit() || c == ':' => {
                settings.push_char(c);
            }
            _ => {}
        }

        Ok(())
    }

    fn update_settings(&mut self) {
        let settings = &mut self.renderer.settings;
        let selected_idx = settings.selected_idx();
        let value = settings.edit_buffer().to_string();
        settings.cancel_editing();

        use SettingsMsg::*;
        let msg = match selected_idx {
            // Timer settings (0-6)
            0 => Some(TimerFocus(parse_duration_minutes(&value))),
            1 => Some(TimerShort(parse_duration_minutes(&value))),
            2 => Some(TimerLong(parse_duration_minutes(&value))),
            3 => Some(TimerLongInterval(value.parse().unwrap_or(4))),
            4 => Some(TimerAutoFocus),
            5 => Some(TimerAutoShort),
            6 => Some(TimerAutoLong),
            // Hook settings (7-9)
            7 => Some(HookFocus(value)),
            8 => Some(HookShort(value)),
            9 => Some(HookLong(value)),
            // Notification path settings (10-12)
            10 => Some(NotificationPathFocus(parse_path(&value))),
            11 => Some(NotificationPathShort(parse_path(&value))),
            12 => Some(NotificationPathLong(parse_path(&value))),
            // Notification volume settings (10-12)
            13 => Some(NotificationVolumeFocus(parse_volume(&value))),
            14 => Some(NotificationVolumeShort(parse_volume(&value))),
            15 => Some(NotificationVolumeLong(parse_volume(&value))),
            _ => return,
        };

        if let Some(m) = msg {
            (self.config, _) = SettingsUpdate::update(m, self.config.clone());
        }

        self.renderer
            .settings
            .set_has_unsaved_changes(self.check_settings_updated());
    }

    fn save_settings(&mut self) {
        let (model, cmd) = SettingsUpdate::update(SettingsMsg::SaveToDisk, self.config.clone());
        self.config = model;

        if let SettingsCmd::SavedToDisk(res) = cmd {
            match res {
                Ok(_) => {
                    self.renderer.settings.set_has_unsaved_changes(false);
                    self.latest_config_save = self.config.clone();
                }
                Err(_) => res.unwrap(),
            }
        }
    }

    // Compare current config with when it was latest saved.
    fn check_settings_updated(&self) -> bool {
        self.config != self.latest_config_save
    }

    fn quit(&mut self) {
        self.router.navigate(Navigation::Quit);
        self.should_quit = true
    }
}

// TODO: show error instead of default
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

fn parse_volume(s: &str) -> Percentage {
    if s.is_empty() {
        Percentage::default()
    } else {
        Percentage::try_from(s).unwrap_or_default()
    }
}
