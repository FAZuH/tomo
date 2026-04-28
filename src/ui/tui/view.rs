use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use crossterm::event::Event;
use crossterm::event::{self};
use log::error;

use crate::config::Config;
use crate::config::Percentage;
use crate::config::pomodoro::Timers;
use crate::models::pomodoro::State;
use crate::services::SoundService;
use crate::services::cmd_runner::run_cmds;
use crate::services::notify::notify;
use crate::ui::AppModel;
use crate::ui::UiError;
use crate::ui::Update;
use crate::ui::View;
use crate::ui::router::Navigation;
use crate::ui::router::Page;
use crate::ui::router::Router;
use crate::ui::tui::TuiError;
use crate::ui::tui::backend::Tui;
use crate::ui::tui::input::Input;
use crate::ui::tui::renderer::TuiRenderer;
use crate::ui::update::settings::SettingsMsg;
use crate::ui::update::settings::SettingsUpdate;
use crate::ui::update::timer::TimerCmd;
use crate::ui::update::timer::TimerMsg;
use crate::ui::update::timer::TimerUpdate;

type Sound = Box<dyn SoundService<SoundType = State>>;

pub struct TuiView {
    router: Router,
    latest_config_save: Option<Config>,
    should_quit: bool,
    renderer: TuiRenderer,
    terminal: Tui,
    needs_redraw: bool,
    sound: Sound,
}

impl View for TuiView {
    type Model = AppModel;

    fn run(&mut self, model: Self::Model) -> Result<(), UiError> {
        Ok(self.run_loop(model)?)
    }
}

impl TuiView {
    pub fn new(sound: Sound) -> Result<Self, TuiError> {
        let renderer = TuiRenderer::new();
        let terminal = Tui::new()?;

        Ok(Self {
            router: Router::new(Page::Timer),
            latest_config_save: None,
            should_quit: false,
            renderer,
            terminal,
            needs_redraw: true,
            sound,
        })
    }

    fn run_loop(&mut self, mut model: AppModel) -> Result<(), TuiError> {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(1000);

        while !self.should_quit {
            let now = Instant::now();
            if now.duration_since(last_tick) >= tick_rate {
                last_tick = now;
                self.needs_redraw = true;
            }

            if self.needs_redraw {
                model = self.tick(model)?;
                self.needs_redraw = false;
            }

            if let Some(input) = Self::get_input()? {
                model = self.handle_key_event(input, model)?;
                self.needs_redraw = true;
            }
            sleep(Duration::from_millis(10));
        }
        Ok(())
    }

    fn get_input() -> Result<Option<Input>, TuiError> {
        if event::poll(Duration::from_millis(10))? {
            match event::read()? {
                Event::Key(key) => Ok(Input::from_keyevent(key)),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    fn tick(&mut self, mut model: AppModel) -> Result<AppModel, TuiError> {
        self.render_terminal(&model)?;
        let (timer, cmd) = TimerUpdate::update(
            TimerMsg::Tick {
                auto_next: self
                    .should_auto_next(model.timer.state(), &model.settings.pomodoro.timer),
            },
            model.timer,
        );
        model.timer = timer;

        model = self.handle_timer_cmd(cmd, model);
        Ok(model)
    }

    fn handle_timer_cmd(&mut self, cmd: TimerCmd, mut model: AppModel) -> AppModel {
        match cmd {
            TimerCmd::None => {}
            TimerCmd::PromptNextSession => {
                if !self.renderer.timer.prompt_next_session() {
                    // only runs on once per session
                    model = self.on_session_end(model);
                }
                self.renderer.timer.set_prompt_next_session(true);
            }
            TimerCmd::NextSession => {
                model = self.on_session_end(model);
                model = self.next_session(model);
            }
            TimerCmd::SessionContinued => {}
        }
        model
    }

    fn on_session_end(&mut self, mut model: AppModel) -> AppModel {
        // TODO: Handle errs
        model = self.run_hooks(model);
        self.play_sound(model.timer.next_state());
        self.send_notification(model.timer.next_state());
        model
    }

    fn run_hooks(&self, model: AppModel) -> AppModel {
        run_cmds(&model.settings.pomodoro.hook, model.timer.state());
        model
    }

    fn send_notification(&self, next_state: State) {
        notify(next_state).unwrap();
    }

    fn play_sound(&mut self, next_state: State) {
        if !self.sound.is_playing() {
            self.sound.set_sound(next_state);
            if let Err(e) = self.sound.play() {
                error!("{e}")
            }
        }
    }

    fn next_session(&mut self, mut model: AppModel) -> AppModel {
        (model.timer, _) = TimerUpdate::update(TimerMsg::NextState, model.timer.clone());
        model
    }

    fn should_auto_next(&self, curr_state: State, timer: &Timers) -> bool {
        match curr_state {
            State::Focus => timer.auto_focus,
            State::LongBreak => timer.auto_long,
            State::ShortBreak => timer.auto_short,
        }
    }

    fn render_terminal(&mut self, model: &AppModel) -> Result<(), TuiError> {
        self.terminal.draw(|f| {
            self.renderer
                .flush(f, &self.router, &model.timer, &model.settings)
        })?;
        Ok(())
    }

    fn handle_key_event(
        &mut self,
        input: Input,
        mut model: AppModel,
    ) -> Result<AppModel, TuiError> {
        // Handle settings input directly on the renderer
        match self.router.active_page() {
            Some(Page::Settings) => model = self.handle_settings(input, model)?,
            Some(Page::Timer) => model = self.handle_timer(input, model)?,
            None => self.should_quit = true,
        }
        Ok(model)
    }

    fn handle_timer(&mut self, input: Input, mut model: AppModel) -> Result<AppModel, TuiError> {
        use Input::*;
        use TimerMsg::*;

        if self.renderer.timer.prompt_next_session() {
            return self.handle_timer_nextstate_prompt(input, model);
        }

        match input {
            Left | Char('h') => {
                (model.timer, _) =
                    TimerUpdate::update(Subtract(Duration::from_secs(30)), model.timer.clone());
            }
            Down | Char('j') => {
                (model.timer, _) =
                    TimerUpdate::update(Subtract(Duration::from_secs(60)), model.timer.clone());
            }
            Right | Char('l') => {
                (model.timer, _) =
                    TimerUpdate::update(Add(Duration::from_secs(30)), model.timer.clone());
            }
            Up | Char('k') => {
                (model.timer, _) =
                    TimerUpdate::update(Add(Duration::from_secs(60)), model.timer.clone());
            }
            Char(' ') => {
                (model.timer, _) = TimerUpdate::update(TogglePause, model.timer.clone());
            }
            Enter => {
                (model.timer, _) = TimerUpdate::update(SkipSession, model.timer.clone());
            }
            Backspace => {
                (model.timer, _) = TimerUpdate::update(ResetSession, model.timer.clone());
            }
            Char('q') => self.quit(),
            Char('s') => self.router.navigate(Navigation::GoTo(Page::Settings)),
            _ => {}
        }
        Ok(model)
    }

    fn handle_timer_nextstate_prompt(
        &mut self,
        input: Input,
        mut model: AppModel,
    ) -> Result<AppModel, TuiError> {
        use Input::*;

        match input {
            Enter | Char('y') => {
                model = self.next_session(model);
                self.renderer.timer.set_prompt_next_session(false);
            }
            Esc | Char('n') => self.quit(),
            _ => {}
        }

        Ok(model)
    }

    /// Handle settings page input directly, mutating renderer state
    fn handle_settings(&mut self, input: Input, mut model: AppModel) -> Result<AppModel, TuiError> {
        let settings = &mut self.renderer.settings;

        // When editing, handle text input
        if settings.is_editing() {
            return self.handle_settings_edit(input, model);
        }

        // When navigating, handle navigation input
        use Input::*;
        match input {
            Up | Char('k') => settings.select_up(),
            Down | Char('j') => settings.select_down(),
            Enter => {
                if SettingsMsg::is_toggle_index(settings.selected_idx()) {
                    model = self.update_settings(model)
                } else {
                    settings.start_editing()
                }
            }
            Char('s') => self.save_settings(&model),
            Char(' ') if SettingsMsg::is_toggle_index(settings.selected_idx()) => {
                model = self.update_settings(model)
            }
            Esc => self.router.navigate(Navigation::GoTo(Page::Timer)),
            Char('q') => self.quit(),
            _ => {}
        }

        Ok(model)
    }

    fn handle_settings_edit(
        &mut self,
        input: Input,
        mut model: AppModel,
    ) -> Result<AppModel, TuiError> {
        let settings = &mut self.renderer.settings;

        use Input::*;
        match input {
            Esc => settings.cancel_editing(),
            Enter => model = self.update_settings(model),
            Backspace => settings.pop_char(),
            Char(c) if c.is_ascii_digit() || c == ':' => {
                settings.push_char(c);
            }
            _ => {}
        }

        Ok(model)
    }

    fn update_settings(&mut self, mut model: AppModel) -> AppModel {
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
            // Alarm path settings (10-12)
            10 => Some(AlarmPathFocus(parse_path(&value))),
            11 => Some(AlarmPathShort(parse_path(&value))),
            12 => Some(AlarmPathLong(parse_path(&value))),
            // Alarm volume settings (10-12)
            13 => Some(AlarmVolumeFocus(parse_volume(&value))),
            14 => Some(AlarmVolumeShort(parse_volume(&value))),
            15 => Some(AlarmVolumeLong(parse_volume(&value))),
            _ => return model,
        };

        if let Some(m) = msg {
            (model.settings, _) = SettingsUpdate::update(m, model.settings.clone());
        }

        self.renderer
            .settings
            .set_has_unsaved_changes(self.check_settings_updated(&model));
        model
    }

    fn save_settings(&mut self, model: &AppModel) {
        model.settings.save().unwrap();
        self.renderer.settings.set_has_unsaved_changes(false);
        self.latest_config_save = Some(model.settings.clone());
    }

    // Compare current config with when it was latest saved.
    fn check_settings_updated(&self, model: &AppModel) -> bool {
        if let Some(last) = &self.latest_config_save {
            return model.settings != *last;
        }
        false
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
