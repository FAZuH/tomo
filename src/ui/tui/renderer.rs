use std::time::Duration;

use ratatui::layout::Constraint;
use ratatui::layout::Flex;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::layout::Size;
use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Gauge;
use ratatui::widgets::Paragraph;
use tui_widgets::big_text::BigText;
use tui_widgets::big_text::PixelSize;
use tui_widgets::scrollview::ScrollView;
use tui_widgets::scrollview::ScrollViewState;
use tui_widgets::scrollview::ScrollbarVisibility;

use crate::config::Config;
use crate::models::pomodoro::Pomodoro;
use crate::models::pomodoro::PomodoroState;
use crate::ui::pages::settings::SETTINGS_VIEW_ITEMS;
use crate::ui::router::Page;
use crate::ui::router::Router;
use crate::utils;

pub struct TuiTimerRenderer {
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
    fn render(&self, frame: &mut Frame, pomodoro: &Pomodoro) {
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

/// Section color scheme for visual distinction
#[derive(Clone, Copy, Debug)]
enum SectionColor {
    Timer,
    Hooks,
    Sounds,
}

impl SectionColor {
    fn border_color(self) -> Color {
        match self {
            SectionColor::Timer => Color::Cyan,
            SectionColor::Hooks => Color::Yellow,
            SectionColor::Sounds => Color::Magenta,
        }
    }

    fn title_style(self) -> Style {
        Style::default()
            .fg(self.border_color())
            .add_modifier(Modifier::BOLD)
    }
}

pub struct TuiSettingsRenderer {
    scroll_state: ScrollViewState,
    selected_idx: u32,
    editing: bool,
    edit_buffer: String,
}

impl TuiSettingsRenderer {
    pub fn new() -> Self {
        Self {
            scroll_state: ScrollViewState::default(),
            selected_idx: 0,
            editing: false,
            edit_buffer: String::new(),
        }
    }

    /// Move selection up
    pub fn select_up(&mut self) {
        self.selected_idx = self
            .selected_idx
            .saturating_sub(1)
            .clamp(0, SETTINGS_VIEW_ITEMS - 1); // 13 items total
    }

    /// Move selection down
    pub fn select_down(&mut self) {
        self.selected_idx = self
            .selected_idx
            .saturating_add(1)
            .clamp(0, SETTINGS_VIEW_ITEMS - 1);
    }

    /// Start editing the currently selected field
    pub fn start_editing(&mut self) {
        self.editing = true;
        self.edit_buffer.clear();
    }

    /// Cancel editing
    pub fn cancel_editing(&mut self) {
        self.editing = false;
        self.edit_buffer.clear();
    }

    /// Get current selection index
    pub fn selected_idx(&self) -> u32 {
        self.selected_idx
    }

    /// Check if currently editing
    pub fn is_editing(&self) -> bool {
        self.editing
    }

    /// Get the current edit buffer
    pub fn edit_buffer(&self) -> &str {
        &self.edit_buffer
    }

    /// Push a character to the edit buffer
    pub fn push_char(&mut self, c: char) {
        self.edit_buffer.push(c);
    }

    /// Pop a character from the edit buffer
    pub fn pop_char(&mut self) {
        self.edit_buffer.pop();
    }

    fn render(&mut self, frame: &mut Frame, config: &Config) {
        let area = frame.area();
        // Reserve space for scrollbar and padding
        let content_width = area.width.saturating_sub(4).max(46);

        // Build sections with proper layout
        let sections = self.build_sections(config, content_width);

        // Calculate total height: title (4) + spacing (1) + sections + padding (2)
        let sections_height: u16 = sections.iter().map(|s| s.height).sum();
        let total_height: u16 = 4 + 1 + sections_height + 2;

        // Create scroll view with full content size
        let mut scroll_view = ScrollView::new(Size::new(content_width, total_height))
            .vertical_scrollbar_visibility(ScrollbarVisibility::Automatic);

        // Render title at top
        let title_area = Rect::new(0, 0, content_width, 4);
        self.render_title(&mut scroll_view, title_area);

        // Render sections with proper spacing
        let mut y = 5u16; // Start after title + 1 row spacing
        for section in sections {
            let section_area = Rect::new(0, y, content_width, section.height);
            self.render_section(&mut scroll_view, section_area, &section);
            y += section.height;
        }

        frame.render_stateful_widget(scroll_view, area, &mut self.scroll_state);
    }

    fn render_title(&self, scroll_view: &mut ScrollView, area: Rect) {
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

    /// Build sections from config, calculating layout and identifying editable items
    fn build_sections(&self, config: &Config, _width: u16) -> Vec<Section> {
        let mut sections = Vec::new();
        let mut item_idx = 0u32;

        // Build Pomodoro Timer section
        let timer_label = "󰔛 Pomodoro Timer";
        let timer_color = Self::section_color_from_label(timer_label);
        let mut timer_rows = Vec::new();

        // Durations subsection
        if !timer_rows.is_empty() {
            timer_rows.push(SectionRow::Blank);
        }
        timer_rows.push(SectionRow::SubSectionHeader("Durations".to_string()));
        self.add_input_to_rows(
            "Focus",
            &format!("{}", config.pomodoro.timer.focus.as_secs() / 60),
            &mut timer_rows,
            &mut item_idx,
        );
        self.add_input_to_rows(
            "Short Break",
            &format!("{}", config.pomodoro.timer.short.as_secs() / 60),
            &mut timer_rows,
            &mut item_idx,
        );
        self.add_input_to_rows(
            "Long Break",
            &format!("{}", config.pomodoro.timer.long.as_secs() / 60),
            &mut timer_rows,
            &mut item_idx,
        );

        self.add_input_to_rows(
            "Long Break Interval",
            &format!("{}", config.pomodoro.timer.long_interval),
            &mut timer_rows,
            &mut item_idx,
        );

        // Auto Start subsection
        if !timer_rows.is_empty() {
            timer_rows.push(SectionRow::Blank);
        }
        timer_rows.push(SectionRow::SubSectionHeader("Auto Start".to_string()));
        self.add_checkbox_to_rows(
            "Focus",
            config.pomodoro.timer.auto_focus,
            &mut timer_rows,
            &mut item_idx,
        );
        self.add_checkbox_to_rows(
            "Short Break",
            config.pomodoro.timer.auto_short,
            &mut timer_rows,
            &mut item_idx,
        );
        self.add_checkbox_to_rows(
            "Long Break",
            config.pomodoro.timer.auto_long,
            &mut timer_rows,
            &mut item_idx,
        );

        let timer_height = 2 + timer_rows.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: timer_label.to_string(),
            color: timer_color,
            height: timer_height,
            rows: timer_rows,
        });

        // Build Command Hooks section
        let hooks_label = "󰛢 Command Hooks";
        let hooks_color = Self::section_color_from_label(hooks_label);
        let mut hooks_rows = Vec::new();

        // Hooks subsection
        if !hooks_rows.is_empty() {
            hooks_rows.push(SectionRow::Blank);
        }
        hooks_rows.push(SectionRow::SubSectionHeader("Hooks".to_string()));
        self.add_input_to_rows(
            "Focus",
            &config.pomodoro.hook.focus,
            &mut hooks_rows,
            &mut item_idx,
        );
        self.add_input_to_rows(
            "Short Break",
            &config.pomodoro.hook.short,
            &mut hooks_rows,
            &mut item_idx,
        );
        self.add_input_to_rows(
            "Long Break",
            &config.pomodoro.hook.long,
            &mut hooks_rows,
            &mut item_idx,
        );

        let hooks_height = 2 + hooks_rows.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: hooks_label.to_string(),
            color: hooks_color,
            height: hooks_height,
            rows: hooks_rows,
        });

        // Build Sounds section
        let sounds_label = "󰕾 Sounds";
        let sounds_color = Self::section_color_from_label(sounds_label);
        let mut sounds_rows = Vec::new();

        // Sound Files subsection
        if !sounds_rows.is_empty() {
            sounds_rows.push(SectionRow::Blank);
        }
        sounds_rows.push(SectionRow::SubSectionHeader("Sound Files".to_string()));
        self.add_input_to_rows(
            "Focus",
            &config
                .pomodoro
                .sound
                .focus
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            &mut sounds_rows,
            &mut item_idx,
        );
        self.add_input_to_rows(
            "Short Break",
            &config
                .pomodoro
                .sound
                .short
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            &mut sounds_rows,
            &mut item_idx,
        );
        self.add_input_to_rows(
            "Long Break",
            &config
                .pomodoro
                .sound
                .long
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            &mut sounds_rows,
            &mut item_idx,
        );

        let sounds_height = 2 + sounds_rows.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: sounds_label.to_string(),
            color: sounds_color,
            height: sounds_height,
            rows: sounds_rows,
        });

        sections
    }

    fn section_color_from_label(label: &str) -> SectionColor {
        if label.contains("Timer") {
            SectionColor::Timer
        } else if label.contains("Hook") {
            SectionColor::Hooks
        } else if label.contains("Sound") {
            SectionColor::Sounds
        } else {
            SectionColor::Timer
        }
    }

    fn add_input_to_rows(
        &self,
        label: &str,
        value: &str,
        rows: &mut Vec<SectionRow>,
        item_idx: &mut u32,
    ) {
        let idx = *item_idx;
        *item_idx += 1;
        let is_selected = self.selected_idx == idx;
        rows.push(SectionRow::Input {
            label: label.to_string(),
            value: if is_selected && self.editing {
                format!("{}█", self.edit_buffer)
            } else {
                value.to_string()
            },
            is_selected,
        });
    }

    fn add_checkbox_to_rows(
        &self,
        label: &str,
        value: bool,
        rows: &mut Vec<SectionRow>,
        item_idx: &mut u32,
    ) {
        let idx = *item_idx;
        *item_idx += 1;
        let is_selected = self.selected_idx == idx;
        rows.push(SectionRow::Checkbox {
            label: label.to_string(),
            value,
            is_selected,
        });
    }

    fn render_section(&self, scroll_view: &mut ScrollView, area: Rect, section: &Section) {
        // Create block with border
        let block = Block::default()
            .title(section.title.clone())
            .title_style(section.color.title_style())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(section.color.border_color()));

        // Render the block
        scroll_view.render_widget(block.clone(), area);

        // Get inner area for content
        let inner = block.inner(area);
        let inner = Rect::new(inner.x, inner.y, inner.width, inner.height);

        // Render rows inside the block
        let mut y = inner.y;
        for row in &section.rows {
            let row_height = row.height();
            let row_area = Rect::new(inner.x, y, inner.width, row_height);
            self.render_section_row(scroll_view, row_area, row);
            y += row_height;
        }
    }

    fn render_section_row(&self, scroll_view: &mut ScrollView, area: Rect, row: &SectionRow) {
        match row {
            SectionRow::Blank => {
                // Nothing to render for blank rows
            }
            SectionRow::SubSectionHeader(label) => {
                let line = Line::from(vec![Span::styled(
                    format!("▸ {} ", label),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::UNDERLINED),
                )]);
                let p = Paragraph::new(line);
                scroll_view.render_widget(p, area);
            }
            SectionRow::Input {
                label,
                value,
                is_selected,
                ..
            } => {
                let selected_bg = if *is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let line = Line::from(vec![
                    Span::styled(
                        format!("{}: ", label),
                        Style::default()
                            .add_modifier(Modifier::DIM)
                            .patch(selected_bg),
                    ),
                    Span::styled(value, Style::default().patch(selected_bg)),
                ]);
                let p = Paragraph::new(line);
                scroll_view.render_widget(p, area);
            }
            SectionRow::Checkbox {
                label,
                value,
                is_selected,
                ..
            } => {
                let selected_bg = if *is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let checkbox = if *value {
                    Span::styled("[x]", Style::default().fg(Color::Cyan).patch(selected_bg))
                } else {
                    Span::styled("[ ]", Style::default().fg(Color::Cyan).patch(selected_bg))
                };
                let line = Line::from(vec![
                    checkbox,
                    Span::styled(" ", selected_bg),
                    Span::styled(label.clone(), selected_bg),
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

/// Represents a section with border
struct Section {
    title: String,
    color: SectionColor,
    height: u16,
    rows: Vec<SectionRow>,
}

/// Individual row within a section
enum SectionRow {
    Blank,
    SubSectionHeader(String),
    Input {
        label: String,
        value: String,
        is_selected: bool,
    },
    Checkbox {
        label: String,
        value: bool,
        is_selected: bool,
    },
}

impl SectionRow {
    fn height(&self) -> u16 {
        match self {
            Self::Blank => 1,
            _ => 1,
        }
    }
}

pub struct TuiRenderer {
    pub timer: TuiTimerRenderer,
    pub settings: TuiSettingsRenderer,
}

impl TuiRenderer {
    pub fn new() -> Self {
        Self {
            timer: TuiTimerRenderer::new(),
            settings: TuiSettingsRenderer::new(),
        }
    }

    pub fn flush(
        &mut self,
        frame: &mut Frame,
        router: &Router,
        pomodoro: &Pomodoro,
        config: &Config,
    ) {
        match router.active_page() {
            Some(Page::Timer) => self.timer.render(frame, pomodoro),
            Some(Page::Settings) => self.settings.render(frame, config),
            None => {}
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
