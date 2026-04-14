use std::time::Duration;

use figlet_rs::Toilet;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::APP_NAME;
use crate::models::pomodoro::PomodoroState;
use crate::ui::view::HomeRenderCommand;
use crate::ui::view::RenderCommand;
use crate::ui::view::SettingsRenderCommand;
use crate::ui::view::TimerRenderCommand;

struct TuiHomeRenderer;

impl TuiHomeRenderer {
    fn render(&self, frame: &mut Frame, cmds: Vec<HomeRenderCommand>) {
        for cmd in cmds {
            match cmd {
                HomeRenderCommand::WelcomeText => self.render_welcome(frame),
                HomeRenderCommand::NavButton(label, page) => todo!(),
            }
        }
    }

    fn render_welcome(&self, frame: &mut Frame) {
        let ascii = ascii_font()
            .convert(&format!("Welcome to {APP_NAME}!"))
            .unwrap()
            .to_string();
        let area = centered_ascii(&ascii, frame.area());
        let p = Paragraph::new(ascii);
        frame.render_widget(p, area);
    }
}

struct TuiTimerRenderer;

impl TuiTimerRenderer {
    fn render(&self, frame: &mut Frame, cmds: Vec<TimerRenderCommand>) {
        for cmd in cmds {
            match cmd {
                TimerRenderCommand::TimerDisplay { remaining, total } => {
                    self.display(frame, remaining, total)
                }
                TimerRenderCommand::SessionLabel(state) => self.session_label(frame, state),
                TimerRenderCommand::PauseIndicator(paused) => self.pause_indicator(frame, paused),
                TimerRenderCommand::ProgressBar(progress) => self.progress_bar(frame, progress),
            }
        }
    }

    fn display(&self, frame: &mut Frame, remaining: Duration, total: Duration) {
        let widget = Paragraph::new(format!("{remaining:?}, {total:?}"));
        frame.render_widget(widget, frame.area());
    }

    fn session_label(&self, frame: &mut Frame, state: PomodoroState) {
        let widget = Paragraph::new(format!("{state:?}"));
        frame.render_widget(widget, frame.area());
    }

    fn pause_indicator(&self, frame: &mut Frame, paused: bool) {
        let widget = Paragraph::new(format!("{paused:?}"));
        frame.render_widget(widget, frame.area());
    }

    fn progress_bar(&self, frame: &mut Frame, progress: f64) {
        let widget = Paragraph::new(format!("{progress:?}"));
        frame.render_widget(widget, frame.area());
    }
}

struct TuiSettingsRenderer;

impl TuiSettingsRenderer {
    fn render(&self, frame: &mut Frame, cmds: Vec<SettingsRenderCommand>) {
        for cmd in cmds {
            match cmd {
                SettingsRenderCommand::SettingsHeader => todo!(),
                SettingsRenderCommand::SettingsField { label, value } => todo!(),
            }
        }
    }
}

pub struct TuiRenderer {
    home: TuiHomeRenderer,
    timer: TuiTimerRenderer,
    settings: TuiSettingsRenderer,
}

impl TuiRenderer {
    pub fn new() -> Self {
        Self {
            home: TuiHomeRenderer,
            timer: TuiTimerRenderer,
            settings: TuiSettingsRenderer,
        }
    }

    pub fn flush(&self, frame: &mut Frame, commands: Vec<RenderCommand>) {
        for cmd in commands {
            match cmd {
                RenderCommand::Home(cmds) => self.home.render(frame, cmds),
                RenderCommand::Timer(cmds) => self.timer.render(frame, cmds),
                RenderCommand::Settings(cmds) => self.settings.render(frame, cmds),
            }
        }
    }
}

fn centered_ascii(ascii: &str, area: Rect) -> Rect {
    let lines = ascii.lines().count() as u16;
    let cols = ascii.lines().map(|l| l.len()).max().unwrap_or(0) as u16;

    let area = centered_rect(cols, lines, area);
    area
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(width),
        Constraint::Fill(1),
    ])
    .split(vertical[1])[1]
}

fn ascii_font() -> Toilet {
    Toilet::mono12().unwrap()
}
