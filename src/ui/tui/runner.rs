use std::borrow::Cow;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::MouseEventKind;
use crossterm::event::{self};
use ratatui::widgets::Widget;
use ratatui_toaster::ToastBuilder;
use ratatui_toaster::ToastType;
use tui_widgets::prompts::State as PromptState;
use tui_widgets::prompts::Status;

use crate::config::Config;
use crate::config::Percentage;
use crate::models::Pomodoro;
use crate::models::pomodoro::State;
use crate::services::SoundService;
use crate::services::cmd_runner::run_cmds;
use crate::services::notify::notify;
use crate::ui::tui::TuiError;
use crate::ui::tui::backend::Tui;
use crate::ui::tui::renderer::TuiRenderer;
use crate::ui::tui::toasts::ToastHandler;
use crate::ui::*;

type Sound = Box<dyn SoundService<SoundType = State>>;

pub struct TuiRunner {
    model: AppModel,

    router: Router,
    latest_config_save: Option<Config>,

    terminal: Tui,

    renderer: TuiRenderer,
    sound: Sound,
    tick: TickHandler,
    toast: ToastHandler,
}

impl Runner for TuiRunner {
    fn run(&mut self) -> Result<(), UiError> {
        Ok(self.run_loop()?)
    }
}

impl TuiRunner {
    pub fn new(model: AppModel, sound: Sound) -> Result<Self, TuiError> {
        let renderer = TuiRenderer::new();
        let terminal = Tui::new()?;

        Ok(Self {
            model,
            router: Router::new(Page::Timer),
            latest_config_save: None,
            renderer,
            terminal,
            sound,
            tick: TickHandler::default(),
            toast: ToastHandler::default(),
        })
    }

    fn run_loop(&mut self) -> Result<(), TuiError> {
        self.snapshot_settings();

        while !self.router.is_quit() {
            let mut redraw = self.tick.new_tick();

            if let Some(input) = Self::get_input()? {
                self.handle_key_event(input)?;
                redraw = true;
            }

            if redraw {
                self.tick();
                self.render_terminal()?;
            }
            sleep(Duration::from_millis(10));
        }
        Ok(())
    }

    fn get_input() -> Result<Option<Event>, TuiError> {
        if event::poll(Duration::from_millis(10))? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }

    fn tick(&mut self) {
        self.toast.tick();
        let auto_next = self.should_auto_next();
        let cmd = self.update_timer(TimerMsg::Tick { auto_next });

        self.handle_timer_cmd(cmd);
    }

    fn handle_timer_cmd(&mut self, cmd: TimerCmd) {
        match cmd {
            TimerCmd::None => {}
            TimerCmd::PromptNextSession => {
                if !self.renderer.timer.prompt_next_session() {
                    // only runs on once per session
                    self.on_session_end();
                }
                self.renderer.timer.set_prompt_next_session(true);
            }
            TimerCmd::NextSession => {
                self.on_session_end();
                self.next_session();
            }
            TimerCmd::SessionContinued => {}
        }
    }

    fn on_session_end(&mut self) {
        self.run_hooks();
        self.play_sound();
        self.send_notification();
    }

    fn run_hooks(&self) {
        run_cmds(&self.settings().pomodoro.hook, self.timer().state());
    }

    fn send_notification(&mut self) {
        if let Err(e) = notify(self.timer().next_state()) {
            self.show_toast(e.to_string(), ToastType::Error);
        }
    }

    fn play_sound(&mut self) {
        if !self.sound.is_playing() {
            self.sound.set_sound(self.timer().next_state());
            if let Err(e) = self.sound.play() {
                self.show_toast(e.to_string(), ToastType::Error);
            }
        }
    }

    fn show_toast(&mut self, message: impl Into<Cow<'static, str>>, r#type: ToastType) {
        self.toast
            .show_toast(ToastBuilder::new(message.into()).toast_type(r#type));
    }

    fn next_session(&mut self) {
        self.update_timer(TimerMsg::NextState);
    }

    fn should_auto_next(&self) -> bool {
        let timer = &self.settings().pomodoro.timer;
        match self.model.timer.state() {
            State::Focus => timer.auto_focus,
            State::LongBreak => timer.auto_long,
            State::ShortBreak => timer.auto_short,
        }
    }

    fn render_terminal(&mut self) -> Result<(), TuiError> {
        self.terminal.draw(|f| {
            let area = f.area();
            self.renderer
                .flush(f, &self.router, &self.model.timer, &self.model.settings);
            self.toast.set_area(area);
            self.toast.render(area, f.buffer_mut());
        })?;
        Ok(())
    }

    fn handle_key_event(&mut self, input: Event) -> Result<(), TuiError> {
        // Handle settings input directly on the renderer
        match self.router.active_page() {
            Some(Page::Settings) => self.handle_settings(input),
            Some(Page::Timer) => self.handle_timer(input),
            None => self.quit(),
        }
        Ok(())
    }

    fn handle_timer(&mut self, event: Event) {
        use KeyCode::*;
        use TimerMsg::*;

        if self.renderer.timer.prompt_next_session() {
            return self.handle_timer_nextstate_prompt(event);
        }

        if let Event::Key(key) = event {
            match key.code {
                Left | Char('h') => {
                    self.update_timer(Subtract(Duration::from_secs(30)));
                }
                Down | Char('j') => {
                    self.update_timer(Subtract(Duration::from_secs(60)));
                }
                Right | Char('l') => {
                    self.update_timer(Add(Duration::from_secs(30)));
                }
                Up | Char('k') => {
                    self.update_timer(Add(Duration::from_secs(60)));
                }
                Char(' ') => {
                    self.update_timer(TogglePause);
                }
                Enter => {
                    self.update_timer(SkipSession);
                }
                Backspace => {
                    self.update_timer(ResetSession);
                }
                Char('q') => self.quit(),
                Char('s') => self.router.navigate(Page::Settings),
                _ => {}
            }
        }
    }

    fn handle_timer_nextstate_prompt(&mut self, event: Event) {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Enter | KeyCode::Char('y') => {
                    self.next_session();
                    self.renderer.timer.set_prompt_next_session(false);
                }
                KeyCode::Esc | KeyCode::Char('n') => self.quit(),
                _ => {}
            }
        }
    }

    /// Handle settings page input directly, mutating renderer state
    fn handle_settings(&mut self, event: Event) {
        let renderer = &mut self.renderer.settings;

        // When editing, handle text input
        if renderer.is_editing() {
            return self.handle_settings_edit(event);
        }

        // When navigating, handle navigation input
        use KeyCode::*;
        match event {
            Event::Key(key) => match key.code {
                Up | Char('k') => renderer.select_up(),
                Down | Char('j') => renderer.select_down(),
                Enter => {
                    if SettingsMsg::is_toggle_index(renderer.selected_idx()) {
                        self.apply_settings_edit()
                    } else {
                        renderer.start_editing_for_field(&self.model.settings.pomodoro)
                    }
                }
                Char('s') => self.save_settings(),
                Char(' ') if SettingsMsg::is_toggle_index(renderer.selected_idx()) => {
                    self.apply_settings_edit()
                }
                Esc => self.router.navigate(Page::Timer),
                Char('q') => self.quit(),
                _ => {}
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollDown => renderer.scroll_down(),
                MouseEventKind::ScrollUp => renderer.scroll_up(),
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_settings_edit(&mut self, event: Event) {
        let settings = &mut self.renderer.settings;

        if let Event::Key(key) = event
            && let Some(prompt) = settings.prompt_state_mut()
        {
            prompt.text_state.handle_key_event(key);

            match prompt.text_state.status() {
                Status::Done => {
                    self.apply_settings_edit();
                }
                Status::Aborted => {
                    settings.cancel_editing();
                }
                _ => {}
            }
        }
    }

    fn apply_settings_edit(&mut self) {
        let settings = &mut self.renderer.settings;
        let selected_idx = settings.selected_idx();
        let value = settings.take_edit_value();
        settings.cancel_editing();

        let msg = match self.msg_from_edit(value, selected_idx) {
            Some(msg) => msg,
            None => return,
        };

        self.update_settings(msg);

        self.renderer
            .settings
            .set_has_unsaved_changes(self.check_settings_updated());
    }

    fn msg_from_edit(&mut self, value: String, selected_idx: u32) -> Option<SettingsMsg> {
        use SettingsMsg::*;
        let msg = match selected_idx {
            // Timer settings (0-6)
            0 => TimerFocus(self.parse_dur(value)?),
            1 => TimerShort(self.parse_dur(value)?),
            2 => TimerLong(self.parse_dur(value)?),
            3 => TimerLongInterval(self.try_parse(value, |s| s.parse::<u32>(), "integer")?),
            4 => TimerAutoFocus,
            5 => TimerAutoShort,
            6 => TimerAutoLong,
            // Hook settings (7-9)
            7 => HookFocus(value),
            8 => HookShort(value),
            9 => HookLong(value),
            // Alarm path settings (10-12)
            10 => AlarmPathFocus(self.parse_path(value)),
            11 => AlarmPathShort(self.parse_path(value)),
            12 => AlarmPathLong(self.parse_path(value)),
            // Alarm volume settings (10-12)
            13 => AlarmVolumeFocus(self.parse_vol(value)?),
            14 => AlarmVolumeShort(self.parse_vol(value)?),
            15 => AlarmVolumeLong(self.parse_vol(value)?),
            _ => return None,
        };
        Some(msg)
    }

    fn save_settings(&mut self) {
        match self.model.settings.save() {
            Ok(()) => {
                self.renderer.settings.set_has_unsaved_changes(false);
                self.snapshot_settings();
                self.show_toast("Settings saved!", ToastType::Success);
            }
            Err(e) => {
                self.show_toast(format!("Failed to save: {e}"), ToastType::Error);
            }
        }
    }

    /// Compare current config with when it was latest saved.
    fn check_settings_updated(&self) -> bool {
        if let Some(last) = &self.latest_config_save {
            return self.model.settings != *last;
        }
        true
    }

    /// Snapshot current settings.
    ///
    /// Use with [`Self::check_settings_updated`]
    fn snapshot_settings(&mut self) {
        self.latest_config_save = Some(self.model.settings.clone())
    }

    fn quit(&mut self) {
        self.router.navigate(Navigation::Quit);
    }

    fn update_timer(&mut self, msg: TimerMsg) -> TimerCmd {
        self.model.timer.update(msg)
    }

    fn update_settings(&mut self, msg: SettingsMsg) -> SettingsCmd {
        self.model.settings.update(msg)
    }

    fn timer(&self) -> &Pomodoro {
        &self.model.timer
    }

    fn settings(&self) -> &Config {
        &self.model.settings
    }

    fn parse_path(&mut self, s: impl AsRef<str>) -> Option<PathBuf> {
        let s = s.as_ref();
        if s.is_empty() {
            None
        } else {
            let path = PathBuf::from(s);
            if !path.exists() {
                self.show_toast("Path does not exist", ToastType::Warning);
            }
            Some(path)
        }
    }

    fn parse_dur(&mut self, s: impl AsRef<str>) -> Option<Duration> {
        self.try_parse(s, |s| s.parse::<u64>(), "integer")
            .map(|val| Duration::from_secs(val * 60))
    }

    fn parse_vol(&mut self, s: impl AsRef<str>) -> Option<Percentage> {
        let s = s.as_ref();
        if s.is_empty() {
            Some(Percentage::default())
        } else {
            self.try_parse(s, |s| Percentage::try_from(s), "percent")
        }
    }

    fn try_parse<T, E: std::fmt::Debug>(
        &mut self,
        s: impl AsRef<str>,
        f: impl for<'a> FnOnce(&'a str) -> Result<T, E>,
        label: &str,
    ) -> Option<T> {
        let s = s.as_ref();
        f(s).map_err(|e| {
            self.show_toast(
                format!("Failed converting {s} to {label}: {e:?}"),
                ToastType::Error,
            );
        })
        .ok()
    }
}

struct TickHandler {
    last_tick: Option<Instant>,
    tick_rate: Duration,
}

impl TickHandler {
    fn new_tick(&mut self) -> bool {
        match self.last_tick {
            Some(last) => {
                let now = Instant::now();
                let new_tick = now.duration_since(last) >= self.tick_rate;
                if new_tick {
                    self.last_tick = Some(now);
                }
                new_tick
            }
            None => true,
        }
    }
}

impl Default for TickHandler {
    fn default() -> Self {
        Self {
            last_tick: None,
            tick_rate: Duration::from_secs(1),
        }
    }
}
