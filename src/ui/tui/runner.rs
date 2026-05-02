use std::borrow::Cow;
use std::time::Duration;
use std::time::Instant;

use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::MouseEventKind;
use crossterm::event::{self};
use log::debug;
use ratatui_toaster::ToastBuilder;
use ratatui_toaster::ToastPosition;
use ratatui_toaster::ToastType as ToasterType;
use tui_widgets::prompts::State as PromptState;
use tui_widgets::prompts::Status;

use crate::config::Alarm;
use crate::config::Config;
use crate::model::Mode;
use crate::model::Pomodoro;
use crate::repo::Repos;
use crate::repo::model::PomodoroState;
use crate::repo::model::Session;
use crate::service::SoundService;
use crate::service::cmd_runner::run_cmds;
use crate::service::notify::notify;
use crate::ui::UiError;
use crate::ui::prelude::*;
use crate::ui::tui::TuiError;
use crate::ui::tui::backend::Tui;
use crate::ui::tui::toast::ToastHandler;
use crate::ui::tui::view::*;

type Sound = Box<dyn SoundService<SoundType = Alarm>>;
type View = Box<dyn for<'a, 'b> StatefulViewRef<Canvas<'a, 'b>, State = TuiState, Result = ()>>;
type Repo = Box<dyn Repos>;

pub struct TuiRunner {
    state: TuiState,

    redraw: bool,
    active_session: Option<Session>,

    terminal: Tui,
    view: View,

    sound: Sound,
    tick: TickTimer,
    repo: Repo,
}

impl Runner for TuiRunner {
    fn run(&mut self) -> Result<(), UiError> {
        Ok(self.run_loop()?)
    }
}

impl TuiRunner {
    pub fn new(
        pomo: Pomodoro,
        conf: Config,
        view: View,
        sound: Sound,
        repo: Repo,
    ) -> Result<Self, TuiError> {
        let terminal = Tui::new()?;

        let state = TuiState::new(
            Router::new(Page::Timer),
            TimerState::new(TimerModel::new(), pomo),
            SettingsState::new(SettingsModel::new(), conf),
            ToastHandler::default(),
        );

        Ok(Self {
            state,
            redraw: true,
            active_session: None,
            terminal,
            view,
            sound,
            tick: TickTimer::default(),
            repo,
        })
    }

    fn run_loop(&mut self) -> Result<(), TuiError> {
        self.state.snapshot_settings();
        self.initial_run();

        while !self.router().is_quit() {
            self.render_terminal()?;
            self.tick();
            self.handle_inputs()?;
        }
        Ok(())
    }

    fn render_terminal(&mut self) -> Result<(), TuiError> {
        if self.take_redraw() {
            self.terminal.draw(|f| {
                self.view.render_stateful_ref(f, &mut self.state);
                if self.state.toast.has_toast() {
                    self.state.toast.set_area(f.area());
                    f.render_widget(&*self.state.toast, f.area());
                }
            })?;
        }
        Ok(())
    }

    fn tick(&mut self) {
        if self.tick.new_tick() {
            self.session_tick();
            self.state.toast.tick();
            self.update_pomo(PomodoroMsg::Tick);
            self.redraw();
        }
    }

    fn initial_run(&mut self) {
        if self.conf().pomodoro.timer.auto_start_on_launch {
            self.update_pomo(PomodoroMsg::Start);
        } else {
            self.update_pomo(PomodoroMsg::StartPaused);
        }
    }
}

// ---------------------------------------------------------
//  ___ ___ _____   _____ ___ ___
// / __| __| _ \ \ / /_ _/ __| __|
// \__ \ _||   /\ V / | | (__| _|
// |___/___|_|_\ \_/ |___\___|___|
// ---------------------------------------------------------

impl TuiRunner {
    fn on_session_end(&mut self) {
        self.session_stop();
        self.run_hooks();
        self.play_sound();
        self.send_notification();
    }

    fn run_hooks(&self) {
        run_cmds(&self.conf().pomodoro.hook, self.pomo().mode());
    }

    fn send_notification(&mut self) {
        if let Err(e) = notify(self.pomo().next_mode()) {
            self.show_toast(e.to_string(), ToastType::Error);
        }
    }

    fn show_toast(&mut self, message: impl Into<Cow<'static, str>>, r#type: ToastType) {
        self.toast_mut().show_toast(
            ToastBuilder::new(message.into())
                .toast_type(r#type.into())
                .deduplicate(true)
                .position(ToastPosition::TopRight),
        );
    }

    fn play_sound(&mut self) {
        if !self.sound.is_playing() {
            let alarms = &self.state.settings.conf.pomodoro.alarm;
            let alarm = match self.pomo().next_mode() {
                Mode::Focus => &alarms.focus,
                Mode::LongBreak => &alarms.long,
                Mode::ShortBreak => &alarms.short,
            };
            self.sound.set_sound(alarm.clone());
            if let Err(e) = self.sound.play() {
                self.show_toast(e.to_string(), ToastType::Error);
            }
        }
    }
}

// ---------------------------------------------------------
//  ___ _  _ ___ _   _ _____
// |_ _| \| | _ \ | | |_   _|
//  | || .` |  _/ |_| | | |
// |___|_|\_|_|  \___/  |_|
// ---------------------------------------------------------

impl TuiRunner {
    fn handle_inputs(&mut self) -> Result<(), TuiError> {
        if event::poll(self.tick.time_until_next())? {
            while event::poll(Duration::ZERO)? {
                let event = event::read()?;
                // fix windows sending duplicate KeyDown and KeyUp events
                #[cfg(target_os = "windows")]
                if let Event::Key(key) = &event {
                    if key.kind == event::KeyEventKind::Release {
                        continue;
                    }
                }
                self.handle_key_event(event)?;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, input: Event) -> Result<(), TuiError> {
        match self.router().active_page() {
            Some(Page::Settings) => self.handle_settings(input),
            Some(Page::Timer) => self.handle_timer(input),
            None => self.quit(),
        }
        Ok(())
    }

    fn handle_timer(&mut self, event: Event) {
        use KeyCode::*;
        use PomodoroMsg::*;

        if self.timer().prompt_transition() {
            return self.handle_timer_transition(event);
        }

        if let Event::Key(key) = event {
            match key.code {
                Right | Char('l') => self.update_pomo(Subtract(Duration::from_secs(30))),
                Down | Char('j') => self.update_pomo(Subtract(Duration::from_secs(60))),
                Left | Char('h') => self.update_pomo(Add(Duration::from_secs(30))),
                Up | Char('k') => self.update_pomo(Add(Duration::from_secs(60))),
                Char(' ') => self.update_pomo(TogglePause),
                Enter => self.update_pomo(SkipSession),
                Backspace => self.update_pomo(ResetSession),
                Char('q') => self.quit(),
                Char('s') => self.router_mut().navigate(Page::Settings),
                Char('/') | Char('?') => self.update_timer(TimerMsg::ToggleShowKeybinds),
                _ => {}
            }
        }
    }

    /// Handle settings page input directly, mutating renderer state
    fn handle_settings(&mut self, event: Event) {
        // When editing, handle text input
        if self.settings().is_editing() {
            return self.handle_settings_edit(event);
        }

        // When navigating, handle navigation input
        use KeyCode::*;
        use SettingsMsg::*;
        match event {
            Event::Key(key) => match key.code {
                Up | Char('k') => self.update_settings(SelectUp),
                BackTab => self.update_settings(SectionPrev),
                Tab => self.update_settings(SectionNext),
                Down | Char('j') => self.update_settings(SelectDown),
                Enter | Char(' ') => {
                    if self.settings().selected().is_toggle() {
                        self.update_settings(SaveEdit);
                    } else {
                        let pomo = &self.conf().pomodoro.clone();
                        self.settings_mut().start_editing_for_field(pomo)
                    }
                }
                Char('1') => self.update_settings(SectionSelect(0)),
                Char('2') => self.update_settings(SectionSelect(1)),
                Char('3') => self.update_settings(SectionSelect(2)),
                Char('s') => self.save_settings(),
                Esc => self.router_mut().navigate(Page::Timer),
                Char('q') => self.quit(),
                Char('/') | Char('?') => self.update_settings(ToggleShowKeybinds),
                _ => {}
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollDown => self.update_settings(ScrollUp),
                MouseEventKind::ScrollUp => self.update_settings(ScrollDown),
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_timer_transition(&mut self, event: Event) {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Enter | KeyCode::Char('y') => {
                    self.update_timer(TimerMsg::PromptNextSessionAnswerYes(true));
                }
                KeyCode::Esc | KeyCode::Char('n') => {
                    self.update_timer(TimerMsg::PromptNextSessionAnswerYes(false));
                }
                _ => {}
            }
        }
    }

    fn handle_settings_edit(&mut self, event: Event) {
        if let Event::Key(key) = event
            && let Some(prompt) = self.settings_mut().prompt_state_mut()
        {
            prompt.text_state.handle_key_event(key);

            match prompt.text_state.status() {
                Status::Done => self.update_settings(SettingsMsg::SaveEdit),
                Status::Aborted => self.update_settings(SettingsMsg::CancelEditing),
                _ => {}
            }
        }
    }
}

// ---------------------------------------------------------
//  ___ _____ _ _____ ___    ___  ___ ___ ___    _ _____ ___ ___  _  _ ___
// / __|_   _/_\_   _| __|  / _ \| _ \ __| _ \  /_\_   _|_ _/ _ \| \| / __|
// \__ \ | |/ _ \| | | _|  | (_) |  _/ _||   / / _ \| |  | | (_) | .` \__ \
// |___/ |_/_/ \_\_| |___|  \___/|_| |___|_|_\/_/ \_\_| |___\___/|_|\_|___/
// ---------------------------------------------------------

impl TuiRunner {
    fn handle_pomodoro_cmd(&mut self, cmd: PomodoroCmd) {
        use PomodoroCmd::*;
        match cmd {
            Started => {
                self.session_new();
            }
            StartedPaused => {}
            SessionEnd => {
                if self.should_auto_next() {
                    self.on_session_end();
                    self.update_pomo(PomodoroMsg::NextSession);
                } else {
                    // Will repeatedly run on session end
                    if !self.timer().prompt_transition() {
                        self.on_session_end();
                    }
                    self.update_timer(TimerMsg::SetPromptNextSession(true));
                }
            }
            NextSession => {
                self.session_new();
            }
            SessionPaused => {
                self.session_stop();
            }
            SessionResumed => {
                self.session_new();
            }
            SessionSkipped => {
                self.session_stop();
                self.session_new();
            }
        }
    }

    fn handle_settings_cmd(&mut self, cmd: SettingsCmd) {
        use SettingsCmd::*;
        match cmd {
            SaveEdit(msg) => {
                self.update_conf(msg);
                let is_unsaved = self.state.check_settings_unsaved();
                self.update_settings(SettingsMsg::SetUnsavedChanges(is_unsaved));
            }
            ShowToast { message, r#type } => {
                self.show_toast(message, r#type);
            }
        }
    }

    fn handle_timer_cmd(&mut self, cmd: TimerCmd) {
        use TimerCmd::*;
        match cmd {
            PromptTransitionAnsweredYes => {
                self.update_pomo(PomodoroMsg::NextSession);
                self.update_timer(TimerMsg::SetPromptNextSession(false));
                let _ = self.sound.stop();
            }
            PromptTransitionAnsweredNo => {
                self.update_pomo(PomodoroMsg::Pause);
                self.handle_timer_cmd(PromptTransitionAnsweredNo);
            }
        }
    }

    fn handle_config_cmd(&mut self, cmd: ConfigCmd) {
        match cmd {
            ConfigCmd::None => {}
        }
    }

    fn should_auto_next(&self) -> bool {
        let timer = &self.conf().pomodoro.timer;
        match self.pomo().mode() {
            Mode::Focus => timer.auto_focus,
            Mode::LongBreak => timer.auto_long,
            Mode::ShortBreak => timer.auto_short,
        }
    }

    fn save_settings(&mut self) {
        match self.conf().save() {
            Ok(()) => {
                self.update_settings(SettingsMsg::SetUnsavedChanges(false));
                self.state.snapshot_settings();
                self.show_toast("Settings saved!", ToastType::Success);
            }
            Err(e) => {
                self.show_toast(format!("Failed to save: {e}"), ToastType::Error);
            }
        }
    }

    fn quit(&mut self) {
        self.router_mut().navigate(Navigation::Quit);
    }
}

// _________________________________________________________
//  ___ _____ _ _____ ___
// / __|_   _/_\_   _| __|
// \__ \ | |/ _ \| | | _|
// |___/ |_/_/ \_\_| |___|
// ---------------------------------------------------------

impl TuiRunner {
    // Toast
    fn toast_mut(&mut self) -> &mut ToastHandler {
        self.redraw();
        &mut self.state.toast
    }

    // Router
    fn router(&self) -> &Router {
        &self.state.router
    }
    fn router_mut(&mut self) -> &mut Router {
        self.redraw();
        &mut self.state.router
    }
    // fn update_router(&mut self, msg: RouterMsg) -> RouterCmd { self.state.router.update(msg) }

    // Timer
    fn timer(&self) -> &TimerModel {
        &self.state.timer.model
    }
    fn timer_mut(&mut self) -> &mut TimerModel {
        self.redraw();
        &mut self.state.timer.model
    }
    fn update_timer(&mut self, msg: TimerMsg) {
        let cmds = self.timer_mut().update(msg);
        for cmd in cmds {
            self.handle_timer_cmd(cmd);
        }
    }

    fn pomo(&self) -> &Pomodoro {
        &self.state.timer.pomo
    }
    fn pomo_mut(&mut self) -> &mut Pomodoro {
        self.redraw();
        &mut self.state.timer.pomo
    }
    fn update_pomo(&mut self, msg: PomodoroMsg) {
        let cmds = self.pomo_mut().update(msg);
        for cmd in cmds {
            self.handle_pomodoro_cmd(cmd);
        }
    }

    // Settings
    fn settings(&self) -> &SettingsModel {
        &self.state.settings.model
    }
    fn settings_mut(&mut self) -> &mut SettingsModel {
        self.redraw();
        &mut self.state.settings.model
    }
    fn update_settings(&mut self, msg: SettingsMsg) {
        let cmds = self.settings_mut().update(msg);
        for cmd in cmds {
            self.handle_settings_cmd(cmd);
        }
    }

    fn conf(&self) -> &Config {
        &self.state.settings.conf
    }
    fn conf_mut(&mut self) -> &mut Config {
        self.redraw();
        &mut self.state.settings.conf
    }
    fn update_conf(&mut self, msg: ConfigMsg) {
        let cmds = self.conf_mut().update(msg);
        for cmd in cmds {
            self.handle_config_cmd(cmd);
        }
    }

    fn redraw(&mut self) {
        self.redraw = true
    }

    fn take_redraw(&mut self) -> bool {
        if self.redraw {
            self.redraw = false;
            true
        } else {
            false
        }
    }
}

// ---------------------------------------------------------
//  ___ ___ ___ ___ ___ ___  _  _
// / __| __/ __/ __|_ _/ _ \| \| |
// \__ \ _|\__ \__ \| | (_) | .` |
// |___/___|___/___/___\___/|_|\_|
// ---------------------------------------------------------

// TODO: Handle errs
impl TuiRunner {
    fn session_new(&mut self) {
        if self.active_session.is_some() {
            self.session_stop();
        }
        self.active_session = Some(
            self.repo
                .session()
                .new_session(None, self.pomo().mode().into())
                .unwrap(),
        );
        debug!("session: new: {:?}", self.active_session);
    }

    fn session_tick(&self) {
        if let Some(ses) = &self.active_session {
            debug!("session: update: id={}", ses.id);
            let _ = self.repo.session().update(ses.id);
        }
    }

    fn session_stop(&mut self) {
        if let Some(ses) = self.active_session.take() {
            debug!("session: stop: id={}", ses.id);
            let _ = self.repo.session().end_session(ses.id);
        }
    }
}

// ---------------------------------------------------------
//  _____ ___ ___ _  __
// |_   _|_ _/ __| |/ /
//   | |  | | (__| ' <
//   |_| |___\___|_|\_\
// ---------------------------------------------------------

struct TickTimer {
    last_tick: Instant,
    tick_rate: Duration,
}

impl TickTimer {
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

    fn time_until_next(&self) -> Duration {
        let elapsed = Instant::now().duration_since(self.last_tick);
        self.tick_rate.saturating_sub(elapsed)
    }
}

impl Default for TickTimer {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}

impl From<ToastType> for ToasterType {
    fn from(value: ToastType) -> Self {
        match value {
            ToastType::Error => ToasterType::Error,
            ToastType::Warning => ToasterType::Warning,
            ToastType::Success => ToasterType::Success,
        }
    }
}

impl From<Mode> for PomodoroState {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Focus => Self::Focus,
            Mode::LongBreak => Self::LongBreak,
            Mode::ShortBreak => Self::ShortBreak,
        }
    }
}
