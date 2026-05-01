use std::borrow::Cow;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::MouseEventKind;
use crossterm::event::{self};
use ratatui_toaster::ToastBuilder;
use ratatui_toaster::ToastPosition;
use ratatui_toaster::ToastType;
use tui_widgets::prompts::State as PromptState;
use tui_widgets::prompts::Status;

use crate::config::Config;
use crate::config::Percentage;
use crate::models::Pomodoro;
use crate::models::pomodoro::Mode;
use crate::services::SoundService;
use crate::services::cmd_runner::run_cmds;
use crate::services::notify::notify;
use crate::ui::tui::TuiError;
use crate::ui::tui::backend::Tui;
use crate::ui::tui::model::SettingsItem;
use crate::ui::tui::model::SettingsModel;
use crate::ui::tui::model::SettingsMsg;
use crate::ui::tui::model::TimerModel;
use crate::ui::tui::model::TimerMsg;
use crate::ui::tui::toasts::ToastHandler;
use crate::ui::tui::view::Canvas;
use crate::ui::tui::view::TuiState;
use crate::ui::tui::view::settings::SettingsState;
use crate::ui::tui::view::timer::TimerState;
use crate::ui::*;

type Sound = Box<dyn SoundService<SoundType = Mode> + Send + Sync>;
type View = Box<
    dyn for<'a, 'b> StatefulViewRef<Canvas<'a, 'b>, State = TuiState, Result = ()> + Send + Sync,
>;

pub struct TuiRunner {
    state: TuiState,
    latest_config_save: Option<Config>,

    terminal: Tui,
    view: View,

    sound: Sound,
    toast: ToastHandler,
    tick: TickHandler,
}

impl Runner for TuiRunner {
    fn run(&mut self) -> Result<(), UiError> {
        Ok(self.run_loop()?)
    }
}

impl TuiRunner {
    pub fn new(pomo: Pomodoro, conf: Config, view: View, sound: Sound) -> Result<Self, TuiError> {
        let terminal = Tui::new()?;

        let state = TuiState::new(
            Router::new(Page::Timer),
            TimerState::new(TimerModel::new(), pomo),
            SettingsState::new(SettingsModel::new(), conf),
        );

        Ok(Self {
            state,
            latest_config_save: None,
            terminal,
            view,
            sound,
            toast: ToastHandler::default(),
            tick: TickHandler::default(),
        })
    }

    fn run_loop(&mut self) -> Result<(), TuiError> {
        self.snapshot_settings();

        while !self.router().is_quit() {
            let mut redraw = self.tick.new_tick();

            if let Some(input) = Self::get_input()? {
                redraw = self.handle_key_event(input)?;
            }

            if redraw {
                self.tick();
                self.render_terminal()?;
            }
            std::thread::sleep(Duration::from_millis(50));
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

    fn render_terminal(&mut self) -> Result<(), TuiError> {
        self.terminal.draw(|f| {
            self.toast.set_area(f.area());
            self.view.render_stateful_ref(f, &mut self.state);
            f.render_widget(&*self.toast, f.area());
        })?;
        Ok(())
    }

    fn tick(&mut self) {
        self.toast.tick();
        let auto_next = self.should_auto_next();
        let cmd = self.update_pomo(PomodoroMsg::Tick { auto_next });

        self.handle_pomodoro_cmd(cmd);
    }

    fn handle_pomodoro_cmd(&mut self, cmd: PomodoroCmd) {
        match cmd {
            PomodoroCmd::None => {}
            PomodoroCmd::PromptNextSession => {
                if !self.timer().prompt_transition() {
                    // only runs on once per session
                    self.on_session_end();
                }
                self.update_timer(TimerMsg::SetPromptNextSession(true));
            }
            PomodoroCmd::NextSession => {
                self.on_session_end();
                self.transition();
            }
            PomodoroCmd::SessionContinued => {}
        }
    }

    fn on_session_end(&mut self) {
        self.run_hooks();
        self.play_sound();
        self.send_notification();
    }

    fn run_hooks(&self) {
        run_cmds(&self.conf().pomodoro.hook, self.state.pomo().mode());
    }

    fn send_notification(&mut self) {
        if let Err(e) = notify(self.pomo().next_mode()) {
            self.show_toast(e.to_string(), ToastType::Error);
        }
    }

    fn show_toast(&mut self, message: impl Into<Cow<'static, str>>, r#type: ToastType) {
        self.toast.show_toast(
            ToastBuilder::new(message.into())
                .toast_type(r#type)
                .deduplicate(true)
                .position(ToastPosition::TopRight),
        );
    }

    fn play_sound(&mut self) {
        if !self.sound.is_playing() {
            self.sound.set_sound(self.pomo().next_mode());
            if let Err(e) = self.sound.play() {
                self.show_toast(e.to_string(), ToastType::Error);
            }
        }
    }

    fn transition(&mut self) {
        self.update_pomo(PomodoroMsg::NextState);
    }

    fn should_auto_next(&self) -> bool {
        let timer = &self.conf().pomodoro.timer;
        match self.pomo().mode() {
            Mode::Focus => timer.auto_focus,
            Mode::LongBreak => timer.auto_long,
            Mode::ShortBreak => timer.auto_short,
        }
    }

    fn handle_key_event(&mut self, input: Event) -> Result<bool, TuiError> {
        let mut redraw = true;
        match self.router().active_page() {
            Some(Page::Settings) => redraw = self.handle_settings(input),
            Some(Page::Timer) => redraw = self.handle_timer(input),
            None => self.quit(),
        }
        Ok(redraw)
    }

    fn handle_timer(&mut self, event: Event) -> bool {
        use KeyCode::*;
        use PomodoroMsg::*;

        if self.timer().prompt_transition() {
            return self.handle_timer_transition(event);
        }

        let mut redraw = true;
        if let Event::Key(key) = event {
            match key.code {
                Right | Char('l') => {
                    self.update_pomo(Subtract(Duration::from_secs(30)));
                }
                Down | Char('j') => {
                    self.update_pomo(Subtract(Duration::from_secs(60)));
                }
                Left | Char('h') => {
                    self.update_pomo(Add(Duration::from_secs(30)));
                }
                Up | Char('k') => {
                    self.update_pomo(Add(Duration::from_secs(60)));
                }
                Char(' ') => {
                    self.update_pomo(TogglePause);
                }
                Enter => {
                    self.update_pomo(SkipSession);
                }
                Backspace => {
                    self.update_pomo(ResetSession);
                }
                Char('q') => self.quit(),
                Char('s') => self.router_mut().navigate(Page::Settings),
                Char('/') | Char('?') => {
                    self.timer_mut().toggle_keybinds();
                }
                _ => redraw = false,
            }
        } else {
            redraw = false
        }
        redraw
    }

    fn handle_timer_transition(&mut self, event: Event) -> bool {
        let mut redraw = true;
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Enter | KeyCode::Char('y') => {
                    self.transition();
                    self.update_timer(TimerMsg::SetPromptNextSession(false));
                }
                KeyCode::Esc | KeyCode::Char('n') => self.quit(),
                _ => redraw = false,
            }
        } else {
            redraw = false
        }
        redraw
    }

    /// Handle settings page input directly, mutating renderer state
    fn handle_settings(&mut self, event: Event) -> bool {
        // When editing, handle text input
        if self.settings().is_editing() {
            return self.handle_settings_edit(event);
        }

        let mut redraw = true;
        // When navigating, handle navigation input
        use KeyCode::*;
        use SettingsMsg::*;
        match event {
            Event::Key(key) => match key.code {
                Up | Char('k') => {
                    let _ = self.update_settings(SelectUp);
                }
                BackTab => {
                    let _ = self.update_settings(SectionPrev);
                }
                Tab => {
                    let _ = self.update_settings(SectionNext);
                }
                Down | Char('j') => {
                    let _ = self.update_settings(SelectDown);
                }
                Enter => {
                    if self.settings_mut().selected().is_toggle() {
                        self.apply_settings_edit()
                    } else {
                        let pomo = &self.conf().pomodoro.clone();
                        self.settings_mut().start_editing_for_field(pomo)
                    }
                }
                Char('1') => {
                    let _ = self.update_settings(SectionSelect(0));
                }
                Char('2') => {
                    let _ = self.update_settings(SectionSelect(1));
                }
                Char('3') => {
                    let _ = self.update_settings(SectionSelect(2));
                }
                Char('s') => self.save_settings(),
                Char(' ') if self.settings().selected().is_toggle() => self.apply_settings_edit(),
                Esc => self.router_mut().navigate(Page::Timer),
                Char('q') => self.quit(),
                Char('/') | Char('?') => self.settings_mut().toggle_keybinds(),
                _ => redraw = false,
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollDown => {
                    let _ = self.update_settings(ScrollUp);
                }
                MouseEventKind::ScrollUp => {
                    let _ = self.update_settings(ScrollDown);
                }
                _ => redraw = false,
            },
            _ => redraw = false,
        }
        redraw
    }

    fn handle_settings_edit(&mut self, event: Event) -> bool {
        let mut redraw = true;
        if let Event::Key(key) = event
            && let Some(prompt) = self.settings_mut().prompt_state_mut()
        {
            prompt.text_state.handle_key_event(key);

            match prompt.text_state.status() {
                Status::Done => {
                    self.apply_settings_edit();
                }
                Status::Aborted => {
                    self.update_settings(SettingsMsg::CancelEditing);
                }
                _ => redraw = false,
            }
        } else {
            redraw = false;
        }
        redraw
    }

    fn apply_settings_edit(&mut self) {
        let selected = self.settings().selected();
        let value = self.settings_mut().take_edit_value();
        self.update_settings(SettingsMsg::CancelEditing);

        let msg = match self.msg_from_edit(value, selected) {
            Some(msg) => msg,
            None => return,
        };

        self.update_conf(msg);
        let is_unsaved = self.check_settings_unsaved();
        self.update_settings(SettingsMsg::SetUnsavedChanges(is_unsaved));
    }

    fn msg_from_edit(&mut self, value: String, selected: SettingsItem) -> Option<ConfigMsg> {
        use ConfigMsg::*;
        use SettingsItem as I;
        let msg = match selected {
            I::TimerFocus => TimerFocus(self.parse_dur(value)?),
            I::TimerShort => TimerShort(self.parse_dur(value)?),
            I::TimerLong => TimerLong(self.parse_dur(value)?),
            I::TimerLongInterval => {
                TimerLongInterval(self.try_parse(value, |s| s.parse::<u32>(), "integer")?)
            }
            I::TimerAutoFocus => TimerAutoFocus,
            I::TimerAutoShort => TimerAutoShort,
            I::TimerAutoLong => TimerAutoLong,
            I::HookFocus => HookFocus(value),
            I::HookShort => HookShort(value),
            I::HookLong => HookLong(value),
            I::AlarmPathFocus => AlarmPathFocus(self.parse_path(value)),
            I::AlarmPathShort => AlarmPathShort(self.parse_path(value)),
            I::AlarmPathLong => AlarmPathLong(self.parse_path(value)),
            I::AlarmVolumeFocus => AlarmVolumeFocus(self.parse_vol(value)?),
            I::AlarmVolumeShort => AlarmVolumeShort(self.parse_vol(value)?),
            I::AlarmVolumeLong => AlarmVolumeLong(self.parse_vol(value)?),
        };
        Some(msg)
    }

    fn save_settings(&mut self) {
        match self.conf().save() {
            Ok(()) => {
                self.update_settings(SettingsMsg::SetUnsavedChanges(false));
                self.snapshot_settings();
                self.show_toast("Settings saved!", ToastType::Success);
            }
            Err(e) => {
                self.show_toast(format!("Failed to save: {e}"), ToastType::Error);
            }
        }
    }

    /// Compare current config with when it was latest saved.
    fn check_settings_unsaved(&self) -> bool {
        if let Some(last) = &self.latest_config_save {
            return *self.conf() != *last;
        }
        true
    }

    /// Snapshot current settings.
    ///
    /// Use with [`Self::check_settings_updated`]
    fn snapshot_settings(&mut self) {
        self.latest_config_save = Some(self.conf().clone())
    }

    fn quit(&mut self) {
        self.router_mut().navigate(Navigation::Quit);
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

impl DerefMut for TuiRunner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl Deref for TuiRunner {
    type Target = TuiState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

struct TickHandler {
    last_tick: Instant,
    tick_rate: Duration,
}

impl TickHandler {
    fn new(tick_rate: Duration) -> Self {
        Self {
            last_tick: Instant::now(),
            tick_rate,
        }
    }

    fn new_tick(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_tick) >= self.tick_rate {
            self.last_tick = now;
            true
        } else {
            false
        }
    }
}

impl Default for TickHandler {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}
