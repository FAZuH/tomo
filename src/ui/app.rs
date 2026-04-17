use crate::config::Config;
use crate::models::pomodoro::PomodoroError;
use crate::models::Pomodoro;
use crate::ui::controller::SettingsController;
use crate::ui::controller::TimerController;
use crate::ui::view::RenderCommand;
use crate::ui::view::SettingsView;
use crate::ui::view::SettingsViewActions;
use crate::ui::view::TimerView;
use crate::ui::view::TimerViewActions;
use crate::ui::Navigation;
use crate::ui::Page;

pub struct App<I> {
    active_page: Page,
    timer: TimerController,
    settings: SettingsController,

    timer_inputmap: Box<dyn InputMapper<I, TimerViewActions>>,
    settings_inputmap: Box<dyn InputMapper<I, SettingsViewActions>>,
}

impl<I> App<I> {
    pub fn builder() -> AppBuilder<I> {
        AppBuilder::default()
    }

    pub fn handle(&mut self, input: I) -> Result<Navigation, AppError> {
        match self.active_page {
            Page::Timer => match self.timer_inputmap.into_action(input) {
                Some(action) => Ok(self.timer.handle(action)?),
                None => Ok(Navigation::Stay),
            },
            Page::Settings => match self.settings_inputmap.into_action(input) {
                Some(action) => Ok(self.settings.handle(action)?),
                None => Ok(Navigation::Stay),
            },
        }
    }

    /// Handle settings actions directly (bypasses input mapper)
    pub fn handle_settings_action(
        &mut self,
        action: SettingsViewActions,
    ) -> Result<Navigation, AppError> {
        Ok(self.settings.handle(action)?)
    }

    pub fn navigate(&mut self, nav: Navigation) {
        if let Navigation::GoTo(page) = nav {
            self.active_page = page;
        }
    }

    pub fn active_page(&self) -> Page {
        self.active_page
    }

    pub fn tick(&mut self) -> Result<(), AppError> {
        self.timer.tick();
        Ok(())
    }

    pub fn render(&self) -> Vec<RenderCommand> {
        let cmd = match self.active_page {
            Page::Timer => RenderCommand::Timer(self.timer.render()),
            Page::Settings => RenderCommand::Settings(self.settings.render()),
        };
        vec![cmd]
    }
}

pub trait InputMapper<I, A> {
    fn into_action(&mut self, input: I) -> Option<A>;
}

pub struct AppBuilder<I> {
    config: Option<Config>,
    pomodoro: Option<Pomodoro>,
    page: Option<Page>,

    timer_view: Option<Box<dyn TimerView>>,
    settings_view: Option<Box<dyn SettingsView>>,

    timer_inputmap: Option<Box<dyn InputMapper<I, TimerViewActions>>>,
    settings_inputmap: Option<Box<dyn InputMapper<I, SettingsViewActions>>>,
}

impl<I> AppBuilder<I> {
    pub fn build(self) -> Result<App<I>, AppBuildError> {
        use AppBuildError::*;
        Ok(App {
            active_page: self.page.unwrap_or(Page::Timer),
            timer: TimerController::new(
                self.timer_view.ok_or(MissingTimerView)?,
                self.pomodoro.ok_or(MissingPomodoro)?,
            ),
            settings: SettingsController::new(
                self.settings_view.ok_or(MissingSettingsView)?,
                self.config.ok_or(MissingConfig)?,
            ),
            timer_inputmap: self.timer_inputmap.ok_or(MissingTimerMapper)?,
            settings_inputmap: self.settings_inputmap.ok_or(MissingSettingsMapper)?,
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

    pub fn timer_view(mut self, view: Box<dyn TimerView>) -> Self {
        self.timer_view = Some(view);
        self
    }

    pub fn settings_view(mut self, view: Box<dyn SettingsView>) -> Self {
        self.settings_view = Some(view);
        self
    }

    pub fn timer_inputmap(mut self, inputmap: Box<dyn InputMapper<I, TimerViewActions>>) -> Self {
        self.timer_inputmap = Some(inputmap);
        self
    }

    pub fn settings_inputmap(
        mut self,
        inputmap: Box<dyn InputMapper<I, SettingsViewActions>>,
    ) -> Self {
        self.settings_inputmap = Some(inputmap);
        self
    }
}

impl<I> Default for AppBuilder<I> {
    fn default() -> Self {
        Self {
            config: None,
            pomodoro: None,
            page: None,
            timer_view: None,
            settings_view: None,
            timer_inputmap: None,
            settings_inputmap: None,
        }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum AppBuildError {
    #[error("missing config")]
    MissingConfig,
    #[error("missing pomodoro")]
    MissingPomodoro,

    #[error("missing timer_view")]
    MissingTimerView,
    #[error("missing settings_view")]
    MissingSettingsView,

    #[error("missing timer_inputmap")]
    MissingTimerMapper,
    #[error("missing settings_inputmap")]
    MissingSettingsMapper,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Pomodoro(#[from] PomodoroError),
}
