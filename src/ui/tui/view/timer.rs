use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;

use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::Gauge;
use ratatui::widgets::Paragraph;
use tui_widgets::popup::Popup;

use crate::models::Pomodoro;
use crate::models::pomodoro::Mode;
use crate::ui::tui::model::TimerModel;
use crate::utils;

type State = TimerState;
type Buf<'a> = &'a mut Buffer;

pub struct TuiTimerRenderer {}

impl TuiTimerRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct TimerState {
    pub(super) model: TimerModel,
    pub(super) pomo: Pomodoro,
}

impl TimerState {
    pub fn new(model: TimerModel, pomo: Pomodoro) -> Self {
        Self { model, pomo }
    }
}

impl StatefulWidget for TuiTimerRenderer {
    type State = State;

    fn render(self, area: Rect, buf: Buf, state: &mut State) {
        let TimerState { model, pomo } = state;

        let mode = pomo.mode();
        let timer = pomo.remaining_time();
        let paused = !pomo.is_running();
        let progress = pomo.progress();

        let rows = LAYOUT.split(area);

        self.state(rows[1], buf, mode, paused);
        self.timer(rows[3], buf, &timer, mode);
        self.progress_bar(rows[4], buf, progress, mode);
        self.stats(
            rows[6],
            buf,
            pomo.long_interval(),
            pomo.total_sessions(),
            pomo.focus_sessions(),
        );
        self.shortcuts(rows[8], buf);
        self.prompt(area, buf, model, pomo);
    }
}

impl TuiTimerRenderer {
    // Render popup if prompt is active
    fn prompt(&self, area: Rect, buf: Buf, model: &TimerModel, pomo: &Pomodoro) {
        if model.prompt_next_session() {
            let next = pomo.next_mode().to_string().to_lowercase();
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

            Widget::render(
                Popup::new(body)
                    .border_style(Style::new().fg(Color::Yellow))
                    .border_set(border::ROUNDED),
                area,
                buf,
            );
        }
    }

    fn state(&self, area: Rect, buf: Buf, mode: Mode, paused: bool) {
        let (label, label_width) = &STATE_LABELS[&mode];
        let color = mode.into();
        let center = Alignment::Center;

        if paused {
            let [area_label, area_paused] = Layout::horizontal([
                Constraint::Length(*label_width),
                Constraint::Length(*PAUSED_WIDTH),
            ])
            .flex(Flex::Center)
            .areas::<2>(area);

            Paragraph::new(label.as_str())
                .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
                .alignment(center)
                .render(area_label, buf);
            PAUSED_PARAGRAPH.clone().render(area_paused, buf);
        } else {
            Paragraph::new(label.as_str())
                .style(Style::default().fg(color))
                .alignment(center)
                .render(area, buf);
        }
    }

    fn timer(&self, area: Rect, buf: Buf, remaining: &Duration, color: impl Into<Color>) {
        let time_str = format_duration_clock(remaining);
        let ascii = utils::ascii_mono12(&time_str);
        let width = utils::string_width(&ascii) as u16;
        let height = utils::string_height(&ascii) as u16;
        let area = area.centered(Constraint::Length(width), Constraint::Length(height));

        Paragraph::new(ascii)
            .style(
                Style::default()
                    .fg(color.into())
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .render(area, buf);
    }

    fn progress_bar(&self, area: Rect, buf: Buf, progress: f64, color: impl Into<Color>) {
        let layout = Layout::horizontal([Constraint::Length(55)]).flex(Flex::Center);
        let area = area.layout::<1>(&layout)[0];

        Gauge::default()
            .ratio(progress.clamp(0.0, 1.0))
            .use_unicode(true)
            .gauge_style(Style::default().fg(color.into()))
            .render(area, buf);
    }

    fn stats(
        &self,
        area: Rect,
        buf: Buf,
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

        Paragraph::new(line)
            .alignment(Alignment::Center)
            .render(area, buf);
    }

    fn shortcuts(&self, area: Rect, buf: Buf) {
        SHORTCUTS.clone().render(area, buf);
    }
}

fn format_duration_clock(d: &Duration) -> String {
    let secs = d.as_secs();
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

impl From<Mode> for Color {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Focus => Color::LightBlue,
            Mode::ShortBreak => Color::LightGreen,
            Mode::LongBreak => Color::LightCyan,
        }
    }
}

static LAYOUT: LazyLock<Layout> = LazyLock::new(|| {
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
});

static STATE_LABELS: LazyLock<HashMap<Mode, (String, u16)>> = LazyLock::new(|| {
    let mut ret = HashMap::new();
    for (state, text) in [
        (Mode::Focus, "FOCUS"),
        (Mode::ShortBreak, "SHORT BREAK"),
        (Mode::LongBreak, "LONG BREAK"),
    ] {
        let label = utils::ascii_future(text);
        let width = utils::string_width(&label) as u16;
        ret.insert(state, (label, width));
    }

    ret
});

static PAUSED_TEXT: LazyLock<String> = LazyLock::new(|| utils::ascii_future(" ( PAUSED )"));
static PAUSED_WIDTH: LazyLock<u16> =
    LazyLock::new(|| utils::string_width(PAUSED_TEXT.as_str()) as u16);
static PAUSED_PARAGRAPH: LazyLock<Paragraph<'static>> = LazyLock::new(|| {
    Paragraph::new(PAUSED_TEXT.to_string()).style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )
});

static SHORTCUTS: LazyLock<Paragraph<'static>> = LazyLock::new(|| {
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
    Paragraph::new(line).alignment(Alignment::Center)
});
