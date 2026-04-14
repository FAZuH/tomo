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
use crate::ui::app::App;
use crate::ui::app::AppBuildError;
use crate::ui::error::UiError;
use crate::ui::tui::TuiError;
use crate::ui::tui::renderer::TuiRenderer;
use crate::ui::tui::view::TuiHomeView;
use crate::ui::tui::view::TuiSettingsView;
use crate::ui::tui::view::TuiTimerView;

pub struct TuiRunner {
    app: App,
}

impl TuiRunner {
    pub fn new(config: Config, pomodoro: Pomodoro) -> Result<Self, AppBuildError> {
        let app = App::builder()
            .pomodoro(pomodoro)
            .config(config)
            .home_view(Box::new(TuiHomeView::new()))
            .timer_view(Box::new(TuiTimerView::new()))
            .settings_view(Box::new(TuiSettingsView::new()))
            .build()?;

        Ok(Self { app })
    }

    pub fn run(&mut self) -> Result<(), UiError> {
        enable_raw_mode().map_err(|e| UiError::from(TuiError::from(e)))?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(|e| UiError::from(TuiError::from(e)))?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))
            .map_err(|e| UiError::from(TuiError::from(e)))?;

        let renderer = TuiRenderer::new();

        loop {
            self.app.tick()?;
            let cmds = self.app.render();
            terminal
                .draw(|f| renderer.flush(f, cmds))
                .map_err(|e| UiError::from(TuiError::from(e)))?;

            if event::poll(Duration::from_millis(100))
                .map_err(|e| UiError::from(TuiError::from(e)))?
            {
                if let Event::Key(key) =
                    event::read().map_err(|e| UiError::from(TuiError::from(e)))?
                {
                    if let Some(input) = Input::from_keyevent(key) {
                        let nav = self.app.handle(input)?;
                        if matches!(nav, Navigation::Quit) {
                            break;
                        }
                        self.app.navigate(nav);
                    }
                }
            }
        }

        disable_raw_mode().map_err(|e| UiError::from(TuiError::from(e)))?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)
            .map_err(|e| UiError::from(TuiError::from(e)))?;

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
