use crate::config::Config;
use crate::models::Pomodoro;
use crate::models::pomodoro::PomodoroError;
use crate::ui::FromInput;
use crate::ui::Input;
use crate::ui::Navigation;
use crate::ui::Page;
use crate::ui::controller::HomeController;
use crate::ui::controller::SettingsController;
use crate::ui::controller::TimerController;
use crate::ui::view::HomeView;
use crate::ui::view::HomeViewActions;
use crate::ui::view::RenderCommand;
use crate::ui::view::SettingsView;
use crate::ui::view::TimerView;
use crate::ui::view::TimerViewActions;

#[derive(Debug, thiserror::Error)]
pub enum AppBuildError {
    #[error("missing config")]
    MissingConfig,
    #[error("missing pomodoro")]
    MissingPomodoro,
    #[error("missing home_view")]
    MissingHomeView,
    #[error("missing timer_view")]
    MissingTimerView,
    #[error("missing settings_view")]
    MissingSettingsView,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Pomodoro(#[from] PomodoroError),
}

pub struct App {
    active_page: Page,
    home: HomeController,
    timer: TimerController,
    settings: SettingsController,
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub fn handle(&mut self, input: Input) -> Result<Navigation, AppError> {
        match self.active_page {
            Page::Home => match HomeViewActions::from_input(input) {
                Some(action) => Ok(self.home.handle(action)),
                None => Ok(Navigation::Stay),
            },
            Page::Timer => match TimerViewActions::from_input(input) {
                Some(action) => Ok(self.timer.handle(action)?),
                None => Ok(Navigation::Stay),
            },
            Page::Settings => todo!(),
        }
    }

    pub fn navigate(&mut self, nav: Navigation) {
        if let Navigation::GoTo(page) = nav {
            self.active_page = page;
        }
    }

    pub fn tick(&mut self) -> Result<(), PomodoroError> {
        self.timer.tick()
    }

    pub fn render(&self) -> Vec<RenderCommand> {
        let cmd = match self.active_page {
            Page::Home => RenderCommand::Home(self.home.render()),
            Page::Timer => RenderCommand::Timer(self.timer.render()),
            Page::Settings => RenderCommand::Settings(self.settings.render()),
        };
        vec![cmd]
    }
}

#[derive(Default)]
pub struct AppBuilder {
    config: Option<Config>,
    pomodoro: Option<Pomodoro>,
    page: Option<Page>,

    home_view: Option<Box<dyn HomeView>>,
    timer_view: Option<Box<dyn TimerView>>,
    settings_view: Option<Box<dyn SettingsView>>,
}

impl AppBuilder {
    pub fn build(self) -> Result<App, AppBuildError> {
        Ok(App {
            active_page: self.page.unwrap_or(Page::Home),
            home: HomeController::new(self.home_view.ok_or(AppBuildError::MissingHomeView)?),
            timer: TimerController::new(
                self.timer_view.ok_or(AppBuildError::MissingTimerView)?,
                self.pomodoro.ok_or(AppBuildError::MissingPomodoro)?,
            ),
            settings: SettingsController::new(
                self.settings_view
                    .ok_or(AppBuildError::MissingSettingsView)?,
                self.config.ok_or(AppBuildError::MissingConfig)?,
            ),
        })
    }

    pub fn config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    pub fn pomodoro(mut self, pomodoro: Pomodoro) -> Self {
        self.pomodoro = Some(pomodoro);
        self
    }

    pub fn page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    pub fn home_view(mut self, view: Box<dyn HomeView>) -> Self {
        self.home_view = Some(view);
        self
    }

    pub fn timer_view(mut self, view: Box<dyn TimerView>) -> Self {
        self.timer_view = Some(view);
        self
    }

    pub fn settings_view(mut self, view: Box<dyn SettingsView>) -> Self {
        self.settings_view = Some(view);
        self
    }
}

impl TryFrom<Input> for TimerViewActions {
    type Error = ();

    fn try_from(value: Input) -> Result<TimerViewActions, ()> {
        use Input::*;
        use TimerViewActions::*;
        let ret = match value {
            Left => Sub1Min,
            Down => Sub5Min,
            Right => Add1Min,
            Up => Add5Min,
            Char(' ') => TogglePause,
            Backspace => ResetSession,
            Enter => SkipSession,
            _ => return Err(()),
        };
        Ok(ret)
    }
}
