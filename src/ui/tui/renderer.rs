use std::time::Duration;

use ratatui::layout::Constraint;
use ratatui::layout::Flex;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::widgets::Gauge;
use ratatui::widgets::Paragraph;

use crate::models::pomodoro::PomodoroState;
use crate::ui::view::RenderCommand;
use crate::ui::view::SettingsRenderCommand;
use crate::ui::view::TimerRenderCommand;
use crate::utils;

struct TuiTimerRenderer {
    layout: Layout,
    paused_p: Paragraph<'static>,
    paused_p_length: u16,
}

impl TuiTimerRenderer {
    pub fn new() -> Self {
        let (paused_p, paused_p_length) = Self::paused_paragraph();
        Self {
            layout: Self::layout(), 
            paused_p,
            paused_p_length,
        }
    }
    fn render(&self, frame: &mut Frame, cmds: Vec<TimerRenderCommand>) {
        let mut state = None;
        let mut timer = None;
        let mut pause_indicator = false;
        let mut stats = None;
        let mut progress = 0.0;

        use TimerRenderCommand::*;
        for cmd in &cmds {
            match cmd {
                State(s) => state = Some(*s),
                Timer { remaining: r } => timer = Some(*r),
                Progress(p) => progress = *p,
                PauseIndicator(p) => pause_indicator = *p,
                Stats { .. } => stats = Some(cmd.clone()),
            }
        }

        let rows = self.layout.split(frame.area());

        if let Some(state) = state {
            self.state(frame, rows[1], state, pause_indicator);
        }

        if let Some(remaining) = timer {
            self.timer(
                frame,
                rows[3],
                &remaining,
                state.map(session_color).unwrap_or(Color::White),
            );
        }

        self.progress_bar(
            frame,
            rows[4],
            progress,
            state.map(session_color).unwrap_or(Color::Cyan),
        );

        if let Some(Stats {
            long_interval,
            total_sessions,
            focus_sessions,
            ..
        }) = stats
        {
            self.stats(
                frame,
                rows[6],
                long_interval,
                total_sessions,
                focus_sessions,
            );
        }

        self.shortcuts(frame, rows[8]);
    }

    fn state(&self, frame: &mut Frame, area: Rect, state: PomodoroState, paused: bool) {
        // TODO: Pre-compute this and store ascii instead
        let (label, color) = match state {
            PomodoroState::Focus => ("FOCUS", Color::LightRed),
            PomodoroState::ShortBreak => ("SHORT BREAK", Color::LightGreen),
            PomodoroState::LongBreak => ("LONG BREAK", Color::LightCyan),
        };
        let label = utils::ascii_future(label);
        let center = Alignment::Center;

        if paused {

            let [area_label, area_paused] = Layout::horizontal([
                Constraint::Length(utils::string_width(&label) as u16),
                Constraint::Length(67),
            ])
            .flex(Flex::Center)
            .areas::<2>(area);

            let p_label = Paragraph::new(label)
                .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
                .alignment(center);

            frame.render_widget(&p_label, area_label);
            frame.render_widget(&self.paused_p, area_paused);
        } else {
            let p = Paragraph::new(label)
                .style(Style::default().fg(color))
                .alignment(center);
            frame.render_widget(p, area);
        }
    }

    fn timer(&self, frame: &mut Frame, area: Rect, remaining: &Duration, color: Color) {
        let time_str = format_duration_clock(remaining);
        let ascii = utils::ascii_mono12(time_str);

        let width = utils::string_width(&ascii) as u16;
        let height = utils::string_height(&ascii) as u16;
        let area = area.centered(Constraint::Length(width), Constraint::Length(height));

        let p = Paragraph::new(ascii)
            .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(p, area);
    }

    fn progress_bar(&self, frame: &mut Frame, area: Rect, progress: f64, color: Color) {
        let gauge = Gauge::default()
            .ratio(progress.clamp(0.0, 1.0))
            .use_unicode(true)
            .gauge_style(Style::default().fg(color));
        let layout = Layout::horizontal([Constraint::Length(55)]).flex(Flex::Center);
        let area = area.layout::<1>(&layout)[0];
        frame.render_widget(gauge, area);
    }

    fn stats(
        &self,
        frame: &mut Frame,
        area: Rect,
        long_interval: u32,
        total_sessions: u32,
        focus_sessions: u32,
    ) {
        let dim = Style::default().dim();
        let bright = Style::default();
        let line = Line::from(vec![
            Span::styled("Focused: ", dim),
            Span::styled(focus_sessions.to_string(), bright),
            Span::styled("  │  Sessions: ", dim),
            Span::styled(total_sessions.to_string(), bright),
            Span::styled("  │  Long break every: ", dim),
            Span::styled(long_interval.to_string(), bright),
        ]);
        let p = Paragraph::new(line).alignment(Alignment::Center);
        frame.render_widget(p, area);
    }

    fn shortcuts(&self, frame: &mut Frame, area: Rect) {
        let dim = Style::default().dim();
        let bright = Style::default();
        let sep = Span::styled(" • ", dim);
        let line = Line::from(vec![
            Span::styled("Space", bright),
            Span::styled(": Pause", dim),
            sep.clone(),
            Span::styled("Enter", bright),
            Span::styled(": Skip", dim),
            sep.clone(),
            Span::styled("Backspace", bright),
            Span::styled(": Reset", dim),
            sep.clone(),
            Span::styled("←→", bright),
            Span::styled(": ±30s", dim),
            sep.clone(),
            Span::styled("↑↓", bright),
            Span::styled(": ±1m", dim),
            sep.clone(),
            Span::styled("q", bright),
            Span::styled(": Quit", dim),
        ]);
        let p = Paragraph::new(line).alignment(Alignment::Center);
        frame.render_widget(p, area);
    }

    fn layout() -> Layout {
        Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),  // state
            Constraint::Length(2),
            Constraint::Length(9),  // timer
            Constraint::Length(1),  // progress_bar
            Constraint::Length(1),
            Constraint::Length(1),  // stats
            Constraint::Length(1),
            Constraint::Length(1),  // shortcuts
            Constraint::Fill(1),
        ])
    }

    fn paused_paragraph() -> (Paragraph<'static>, u16) {
        let label = utils::ascii_future(" ( PAUSED )");
        let width = utils::string_width(&label) as u16;
        let p = Paragraph::new(label)
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
        (p, width)
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

fn session_color(state: PomodoroState) -> Color {
    match state {
        PomodoroState::Focus => Color::LightBlue,
        PomodoroState::ShortBreak => Color::LightGreen,
        PomodoroState::LongBreak => Color::LightCyan,
    }
}
