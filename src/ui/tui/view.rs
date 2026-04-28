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
use crate::config::pomodoro::Hooks;
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
use crate::ui::tui::renderer::TuiRenderer;
use crate::ui::tui::toasts::ToastHandler;
use crate::ui::update::settings::SettingsMsg;
use crate::ui::update::settings::SettingsUpdate;
use crate::ui::update::timer::TimerCmd;
use crate::ui::update::timer::TimerMsg;
use crate::ui::update::timer::TimerUpdate;

type Sound = Box<dyn SoundService<SoundType = State>>;

pub struct TuiView {
    router: Router,
    latest_config_save: Option<Config>,

    terminal: Tui,

    renderer: TuiRenderer,
    sound: Sound,
    tick: TickHandler,
    toast: ToastHandler,
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
            renderer,
            terminal,
            sound,
            tick: TickHandler::default(),
            toast: ToastHandler::default(),
        })
    }

    fn run_loop(&mut self, mut model: AppModel) -> Result<(), TuiError> {
        self.snapshot_settings(&model);

        while !self.router.is_quit() {
            let mut redraw = self.tick.new_tick();

            if let Some(input) = Self::get_input()? {
                model = self.handle_key_event(input, model)?;
                redraw = true;
            }

            if redraw {
                model = self.tick(model);
                self.render_terminal(&model)?;
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

    fn tick(&mut self, mut model: AppModel) -> AppModel {
        self.toast.tick();
        let cmd = TimerUpdate::tick(
            self.should_auto_next(model.timer.state(), &model.settings.pomodoro.timer),
            &model.timer,
        );

        model = self.handle_timer_cmd(cmd, model);
        model
    }

    fn handle_timer_cmd(&mut self, cmd: TimerCmd, mut model: AppModel) -> AppModel {
        match cmd {
            TimerCmd::None => {}
            TimerCmd::PromptNextSession => {
                if !self.renderer.timer.prompt_next_session() {
                    // only runs on once per session
                    self.on_session_end(&model);
                }
                self.renderer.timer.set_prompt_next_session(true);
            }
            TimerCmd::NextSession => {
                self.on_session_end(&model);
                model = self.next_session(model);
            }
            TimerCmd::SessionContinued => {}
        }
        model
    }

    fn on_session_end(&mut self, model: &AppModel) {
        self.run_hooks(&model.settings.pomodoro.hook, model.timer.state());
        self.play_sound(model.timer.next_state());
        self.send_notification(model.timer.next_state());
    }

    fn run_hooks(&self, conf: &Hooks, curr_state: State) {
        run_cmds(conf, curr_state);
    }

    fn send_notification(&mut self, next_state: State) {
        if let Err(e) = notify(next_state) {
            self.show_toast(e.to_string(), ToastType::Error);
        }
    }

    fn play_sound(&mut self, next_state: State) {
        if !self.sound.is_playing() {
            self.sound.set_sound(next_state);
            if let Err(e) = self.sound.play() {
                self.show_toast(e.to_string(), ToastType::Error);
            }
        }
    }

    fn show_toast(&mut self, message: impl Into<Cow<'static, str>>, r#type: ToastType) {
        self.toast
            .show_toast(ToastBuilder::new(message.into()).toast_type(r#type));
    }

    fn next_session(&mut self, mut model: AppModel) -> AppModel {
        (model.timer, _) = TimerUpdate::update(TimerMsg::NextState, model.timer);
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
            let area = f.area();
            self.renderer
                .flush(f, &self.router, &model.timer, &model.settings);
            self.toast.set_area(area);
            self.toast.render(area, f.buffer_mut());
        })?;
        Ok(())
    }

    fn handle_key_event(
        &mut self,
        input: Event,
        mut model: AppModel,
    ) -> Result<AppModel, TuiError> {
        // Handle settings input directly on the renderer
        match self.router.active_page() {
            Some(Page::Settings) => model = self.handle_settings(input, model),
            Some(Page::Timer) => model = self.handle_timer(input, model),
            None => self.quit(),
        }
        Ok(model)
    }

    fn handle_timer(&mut self, event: Event, mut model: AppModel) -> AppModel {
        use KeyCode as K;
        use TimerMsg::*;

        if self.renderer.timer.prompt_next_session() {
            return self.handle_timer_nextstate_prompt(event, model);
        }

        if let Event::Key(key) = event {
            match key.code {
                K::Left | K::Char('h') => {
                    (model.timer, _) =
                        TimerUpdate::update(Subtract(Duration::from_secs(30)), model.timer);
                }
                K::Down | K::Char('j') => {
                    (model.timer, _) =
                        TimerUpdate::update(Subtract(Duration::from_secs(60)), model.timer);
                }
                K::Right | K::Char('l') => {
                    (model.timer, _) =
                        TimerUpdate::update(Add(Duration::from_secs(30)), model.timer);
                }
                K::Up | K::Char('k') => {
                    (model.timer, _) =
                        TimerUpdate::update(Add(Duration::from_secs(60)), model.timer);
                }
                K::Char(' ') => {
                    (model.timer, _) = TimerUpdate::update(TogglePause, model.timer);
                }
                K::Enter => {
                    (model.timer, _) = TimerUpdate::update(SkipSession, model.timer);
                }
                K::Backspace => {
                    (model.timer, _) = TimerUpdate::update(ResetSession, model.timer);
                }
                K::Char('q') => self.quit(),
                K::Char('s') => self.router.navigate(Page::Settings),
                _ => {}
            }
        }
        model
    }

    fn handle_timer_nextstate_prompt(&mut self, event: Event, mut model: AppModel) -> AppModel {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Enter | KeyCode::Char('y') => {
                    model = self.next_session(model);
                    self.renderer.timer.set_prompt_next_session(false);
                }
                KeyCode::Esc | KeyCode::Char('n') => self.quit(),
                _ => {}
            }
        }
        model
    }

    /// Handle settings page input directly, mutating renderer state
    fn handle_settings(&mut self, event: Event, mut model: AppModel) -> AppModel {
        let settings = &mut self.renderer.settings;

        // When editing, handle text input
        if settings.is_editing() {
            return self.handle_settings_edit(event, model);
        }

        // When navigating, handle navigation input
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up | KeyCode::Char('k') => settings.select_up(),
                KeyCode::Down | KeyCode::Char('j') => settings.select_down(),
                KeyCode::Enter => {
                    if SettingsMsg::is_toggle_index(settings.selected_idx()) {
                        model = self.update_settings(model)
                    } else {
                        settings.start_editing_for_field(&model.settings.pomodoro)
                    }
                }
                KeyCode::Char('s') => self.save_settings(&model),
                KeyCode::Char(' ') if SettingsMsg::is_toggle_index(settings.selected_idx()) => {
                    model = self.update_settings(model)
                }
                KeyCode::Esc => self.router.navigate(Page::Timer),
                KeyCode::Char('q') => self.quit(),
                _ => {}
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollDown => settings.scroll_down(),
                MouseEventKind::ScrollUp => settings.scroll_up(),
                _ => {}
            },
            _ => {}
        }

        model
    }

    fn handle_settings_edit(&mut self, event: Event, mut model: AppModel) -> AppModel {
        let settings = &mut self.renderer.settings;

        if let Event::Key(key) = event
            && let Some(prompt) = settings.prompt_state_mut()
        {
            prompt.text_state.handle_key_event(key);

            match prompt.text_state.status() {
                Status::Done => {
                    model = self.update_settings(model);
                }
                Status::Aborted => {
                    settings.cancel_editing();
                }
                _ => {}
            }
        }
        model
    }

    fn update_settings(&mut self, mut model: AppModel) -> AppModel {
        let settings = &mut self.renderer.settings;
        let selected_idx = settings.selected_idx();
        let value = settings.take_edit_value();
        settings.cancel_editing();

        let msg = match self.msg_from_edit(value, selected_idx) {
            Some(msg) => msg,
            None => return model,
        };

        (model.settings, _) = SettingsUpdate::update(msg, model.settings);

        self.renderer
            .settings
            .set_has_unsaved_changes(self.check_settings_updated(&model));
        model
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

    fn save_settings(&mut self, model: &AppModel) {
        match model.settings.save() {
            Ok(()) => {
                self.renderer.settings.set_has_unsaved_changes(false);
                self.snapshot_settings(model);
                self.show_toast("Settings saved!", ToastType::Success);
            }
            Err(e) => {
                self.show_toast(format!("Failed to save: {e}"), ToastType::Error);
            }
        }
    }

    /// Compare current config with when it was latest saved.
    fn check_settings_updated(&self, model: &AppModel) -> bool {
        if let Some(last) = &self.latest_config_save {
            return model.settings != *last;
        }
        true
    }

    /// Snapshot current settings.
    ///
    /// Use with [`Self::check_settings_updated`]
    fn snapshot_settings(&mut self, model: &AppModel) {
        self.latest_config_save = Some(model.settings.clone())
    }

    fn quit(&mut self) {
        self.router.navigate(Navigation::Quit);
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
