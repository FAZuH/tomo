use std::thread::sleep;
use std::time::Duration;

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
use crate::ui::Input;
use crate::ui::Navigation;
use crate::ui::Page;
use crate::ui::app::App;
use crate::ui::app::AppBuildError;
use crate::ui::tui::TuiError;
use crate::ui::tui::input::InputMapper;
use crate::ui::tui::input::TimerInputMapper;
use crate::ui::tui::input::commit_settings_edit;
use crate::ui::tui::renderer::TuiRenderer;
use crate::ui::tui::renderer::TuiSettingsRenderer;
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
            .timer_view(Box::new(TuiTimerView::new()))
            .settings_view(Box::new(TuiSettingsView::new()))
            .build()?;

        Ok(Self { app })
    }

    pub fn run(&mut self) -> Result<(), TuiError> {
        enable_raw_mode().map_err(TuiError::from)?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(TuiError::from)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout)).map_err(TuiError::from)?;

        let mut renderer = TuiRenderer::new();

        let res = self.run_loop(&mut terminal, &mut renderer);

        // Unconditionally ignore errors for cleanup
        let _ = disable_raw_mode();
        let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
        let _ = terminal.show_cursor();

        res
    }

    fn run_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
        renderer: &mut TuiRenderer,
    ) -> Result<(), TuiError> {
        loop {
            self.app.tick()?;

            let cmds = self.app.render();
            terminal
                .draw(|f| renderer.flush(f, cmds))
                .map_err(TuiError::from)?;

            if event::poll(Duration::from_millis(100)).map_err(TuiError::from)?
                && let Event::Key(key) = event::read().map_err(TuiError::from)?
                && let Some(input) = Input::from_keyevent(key)
            {
                // Handle settings input directly on the renderer
                if matches!(self.app.active_page(), Page::Settings) {
                    self.handle_settings_input(input, renderer)?;
                } else if let Some(action) = TimerInputMapper::new().into_action(input) {
                    let nav = self.app.handle_timer(action)?;
                    if matches!(nav, Navigation::Quit) {
                        break;
                    }
                    self.app.navigate(nav);
                }
            }
            sleep(Duration::from_millis(100));
        }
        Ok(())
    }

    /// Handle settings page input directly, mutating renderer state
    fn handle_settings_input(
        &mut self,
        input: Input,
        renderer: &mut TuiRenderer,
    ) -> Result<(), TuiError> {
        use Input::*;

        let settings = &mut renderer.settings;

        // When editing, handle text input
        if settings.is_editing() {
            return self.handle_settings_edit_input(input, settings);
        }

        // When navigating, handle navigation input
        match input {
            Up | Char('k') => {
                settings.select_up();
            }
            Down | Char('j') => {
                settings.select_down();
            }
            Enter => {
                settings.start_editing();
            }
            Esc => {
                self.app.navigate(Navigation::GoTo(Page::Timer));
            }
            Char('q') => {
                return Err(TuiError::from(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "quit",
                )));
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_settings_edit_input(
        &mut self,
        input: Input,
        settings: &mut TuiSettingsRenderer,
    ) -> Result<(), TuiError> {
        use Input::*;

        match input {
            Esc => {
                settings.cancel_editing();
            }
            Enter => {
                if let Some(action) = commit_settings_edit(settings) {
                    let nav = self.app.handle_settings(action)?;
                    if matches!(nav, Navigation::Quit) {
                        return Err(TuiError::from(std::io::Error::new(
                            std::io::ErrorKind::Interrupted,
                            "quit",
                        )));
                    }
                    self.app.navigate(nav);
                }
            }
            Backspace => {
                settings.pop_char();
            }
            Char(c) if c.is_ascii_digit() || c == ':' => {
                settings.push_char(c);
            }
            _ => {}
        }

        Ok(())
    }
}

impl From<&Config> for Pomodoro {
    fn from(value: &Config) -> Self {
        let timer = value.pomodoro.timer.clone();
        Self::new(timer.focus, timer.long, timer.short, timer.long_interval)
    }
}
