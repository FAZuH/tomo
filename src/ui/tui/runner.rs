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
use crate::models::Pomodoro;
use crate::ui::Input;
use crate::ui::Navigation;
use crate::ui::Page;
use crate::ui::app::App;
use crate::ui::tui::TuiError;
use crate::ui::tui::input::InputMapper;
use crate::ui::tui::input::TimerInputMapper;
use crate::ui::tui::input::commit_settings_edit;
use crate::ui::tui::renderer::TuiRenderer;
use crate::ui::tui::view::TuiSettingsView;
use crate::ui::tui::view::TuiTimerView;

pub struct TuiRunner {
    app: App,
    should_quit: bool,
    renderer: TuiRenderer,
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    needs_redraw: bool,
}

impl TuiRunner {
    pub fn new(config: Config, pomodoro: Pomodoro) -> Result<Self, TuiError> {
        let app = App::builder()
            .pomodoro(pomodoro)
            .config(config)
            .timer_view(Box::new(TuiTimerView::new()))
            .settings_view(Box::new(TuiSettingsView::new()))
            .build()
            .map_err(|e| TuiError::InitializeError(e.to_string()))?;

        let renderer = TuiRenderer::new();
        let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

        Ok(Self {
            app,
            should_quit: false,
            renderer,
            terminal,
            needs_redraw: true,
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
        let tick_rate = Duration::from_secs(1);

        while !self.should_quit {
            let now = Instant::now();
            if now.duration_since(last_tick) >= tick_rate {
                self.app.tick()?;
                last_tick = now;
                self.needs_redraw = true;
            }

            if self.needs_redraw {
                self.render_terminal()?;
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

    fn render_terminal(&mut self) -> Result<(), TuiError> {
        let cmds = self.app.render();
        self.terminal.draw(|f| self.renderer.flush(f, cmds))?;
        Ok(())
    }

    fn handle_key_event(&mut self, input: Input) -> Result<(), TuiError> {
        // Handle settings input directly on the renderer
        match self.app.active_page() {
            Page::Settings => self.handle_settings(input)?,
            Page::Timer => self.handle_timer(input)?,
        }
        Ok(())
    }

    fn handle_timer(&mut self, input: Input) -> Result<(), TuiError> {
        if let Some(action) = TimerInputMapper::new().into_action(input) {
            let nav = self.app.handle_timer(action)?;
            self.handle_nav(nav);
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
            Enter => settings.start_editing(),
            Esc => self.app.navigate(Navigation::GoTo(Page::Timer)),
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
            Enter => {
                if let Some(action) = commit_settings_edit(settings) {
                    let nav = self.app.handle_settings(action)?;
                    self.handle_nav(nav)
                }
            }
            Backspace => settings.pop_char(),
            Char(c) if c.is_ascii_digit() || c == ':' => {
                settings.push_char(c);
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_nav(&mut self, nav: Navigation) {
        if matches!(nav, Navigation::Quit) {
            self.quit();
        } else {
            self.app.navigate(nav);
        }
    }

    fn quit(&mut self) {
        self.should_quit = true
    }
}

impl From<&Config> for Pomodoro {
    fn from(value: &Config) -> Self {
        let timer = value.pomodoro.timer.clone();
        Self::new(timer.focus, timer.long, timer.short, timer.long_interval)
    }
}
