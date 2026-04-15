use std::time::Duration;

use figlet_rs::Toilet;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::widgets::Gauge;
use ratatui::widgets::Paragraph;

use crate::models::pomodoro::PomodoroState;
use crate::ui::view::RenderCommand;
use crate::ui::view::SettingsRenderCommand;
use crate::ui::view::TimerRenderCommand;

struct TuiTimerRenderer;

impl TuiTimerRenderer {
    fn render(&self, frame: &mut Frame, cmds: Vec<TimerRenderCommand>) {
        let mut state = None;
        let mut timer = None;
        let mut pause_indicator = false;
        let mut stats = None;
        let mut progress = 0.0;

        for cmd in &cmds {
            match cmd {
                TimerRenderCommand::State(s) => state = Some(*s),
                TimerRenderCommand::Timer { remaining: r } => timer = Some(*r),
                TimerRenderCommand::Progress(p) => progress = *p,
                TimerRenderCommand::PauseIndicator(p) => pause_indicator = *p,
                TimerRenderCommand::Stats { .. } => stats = Some(cmd.clone()),
            }
        }

        let label_data = state.map(|s| {
            let (label, color) = match s {
                PomodoroState::Focus => ("FOCUS", Color::Red),
                PomodoroState::ShortBreak => ("SHORT BREAK", Color::Green),
                PomodoroState::LongBreak => ("LONG BREAK", Color::Blue),
            };
            let text = if pause_indicator {
                format!("{}\n\n  ( PAUSED )", label)
            } else {
                label.to_string()
            };
            let ascii = Toilet::future()
                .unwrap()
                .convert(&text)
                .unwrap()
                .to_string();
            let height = ascii.lines().count() as u16;
            (ascii, color, height)
        });

        let timer_data = timer.map(|d| {
            let time_str = format_duration_clock(&d);
            let ascii = Toilet::mono12()
                .unwrap()
                .convert(&time_str)
                .unwrap()
                .to_string();
            let width = ascii.lines().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
            let height = ascii.lines().count() as u16;
            (ascii, width, height)
        });

        let label_height = label_data.as_ref().map_or(0, |d| d.2);
        let timer_height = timer_data.as_ref().map_or(0, |d| d.2);

        let rows = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(label_height.max(1)),
            Constraint::Length(2),
            Constraint::Length(timer_height.max(1)),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(frame.area());

        if let Some((ascii, color, _)) = label_data {
            self.session_label(frame, rows[1], &ascii, color);
        }

        if let Some((ascii, width, height)) = timer_data {
            self.display(frame, rows[3], &ascii, width, height);
        }

        self.progress_bar(frame, rows[4], progress);

        if let Some(TimerRenderCommand::Stats {
            remaining,
            total,
            long_interval,
            total_sessions,
            focus_sessions,
        }) = stats
        {
            self.stats(
                frame,
                rows[6],
                &remaining,
                &total,
                long_interval,
                total_sessions,
                focus_sessions,
            );
        }

        self.shortcuts(frame, rows[8]);
    }

    fn display(&self, frame: &mut Frame, area: Rect, ascii: &str, width: u16, height: u16) {
        let area = area.centered(Constraint::Length(width), Constraint::Length(height));
        let p = Paragraph::new(ascii).alignment(Alignment::Center);
        frame.render_widget(p, area);
    }

    fn session_label(&self, frame: &mut Frame, area: Rect, ascii: &str, color: Color) {
        let p = Paragraph::new(ascii)
            .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(p, area);
    }

    fn progress_bar(&self, frame: &mut Frame, area: Rect, progress: f64) {
        let gauge = Gauge::default()
            .ratio(progress.clamp(0.0, 1.0))
            .gauge_style(Style::default().fg(Color::Cyan));
        frame.render_widget(gauge, area);
    }

    fn stats(
        &self,
        frame: &mut Frame,
        area: Rect,
        remaining: &Duration,
        total: &Duration,
        long_interval: u32,
        total_sessions: u32,
        focus_sessions: u32,
    ) {
        let text = format!(
            "{} / {}  │  Sessions: {}/{}  │  Long break every: {}",
            format_duration_human(remaining),
            format_duration_human(total),
            focus_sessions,
            total_sessions,
            long_interval,
        );
        let p = Paragraph::new(text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(p, area);
    }

    fn shortcuts(&self, frame: &mut Frame, area: Rect) {
        let text = "Space: Pause  Enter: Skip  Backspace: Reset  \u{2190}\u{2192}: \u{00b1}1m  \u{2191}\u{2193}: \u{00b1}5m  q: Quit";
        let p = Paragraph::new(text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(p, area);
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
    timer: TuiTimerRenderer,
    settings: TuiSettingsRenderer,
}

impl TuiRenderer {
    pub fn new() -> Self {
        Self {
            timer: TuiTimerRenderer,
            settings: TuiSettingsRenderer,
        }
    }

    pub fn flush(&self, frame: &mut Frame, commands: Vec<RenderCommand>) {
        for cmd in commands {
            match cmd {
                RenderCommand::Timer(cmds) => self.timer.render(frame, cmds),
                RenderCommand::Settings(cmds) => self.settings.render(frame, cmds),
            }
        }
    }
}

fn format_duration_clock(d: &Duration) -> String {
    let secs = d.as_secs();
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

fn format_duration_human(d: &Duration) -> String {
    let total_secs = d.as_secs();
    let hrs = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    match (hrs, mins, secs) {
        (0, 0, s) => format!("{s}s"),
        (0, m, 0) => format!("{m}m"),
        (0, m, s) => format!("{m}m{s}s"),
        (h, 0, 0) => format!("{h}h"),
        (h, m, 0) => format!("{h}h{m}m"),
        (h, m, s) => format!("{h}h{m}m{s}s"),
    }
}
