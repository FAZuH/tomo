use std::borrow::Cow;
use std::sync::LazyLock;

use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use tui_widgets::big_text::BigText;
use tui_widgets::big_text::PixelSize;
use tui_widgets::prompts::prelude::*;
use tui_widgets::scrollview::ScrollView;
use tui_widgets::scrollview::ScrollbarVisibility;

use crate::config::Config;
use crate::config::pomodoro::Alarms;
use crate::config::pomodoro::Hooks;
use crate::config::pomodoro::PomodoroConfig;
use crate::config::pomodoro::Timers;
use crate::ui::tui::model::SettingsModel;

type State = SettingsState;
type Buf<'a> = &'a mut Buffer;

pub struct TuiSettingsRenderer {}

impl TuiSettingsRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct SettingsState {
    pub(super) model: SettingsModel,
    pub(super) conf: Config,
}

impl SettingsState {
    pub fn new(model: SettingsModel, conf: Config) -> Self {
        Self { model, conf }
    }
}

impl StatefulWidget for TuiSettingsRenderer {
    type State = State;

    fn render(self, area: Rect, buf: Buf, state: &mut Self::State) {
        let SettingsState { model, conf } = state;

        // Reserve space for scrollbar and padding
        let content_width = area.width.saturating_sub(4).max(46);

        // Build sections with proper layout
        let sections = self.build_sections(model, &conf.pomodoro);

        // Calculate total height: title (4) + spacing (1) + sections + padding (2)
        let sections_height: u16 = sections.iter().map(|s| s.height).sum();
        let total_height: u16 = 4 + 1 + sections_height + 2;

        // Create scroll view with full content size
        let mut scroll_view = ScrollView::new(Size::new(content_width, total_height))
            .vertical_scrollbar_visibility(ScrollbarVisibility::Automatic);

        // Render title at top
        let title_area = Rect::new(0, 0, content_width, 4);
        self.title(&mut scroll_view, title_area);

        // Render unsaved changes indicator in the spacing row between title and sections
        let indicator_area = Rect::new(0, 4, content_width, 1);
        self.save_indicator(&mut scroll_view, indicator_area, model);

        // Render sections with proper spacing
        let mut y = 5u16; // Start after title + 1 row spacing
        for section in sections {
            let section_area = Rect::new(0, y, content_width, section.height);
            y += section.height;
            scroll_view.render_widget(section, section_area);
        }

        scroll_view.render(area, buf, model.scroll_state_mut());

        // Render prompt popup
        self.prompt(area, buf, model);
    }
}

impl TuiSettingsRenderer {
    fn title(&self, scroll: &mut ScrollView, area: Rect) {
        scroll.render_widget(TITLE.clone(), area);
    }

    fn save_indicator(&self, scroll: &mut ScrollView, area: Rect, model: &mut SettingsModel) {
        if model.has_unsaved_changes() {
            scroll.render_widget(SAVED_INDICATOR.clone(), area);
        }
    }

    fn prompt(&self, area: Rect, buf: Buf, model: &mut SettingsModel) {
        // Render prompt overlay
        if let Some(ref mut prompt) = model.prompt_state_mut() {
            let popup_width = 50.min(area.width.saturating_sub(4));
            let popup_height = 3;

            let vertical = Layout::vertical([Constraint::Length(popup_height)]).flex(Flex::Center);
            let horizontal =
                Layout::horizontal([Constraint::Length(popup_width)]).flex(Flex::Center);
            let [popup_area] = vertical.areas(area);
            let [popup_area] = horizontal.areas(popup_area);

            Clear.render(popup_area, buf);

            let block = Block::default()
                .title(prompt.label.clone())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            let inner = block.inner(popup_area);
            block.render(popup_area, buf);

            TextPrompt::new(Cow::Borrowed("")).render(inner, buf, &mut prompt.text_state);
        }
    }

    /// Build sections from config, calculating layout and identifying editable items
    fn build_sections(&self, model: &SettingsModel, config: &PomodoroConfig) -> Vec<Section> {
        let mut sections = Vec::new();
        let mut item_idx = 0u32;

        self.build_timer_section(model, &config.timer, &mut sections, &mut item_idx);
        self.build_hooks_section(model, &config.hook, &mut sections, &mut item_idx);
        self.build_alarm_section(model, &config.alarm, &mut sections, &mut item_idx);

        sections
    }

    fn build_timer_section(
        &self,
        model: &SettingsModel,
        conf: &Timers,
        sections: &mut Vec<Section>,
        i: &mut u32,
    ) {
        // Build Pomodoro Timer section
        let label = "󰔛 Pomodoro Timer";
        let color = SectionColor::from_label(label);
        let mut r = Vec::new();

        // Durations subsection
        if !r.is_empty() {
            r.push(SectionRow::Blank);
        }
        r.push(SectionRow::SubSectionHeader("Durations".to_string()));
        self.add_inpt(
            model,
            "Focus",
            format!("{}", conf.focus.as_secs() / 60),
            &mut r,
            i,
        );
        self.add_inpt(
            model,
            "Short Break",
            format!("{}", conf.short.as_secs() / 60),
            &mut r,
            i,
        );
        self.add_inpt(
            model,
            "Long Break",
            format!("{}", conf.long.as_secs() / 60),
            &mut r,
            i,
        );

        self.add_inpt(
            model,
            "Long Break Interval",
            format!("{}", conf.long_interval),
            &mut r,
            i,
        );

        // Auto Start subsection
        if !r.is_empty() {
            r.push(SectionRow::Blank);
        }
        r.push(SectionRow::SubSectionHeader("Auto Start".to_string()));
        self.add_box(model, "Focus", conf.auto_focus, &mut r, i);
        self.add_box(model, "Short Break", conf.auto_short, &mut r, i);
        self.add_box(model, "Long Break", conf.auto_long, &mut r, i);

        let height = 2 + r.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: label.to_string(),
            color,
            height,
            rows: r,
        });
    }

    fn build_hooks_section(
        &self,
        model: &SettingsModel,
        conf: &Hooks,
        sections: &mut Vec<Section>,
        i: &mut u32,
    ) {
        // Build Command Hooks section
        let label = "󰛢 Command Hooks";
        let color = SectionColor::from_label(label);
        let mut r = Vec::new();

        // Hooks subsection
        if !r.is_empty() {
            r.push(SectionRow::Blank);
        }
        r.push(SectionRow::SubSectionHeader("Hooks".to_string()));
        self.add_inpt(model, "Focus", &conf.focus, &mut r, i);
        self.add_inpt(model, "Short Break", &conf.short, &mut r, i);
        self.add_inpt(model, "Long Break", &conf.long, &mut r, i);

        let height = 2 + r.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: label.to_string(),
            color,
            height,
            rows: r,
        });
    }

    fn build_alarm_section(
        &self,
        model: &SettingsModel,
        conf: &Alarms,
        sections: &mut Vec<Section>,
        i: &mut u32,
    ) {
        let mut r = Vec::new();

        // Alarm Files subsection
        r.push(SectionRow::SubSectionHeader("Alarm Files".to_string()));
        self.add_inpt(model, "Focus", conf.focus.path(), &mut r, i);
        self.add_inpt(model, "Short Break", conf.short.path(), &mut r, i);
        self.add_inpt(model, "Long Break", conf.long.path(), &mut r, i);

        // Alarm Volumes subsection
        r.push(SectionRow::Blank);
        r.push(SectionRow::SubSectionHeader("Alarm Volumes".to_string()));
        self.add_inpt(model, "Focus", conf.focus.volume(), &mut r, i);
        self.add_inpt(model, "Short Break", conf.short.volume(), &mut r, i);
        self.add_inpt(model, "Long Break", conf.long.volume(), &mut r, i);

        let height = 2 + r.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: "󰕾 Alarm".to_string(),
            color: SectionColor::Alarm,
            height,
            rows: r,
        });
    }

    fn add_inpt(
        &self,
        model: &SettingsModel,
        label: impl ToString,
        value: impl ToString,
        rows: &mut Vec<SectionRow>,
        item_idx: &mut u32,
    ) {
        let idx = *item_idx;
        *item_idx += 1;
        rows.push(SectionRow::Input {
            label: label.to_string(),
            value: value.to_string(),
            is_selected: model.selected_idx() == idx,
        });
    }

    fn add_box(
        &self,
        model: &SettingsModel,
        label: &str,
        value: bool,
        rows: &mut Vec<SectionRow>,
        item_idx: &mut u32,
    ) {
        let idx = *item_idx;
        *item_idx += 1;
        rows.push(SectionRow::Checkbox {
            label: label.to_string(),
            value,
            is_selected: model.selected_idx() == idx,
        });
    }
}

/// Represents a section with border
#[derive(Clone, Debug, PartialEq, Eq)]
struct Section {
    title: String,
    color: SectionColor,
    height: u16,
    rows: Vec<SectionRow>,
}

/// Individual row within a section
#[derive(Clone, Debug, PartialEq, Eq)]
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

impl Widget for Section {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // Create block with border
        let block = Block::default()
            .title(self.title.clone())
            .title_style(self.color.title_style())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.color.border_color()));

        // Get inner area for content
        let inner = block.inner(area);
        let inner = Rect::new(inner.x, inner.y, inner.width, inner.height);

        // Render the block
        block.render(area, buf);

        // Render rows inside the block
        let mut y = inner.y;
        for row in self.rows {
            let row_height = row.height();
            let row_area = Rect::new(inner.x, y, inner.width, row_height);
            row.render(row_area, buf);
            y += row_height;
        }
    }
}

impl Widget for SectionRow {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self {
            SectionRow::Blank => Line::from("").render(area, buf),
            SectionRow::SubSectionHeader(label) => {
                let line = Line::from(Span::styled(
                    format!("▸ {} ", label),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::UNDERLINED),
                ));
                Paragraph::new(line).render(area, buf);
            }
            SectionRow::Input {
                label,
                value,
                is_selected,
            } => {
                let bg = if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let line = Line::from(vec![
                    Span::styled(
                        format!("{}: ", label),
                        Style::default().add_modifier(Modifier::DIM).patch(bg),
                    ),
                    Span::styled(value, Style::default().patch(bg)),
                ]);
                Paragraph::new(line).render(area, buf);
            }
            SectionRow::Checkbox {
                label,
                value,
                is_selected,
            } => {
                let bg = if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let checkbox = if value {
                    Span::styled("[x]", Style::default().fg(Color::Cyan).patch(bg))
                } else {
                    Span::styled("[ ]", Style::default().fg(Color::Cyan).patch(bg))
                };

                let line = Line::from(vec![
                    checkbox,
                    Span::styled(" ", bg),
                    Span::styled(label.clone(), bg),
                ]);
                Paragraph::new(line).render(area, buf);
            }
        }
    }
}

/// Section color scheme for visual distinction
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    fn from_label(label: impl AsRef<str>) -> Self {
        let label = label.as_ref();
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
}

pub struct SettingsPrompt {
    pub text_state: TextState<'static>,
    pub label: String,
}

static TITLE: LazyLock<BigText<'static>> = LazyLock::new(|| {
    BigText::builder()
        .pixel_size(PixelSize::Quadrant)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .lines(vec!["Settings".into()])
        .centered()
        .build()
});

static SAVED_INDICATOR: LazyLock<Paragraph<'static>> = LazyLock::new(|| {
    Paragraph::new(Line::from(vec![Span::styled(
        "● Unsaved changes",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]))
});
