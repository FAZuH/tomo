use std::collections::HashMap;
use std::time::Duration;

use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::Gauge;
use ratatui::widgets::Paragraph;
use tui_widgets::popup::Popup;

use crate::models::Pomodoro;
use crate::models::pomodoro::State;
use crate::utils;

pub struct TuiTimerRenderer {
    prompt_next_session: bool,
    layout: Layout,
    paused_p: Paragraph<'static>,
    paused_width: u16,
    state_labels: HashMap<State, (String, u16)>,
}

impl TuiTimerRenderer {
    pub fn new() -> Self {
        // Pre-compute paused text
        let paused_text = utils::ascii_future(" ( PAUSED )");
        let paused_width = utils::string_width(&paused_text) as u16;
        let paused_p = Paragraph::new(paused_text).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

        // Pre-compute state labels
        let mut state_labels = HashMap::new();
        for (state, text) in [
            (State::Focus, "FOCUS"),
            (State::ShortBreak, "SHORT BREAK"),
            (State::LongBreak, "LONG BREAK"),
        ] {
            let label = utils::ascii_future(text);
            let width = utils::string_width(&label) as u16;
            state_labels.insert(state, (label, width));
        }

        Self {
            prompt_next_session: false,
            layout: Self::layout(),
            paused_p,
            paused_width,
            state_labels,
        }
    }

    pub fn render(&self, frame: &mut Frame, pomodoro: &Pomodoro) {
        let state = pomodoro.state();
        let timer = pomodoro.remaining_time();
        let pause_indicator = !pomodoro.is_running();
        let total_time = pomodoro.session_duration();
        let progress = if total_time.as_secs() > 0 {
            1.0 - (timer.as_secs_f64() / total_time.as_secs_f64())
        } else {
            0.0
        };

        let rows = self.layout.split(frame.area());

        self.state(frame, rows[1], state, pause_indicator);

        self.timer(frame, rows[3], &timer, session_color(state));

        self.progress_bar(frame, rows[4], progress, session_color(state));

        self.stats(
            frame,
            rows[6],
            pomodoro.long_interval(),
            pomodoro.total_sessions(),
            pomodoro.focus_sessions(),
        );

        self.shortcuts(frame, rows[8]);

        self.prompt(frame, pomodoro);
    }

    pub fn set_prompt_next_session(&mut self, val: bool) {
        self.prompt_next_session = val;
    }

    pub fn prompt_next_session(&self) -> bool {
        self.prompt_next_session
    }

    // Render popup if prompt is active
    fn prompt(&self, frame: &mut Frame, model: &Pomodoro) {
        if self.prompt_next_session {
            let next = model.next_state().to_string().to_lowercase();
            let body = Text::from(vec![
                Line::from(""),
                Line::from(""),
                Line::from(format!("     start {next} session?     ")).alignment(Alignment::Center),
                Line::from(""),
                Line::from(""),
                Line::from("              ").alignment(Alignment::Center),
                Line::from(vec![
                    Span::from("       "),
                    Span::styled(
                        "  y/enter: Yes  ",
                        Style::new().fg(Color::DarkGray).bg(Color::Green),
                    ),
                    Span::from("   "),
                    Span::styled(
                        "  n/esc: No  ",
                        Style::new().fg(Color::DarkGray).bg(Color::Red),
                    ),
                    Span::from("       "),
                ])
                .alignment(Alignment::Center),
                Line::from(""),
                Line::from(""),
            ])
            .alignment(Alignment::Center);
            let prompt_popup = Popup::new(body)
                .border_style(Style::new().fg(Color::Yellow))
                .border_set(border::ROUNDED);

            frame.render_widget(prompt_popup, frame.area());
        }
    }

    fn state(&self, frame: &mut Frame, area: Rect, state: State, paused: bool) {
        let (label, label_width) = &self.state_labels[&state];
        let color = match state {
            State::Focus => Color::LightRed,
            State::ShortBreak => Color::LightGreen,
            State::LongBreak => Color::LightCyan,
        };
        let center = Alignment::Center;

        if paused {
            let [area_label, area_paused] = Layout::horizontal([
                Constraint::Length(*label_width),
                Constraint::Length(self.paused_width),
            ])
            .flex(Flex::Center)
            .areas::<2>(area);

            let p_label = Paragraph::new(label.as_str())
                .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
                .alignment(center);

            frame.render_widget(p_label, area_label);
            frame.render_widget(&self.paused_p, area_paused);
        } else {
            let p = Paragraph::new(label.as_str())
                .style(Style::default().fg(color))
                .alignment(center);
            frame.render_widget(p, area);
        }
    }

    fn timer(&self, frame: &mut Frame, area: Rect, remaining: &Duration, color: Color) {
        let time_str = format_duration_clock(remaining);
        let ascii = utils::ascii_mono12(&time_str);
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
            Span::styled(" ", bright),
            Span::styled(": ±30s", dim),
            sep.clone(),
            Span::styled("", bright),
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
            Constraint::Length(3), // state
            Constraint::Length(2),
            Constraint::Length(9), // timer
            Constraint::Length(1), // progress_bar
            Constraint::Length(1),
            Constraint::Length(1), // stats
            Constraint::Length(1),
            Constraint::Length(1), // shortcuts
            Constraint::Fill(1),
        ])
    }
}

fn format_duration_clock(d: &Duration) -> String {
    let secs = d.as_secs();
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

fn session_color(state: State) -> Color {
    match state {
        State::Focus => Color::LightBlue,
        State::ShortBreak => Color::LightGreen,
        State::LongBreak => Color::LightCyan,
    }
}
