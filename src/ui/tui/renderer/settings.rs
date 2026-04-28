use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use tui_widgets::big_text::BigText;
use tui_widgets::big_text::PixelSize;
use tui_widgets::scrollview::ScrollView;
use tui_widgets::scrollview::ScrollViewState;
use tui_widgets::scrollview::ScrollbarVisibility;

use crate::config::Config;
use crate::config::pomodoro::Alarm;
use crate::ui::update::settings::SETTINGS_VIEW_ITEMS;

pub struct TuiSettingsRenderer {
    scroll_state: ScrollViewState,
    selected_idx: u32,
    editing: bool,
    edit_buffer: String,
    has_unsaved_changes: bool,
}

impl TuiSettingsRenderer {
    pub fn new() -> Self {
        Self {
            scroll_state: ScrollViewState::default(),
            selected_idx: 0,
            editing: false,
            edit_buffer: String::new(),
            has_unsaved_changes: false,
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

    pub fn set_has_unsaved_changes(&mut self, state: bool) {
        self.has_unsaved_changes = state;
    }

    pub fn render(&mut self, frame: &mut Frame, config: &Config) {
        let area = frame.area();
        // Reserve space for scrollbar and padding
        let content_width = area.width.saturating_sub(4).max(46);

        // Build sections with proper layout
        let sections = self.build_sections(config);

        // Calculate total height: title (4) + spacing (1) + sections + padding (2)
        let sections_height: u16 = sections.iter().map(|s| s.height).sum();
        let total_height: u16 = 4 + 1 + sections_height + 2;

        // Create scroll view with full content size
        let mut scroll_view = ScrollView::new(Size::new(content_width, total_height))
            .vertical_scrollbar_visibility(ScrollbarVisibility::Automatic);

        // Render title at top
        let title_area = Rect::new(0, 0, content_width, 4);
        self.render_title(&mut scroll_view, title_area);

        // Render unsaved changes indicator in the spacing row between title and sections
        let indicator_area = Rect::new(0, 4, content_width, 1);
        self.render_unsaved_indicator(&mut scroll_view, indicator_area);

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

    fn render_unsaved_indicator(&self, scroll_view: &mut ScrollView, area: Rect) {
        if !self.has_unsaved_changes {
            return;
        }
        let line = Line::from(vec![Span::styled(
            "● Unsaved changes",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]);
        scroll_view.render_widget(Paragraph::new(line), area);
    }

    /// Build sections from config, calculating layout and identifying editable items
    fn build_sections(&self, config: &Config) -> Vec<Section> {
        let mut sections = Vec::new();
        let mut item_idx = 0u32;

        self.build_timer_section(config, &mut sections, &mut item_idx);
        self.build_hooks_section(config, &mut sections, &mut item_idx);
        self.build_alarm_section(config, &mut sections, &mut item_idx);

        sections
    }

    fn build_timer_section(
        &self,
        config: &Config,
        sections: &mut Vec<Section>,
        item_idx: &mut u32,
    ) {
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
            format!("{}", config.pomodoro.timer.focus.as_secs() / 60),
            &mut timer_rows,
            item_idx,
        );
        self.add_input_to_rows(
            "Short Break",
            format!("{}", config.pomodoro.timer.short.as_secs() / 60),
            &mut timer_rows,
            item_idx,
        );
        self.add_input_to_rows(
            "Long Break",
            format!("{}", config.pomodoro.timer.long.as_secs() / 60),
            &mut timer_rows,
            item_idx,
        );

        self.add_input_to_rows(
            "Long Break Interval",
            format!("{}", config.pomodoro.timer.long_interval),
            &mut timer_rows,
            item_idx,
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
            item_idx,
        );
        self.add_checkbox_to_rows(
            "Short Break",
            config.pomodoro.timer.auto_short,
            &mut timer_rows,
            item_idx,
        );
        self.add_checkbox_to_rows(
            "Long Break",
            config.pomodoro.timer.auto_long,
            &mut timer_rows,
            item_idx,
        );

        let timer_height = 2 + timer_rows.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: timer_label.to_string(),
            color: timer_color,
            height: timer_height,
            rows: timer_rows,
        });
    }

    fn build_hooks_section(
        &self,
        config: &Config,
        sections: &mut Vec<Section>,
        item_idx: &mut u32,
    ) {
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
            item_idx,
        );
        self.add_input_to_rows(
            "Short Break",
            &config.pomodoro.hook.short,
            &mut hooks_rows,
            item_idx,
        );
        self.add_input_to_rows(
            "Long Break",
            &config.pomodoro.hook.long,
            &mut hooks_rows,
            item_idx,
        );

        let hooks_height = 2 + hooks_rows.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: hooks_label.to_string(),
            color: hooks_color,
            height: hooks_height,
            rows: hooks_rows,
        });
    }

    fn build_alarm_section(
        &self,
        config: &Config,
        sections: &mut Vec<Section>,
        item_idx: &mut u32,
    ) {
        let alarm = &config.pomodoro.alarm;
        let mut rows = Vec::new();

        // Alarm Files subsection
        rows.push(SectionRow::SubSectionHeader("Alarm Files".to_string()));
        self.add_input_to_rows(
            "Focus",
            Self::get_alarm_path_value(&alarm.focus),
            &mut rows,
            item_idx,
        );
        self.add_input_to_rows(
            "Short Break",
            Self::get_alarm_path_value(&alarm.short),
            &mut rows,
            item_idx,
        );
        self.add_input_to_rows(
            "Long Break",
            Self::get_alarm_path_value(&alarm.long),
            &mut rows,
            item_idx,
        );

        // Alarm Volumes subsection
        rows.push(SectionRow::Blank);
        rows.push(SectionRow::SubSectionHeader("Alarm Volumes".to_string()));
        self.add_input_to_rows(
            "Focus",
            Self::get_alarm_volume_value(&alarm.focus),
            &mut rows,
            item_idx,
        );
        self.add_input_to_rows(
            "Short Break",
            Self::get_alarm_volume_value(&alarm.short),
            &mut rows,
            item_idx,
        );
        self.add_input_to_rows(
            "Long Break",
            Self::get_alarm_volume_value(&alarm.long),
            &mut rows,
            item_idx,
        );

        let height = 2 + rows.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: "󰕾 Alarm".to_string(),
            color: SectionColor::Alarm,
            height,
            rows,
        });
    }

    fn get_alarm_path_value(alarm: &Alarm) -> String {
        alarm
            .path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default()
    }

    fn get_alarm_volume_value(alarm: &Alarm) -> String {
        alarm.volume.to_string()
    }

    fn section_color_from_label(label: &str) -> SectionColor {
        if label.contains("Timer") {
            SectionColor::Timer
        } else if label.contains("Hook") {
            SectionColor::Hooks
        } else if label.contains("Alarm") {
            SectionColor::Alarm
        } else {
            SectionColor::Timer
        }
    }

    fn add_input_to_rows(
        &self,
        label: impl ToString,
        value: impl ToString,
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

/// Section color scheme for visual distinction
#[derive(Clone, Copy, Debug)]
enum SectionColor {
    Timer,
    Hooks,
    Alarm,
}

impl SectionColor {
    fn border_color(self) -> Color {
        match self {
            SectionColor::Timer => Color::Cyan,
            SectionColor::Hooks => Color::Yellow,
            SectionColor::Alarm => Color::Magenta,
        }
    }

    fn title_style(self) -> Style {
        Style::default()
            .fg(self.border_color())
            .add_modifier(Modifier::BOLD)
    }
}
