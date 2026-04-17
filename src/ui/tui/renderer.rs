use std::time::Duration;

use ratatui::layout::Constraint;
use ratatui::layout::Flex;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::layout::Size;
use ratatui::prelude::*;
use ratatui::widgets::Gauge;
use ratatui::widgets::Paragraph;
use tui_widgets::big_text::BigText;
use tui_widgets::big_text::PixelSize;
use tui_widgets::scrollview::ScrollView;
use tui_widgets::scrollview::ScrollViewState;
use tui_widgets::scrollview::ScrollbarVisibility;

use crate::models::pomodoro::PomodoroState;
use crate::ui::view::RenderCommand;
use crate::ui::view::SettingsRenderCommand;
use crate::ui::view::TimerRenderCommand;
use crate::utils;

struct TuiTimerRenderer {
    layout: Layout,
    paused_p: Paragraph<'static>,
}

impl TuiTimerRenderer {
    pub fn new() -> Self {
        Self {
            layout: Self::layout(),
            paused_p: Self::paused_paragraph(),
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
                Stats { .. } => stats = Some(*cmd),
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

    fn paused_paragraph() -> Paragraph<'static> {
        let label = utils::ascii_future(" ( PAUSED )");
        Paragraph::new(label).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    }
}

struct TuiSettingsRenderer {
    scroll_state: ScrollViewState,
}

impl TuiSettingsRenderer {
    pub fn new() -> Self {
        Self {
            scroll_state: ScrollViewState::default(),
        }
    }

    fn render(&mut self, frame: &mut Frame, cmds: Vec<SettingsRenderCommand>) {
        let area = frame.area();
        let rows = self.flatten_commands(cmds);

        // Calculate total content height
        let total_height: u16 = rows.iter().map(|row| row.height()).sum();
        let content_width = area.width.saturating_sub(2).max(40); // Reserve 2 cols for scrollbar

        // Create scroll view with content size
        let mut scroll_view = ScrollView::new(Size::new(content_width, total_height))
            .vertical_scrollbar_visibility(ScrollbarVisibility::Automatic);

        // Render all rows into the scroll view
        let mut y = 0u16;
        for row in rows {
            let height = row.height();
            let row_area = Rect::new(0, y, content_width, height);
            self.render_row(&mut scroll_view, row_area, &row);
            y += height;
        }

        // Render the scroll view into the frame
        frame.render_stateful_widget(scroll_view, area, &mut self.scroll_state);
    }

    /// Flattens the hierarchical SettingsRenderCommand into a list of RenderRows
    fn flatten_commands(&self, cmds: Vec<SettingsRenderCommand>) -> Vec<RenderRow> {
        let mut rows = Vec::new();
        rows.push(RenderRow::Title);
        rows.push(RenderRow::Blank);

        for cmd in cmds {
            self.flatten_command(cmd, &mut rows, 0);
            // Add blank line after each top-level section
            rows.push(RenderRow::Blank);
        }

        rows
    }

    /// Recursively flattens a command into rows
    #[allow(clippy::only_used_in_recursion)]
    fn flatten_command(&self, cmd: SettingsRenderCommand, rows: &mut Vec<RenderRow>, depth: u16) {
        use SettingsRenderCommand as S;

        match cmd {
            S::Title => {
                // Title is handled separately at the start
            }
            S::Section { label, children } => {
                rows.push(RenderRow::SectionHeader(label));
                for child in children {
                    self.flatten_command(child, rows, depth + 1);
                }
            }
            S::SubSection {
                label, children, ..
            } => {
                rows.push(RenderRow::SubSectionHeader(label));
                for child in children {
                    self.flatten_command(child, rows, depth + 2);
                }
            }
            S::Input { label, value } => {
                rows.push(RenderRow::Input { label, value });
            }
            S::Checkbox { label, value } => {
                rows.push(RenderRow::Checkbox { label, value });
            }
        }
    }

    /// Renders a single row into the scroll view buffer
    fn render_row(&self, scroll_view: &mut ScrollView, area: Rect, row: &RenderRow) {
        match row {
            RenderRow::Title => {
                let big_text = BigText::builder()
                    .pixel_size(PixelSize::Quadrant)
                    .style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                    .lines(vec!["Settings".into()])
                    .centered()
                    .build();
                scroll_view.render_widget(big_text, area);
            }
            RenderRow::Blank => {
                // No need to render anything for blank rows
            }
            RenderRow::SectionHeader(label) => {
                let line = Line::from(vec![Span::styled(
                    label.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                )]);
                let p = Paragraph::new(line);
                scroll_view.render_widget(p, area);
            }
            RenderRow::SubSectionHeader(label) => {
                let line = Line::from(vec![Span::styled(
                    format!("  {}", label),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::DIM),
                )]);
                let p = Paragraph::new(line);
                scroll_view.render_widget(p, area);
            }
            RenderRow::Input { label, value } => {
                let line = Line::from(vec![
                    Span::styled(
                        format!("    {}: ", label),
                        Style::default().add_modifier(Modifier::DIM),
                    ),
                    Span::styled(value.clone(), Style::default()),
                ]);
                let p = Paragraph::new(line);
                scroll_view.render_widget(p, area);
            }
            RenderRow::Checkbox { label, value } => {
                let checkbox = if *value {
                    Span::styled("[x]", Style::default().fg(Color::Cyan))
                } else {
                    Span::styled("[ ]", Style::default().fg(Color::Cyan))
                };
                let line = Line::from(vec![
                    Span::styled("    ", Style::default()),
                    checkbox,
                    Span::styled(" ", Style::default()),
                    Span::styled(label.clone(), Style::default()),
                ]);
                let p = Paragraph::new(line);
                scroll_view.render_widget(p, area);
            }
        }
    }

    /// Scroll up by one row
    pub fn scroll_up(&mut self) {
        self.scroll_state.scroll_up();
    }

    /// Scroll down by one row
    pub fn scroll_down(&mut self) {
        self.scroll_state.scroll_down();
    }
}

/// Internal representation of a row to render in the settings view
enum RenderRow {
    /// Title using BigText (4 rows tall)
    Title,
    /// Blank line for spacing
    Blank,
    /// Section header with icon (bold)
    SectionHeader(String),
    /// Subsection header (bold + dim, indented)
    SubSectionHeader(String),
    /// Input field with label and value
    Input { label: String, value: String },
    /// Checkbox with label and checked state
    Checkbox { label: String, value: bool },
}

impl RenderRow {
    /// Returns the height of this row in terminal rows
    fn height(&self) -> u16 {
        match self {
            Self::Title => 4,
            _ => 1,
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
            timer: TuiTimerRenderer::new(),
            settings: TuiSettingsRenderer::new(),
        }
    }

    pub fn flush(&mut self, frame: &mut Frame, commands: Vec<RenderCommand>) {
        for cmd in commands {
            match cmd {
                RenderCommand::Timer(cmds) => self.timer.render(frame, cmds),
                RenderCommand::Settings(cmds) => self.settings.render(frame, cmds),
            }
        }
    }

    /// Scroll up in the settings view
    pub fn settings_scroll_up(&mut self) {
        self.settings.scroll_up();
    }

    /// Scroll down in the settings view
    pub fn settings_scroll_down(&mut self) {
        self.settings.scroll_down();
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
