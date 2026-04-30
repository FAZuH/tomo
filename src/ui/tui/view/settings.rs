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
use crate::ui::StatefulViewRef;
use crate::ui::tui::model::SettingsItem;
use crate::ui::tui::model::SettingsModel;
use crate::ui::tui::model::SettingsSection;
use crate::ui::tui::view::Canvas;

type State = SettingsState;

pub struct TuiSettingsView {}

pub struct SettingsState {
    pub(super) model: SettingsModel,
    pub(super) conf: Config,
}

impl SettingsState {
    pub fn new(model: SettingsModel, conf: Config) -> Self {
        Self { model, conf }
    }
}

impl TuiSettingsView {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'a> StatefulViewRef<Canvas<'a, '_>> for TuiSettingsView {
    type State = State;
    type Result = ();

    fn render_stateful_ref(&self, canvas: Canvas<'a, '_>, state: &mut State) {
        let area = canvas.area();
        let buf = canvas.buffer_mut();
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
        self.prompt(canvas, area, model);
    }
}

impl TuiSettingsView {
    fn title(&self, scroll: &mut ScrollView, area: Rect) {
        scroll.render_widget(TITLE.clone(), area);
    }

    fn save_indicator(&self, scroll: &mut ScrollView, area: Rect, model: &mut SettingsModel) {
        if model.has_unsaved_changes() {
            scroll.render_widget(SAVED_INDICATOR.clone(), area);
        }
    }

    fn prompt(&self, frame: &mut Frame, area: Rect, model: &mut SettingsModel) {
        // Render prompt overlay
        if let Some(ref mut prompt) = model.prompt_state_mut() {
            let buf = frame.buffer_mut();
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

            TextPrompt::new(Cow::Borrowed("")).draw(frame, inner, &mut prompt.text_state);
        }
    }

    /// Build sections from config, calculating layout and identifying editable items
    fn build_sections(&self, model: &SettingsModel, config: &PomodoroConfig) -> Vec<Section> {
        let mut sections = Vec::new();

        self.build_timer_section(model, &config.timer, &mut sections);
        self.build_hooks_section(model, &config.hook, &mut sections);
        self.build_alarm_section(model, &config.alarm, &mut sections);

        sections
    }

    fn build_timer_section(&self, m: &SettingsModel, conf: &Timers, sections: &mut Vec<Section>) {
        use SettingsItem::*;
        // Build Pomodoro Timer section
        let label = "󰔛 Pomodoro Timer";
        let mut r = Vec::new();

        // Durations subsection
        if !r.is_empty() {
            r.push(SectionRow::Blank);
        }
        r.push(SectionRow::SubSectionHeader("Durations".into()));
        self.add_inpt(
            m,
            TimerFocus,
            format!("{}", conf.focus.as_secs() / 60),
            &mut r,
        );
        self.add_inpt(
            m,
            TimerShort,
            format!("{}", conf.short.as_secs() / 60),
            &mut r,
        );
        self.add_inpt(
            m,
            TimerLong,
            format!("{}", conf.long.as_secs() / 60),
            &mut r,
        );
        self.add_inpt(
            m,
            TimerLongInterval,
            format!("{}", conf.long_interval),
            &mut r,
        );

        // Auto Start subsection
        if !r.is_empty() {
            r.push(SectionRow::Blank);
        }
        r.push(SectionRow::SubSectionHeader("Auto Start".into()));
        self.add_box(m, TimerAutoFocus, conf.auto_focus, &mut r);
        self.add_box(m, TimerAutoShort, conf.auto_short, &mut r);
        self.add_box(m, TimerAutoLong, conf.auto_long, &mut r);

        let height = 2 + r.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: label.into(),
            color: SettingsSection::Timer,
            height,
            rows: r,
        });
    }

    fn build_hooks_section(
        &self,
        model: &SettingsModel,
        conf: &Hooks,
        sections: &mut Vec<Section>,
    ) {
        use SettingsItem::*;
        // Build Command Hooks section
        let label = "󰛢 Command Hooks";
        let mut r = Vec::new();

        // Hooks subsection
        if !r.is_empty() {
            r.push(SectionRow::Blank);
        }
        r.push(SectionRow::SubSectionHeader("Hooks".into()));
        self.add_inpt(model, HookFocus, &conf.focus, &mut r);
        self.add_inpt(model, HookShort, &conf.short, &mut r);
        self.add_inpt(model, HookLong, &conf.long, &mut r);

        let height = 2 + r.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: label.into(),
            color: SettingsSection::Hook,
            height,
            rows: r,
        });
    }

    fn build_alarm_section(
        &self,
        model: &SettingsModel,
        conf: &Alarms,
        sections: &mut Vec<Section>,
    ) {
        use SettingsItem::*;
        let mut r = Vec::new();

        // Alarm Files subsection
        r.push(SectionRow::SubSectionHeader("Alarm Files".into()));
        self.add_inpt(model, AlarmPathFocus, conf.focus.path(), &mut r);
        self.add_inpt(model, AlarmPathShort, conf.short.path(), &mut r);
        self.add_inpt(model, AlarmPathLong, conf.long.path(), &mut r);

        // Alarm Volumes subsection
        r.push(SectionRow::Blank);
        r.push(SectionRow::SubSectionHeader("Alarm Volumes".into()));
        self.add_inpt(model, AlarmVolumeFocus, conf.focus.volume(), &mut r);
        self.add_inpt(model, AlarmVolumeShort, conf.short.volume(), &mut r);
        self.add_inpt(model, AlarmVolumeLong, conf.long.volume(), &mut r);

        let height = 2 + r.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: "󰕾 Alarm".into(),
            color: SettingsSection::Alarm,
            height,
            rows: r,
        });
    }

    fn add_inpt(
        &self,
        model: &SettingsModel,
        item: SettingsItem,
        value: impl ToString,
        rows: &mut Vec<SectionRow>,
    ) {
        rows.push(SectionRow::Input {
            label: item.label().into(),
            value: value.to_string(),
            is_selected: model.selected() == item,
        });
    }

    fn add_box(
        &self,
        model: &SettingsModel,
        item: SettingsItem,
        value: bool,
        rows: &mut Vec<SectionRow>,
    ) {
        rows.push(SectionRow::Checkbox {
            label: item.label().into(),
            value,
            is_selected: model.selected() == item,
        });
    }
}

/// Represents a section with border
#[derive(Clone, Debug, PartialEq, Eq)]
struct Section {
    title: String,
    color: SettingsSection,
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

impl SettingsSection {
    fn border_color(self) -> Color {
        match self {
            SettingsSection::Timer => Color::Cyan,
            SettingsSection::Hook => Color::Yellow,
            SettingsSection::Alarm => Color::Magenta,
        }
    }

    fn title_style(self) -> Style {
        Style::default()
            .fg(self.border_color())
            .add_modifier(Modifier::BOLD)
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
