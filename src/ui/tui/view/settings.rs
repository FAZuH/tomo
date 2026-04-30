use std::borrow::Cow;
use std::sync::LazyLock;

use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
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

        // Split area for scroll view and help bar
        let [content_area, help_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(area);

        // Reserve space for scrollbar and padding
        let content_width = content_area.width.saturating_sub(2).max(46);

        // Build sections with proper layout
        let sections = self.build_sections(model, &conf.pomodoro);

        // Create scroll view with full content size
        let mut scroll_view = ScrollView::new(Size::new(content_width, content_area.height))
            .vertical_scrollbar_visibility(ScrollbarVisibility::Automatic);

        // Render unsaved changes indicator in the spacing row between title and sections
        let indicator_area = Rect::new(0, 0, content_width, 1);
        self.save_indicator(&mut scroll_view, indicator_area, model);

        // Render sections with proper spacing
        let mut y = 2u16;
        let last = sections.last().unwrap().section;
        for section in sections {
            let section_area = if section.section == last {
                Rect::new(0, y, content_width, content_area.height)
            } else {
                Rect::new(0, y, content_width, section.height)
            };
            y += section.height;
            scroll_view.render_widget(section, section_area);
        }

        scroll_view.render(content_area, buf, model.scroll_state_mut());

        // Render help bar at bottom
        self.keybinds(help_area, buf, model);

        // Render prompt popup (over full area, including help bar)
        self.prompt(canvas, area, model);
    }
}

impl TuiSettingsView {
    fn save_indicator(&self, scroll: &mut ScrollView, area: Rect, model: &mut SettingsModel) {
        if model.has_unsaved_changes() {
            scroll.render_widget(SAVED_INDICATOR.clone(), area);
        }
    }

    fn keybinds(&self, area: Rect, buf: &mut Buffer, model: &SettingsModel) {
        if model.show_keybinds() {
            KEYBINDS_ON.clone().render(area, buf);
        } else {
            KEYBINDS_OFF.clone().render(area, buf);
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

    fn build_timer_section(
        &self,
        model: &SettingsModel,
        conf: &Timers,
        sections: &mut Vec<Section>,
    ) {
        use SettingsItem::*;
        // Build Pomodoro Timer section
        let mut r = Vec::new();

        // Durations subsection
        if !r.is_empty() {
            r.push(SectionRow::Blank);
        }
        r.push(SectionRow::SubSectionHeader("Durations".into()));
        self.add_inpt(
            model,
            TimerFocus,
            format!("{}", conf.focus.as_secs() / 60),
            &mut r,
        );
        self.add_inpt(
            model,
            TimerShort,
            format!("{}", conf.short.as_secs() / 60),
            &mut r,
        );
        self.add_inpt(
            model,
            TimerLong,
            format!("{}", conf.long.as_secs() / 60),
            &mut r,
        );
        self.add_inpt(
            model,
            TimerLongInterval,
            format!("{}", conf.long_interval),
            &mut r,
        );

        // Auto Start subsection
        if !r.is_empty() {
            r.push(SectionRow::Blank);
        }
        r.push(SectionRow::SubSectionHeader("Auto Start".into()));
        self.add_box(model, TimerAutoFocus, conf.auto_focus, &mut r);
        self.add_box(model, TimerAutoShort, conf.auto_short, &mut r);
        self.add_box(model, TimerAutoLong, conf.auto_long, &mut r);

        let height = 2 + r.iter().map(|r| r.height()).sum::<u16>();
        sections.push(Section {
            title: "[1] Pomodoro Timer".into(),
            section: SettingsSection::Timer,
            sel_item: model.selected(),
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
            title: "[2] Command Hooks".into(),
            section: SettingsSection::Hook,
            sel_item: model.selected(),
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
            title: "[3] Alarm".into(),
            section: SettingsSection::Alarm,
            sel_item: model.selected(),
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
    section: SettingsSection,
    sel_item: SettingsItem,
    height: u16,
    rows: Vec<SectionRow>,
}

impl Section {
    fn border_color(&self) -> Color {
        if self.sel_item.section() == self.section {
            Color::Green
        } else {
            Color::White
        }
    }
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
        let style = Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD);

        // Create block with border
        let block = Block::default()
            .title(self.title.clone())
            .title_style(style)
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(self.border_color()));

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
                let line = Line::from(vec![
                    Span::styled(
                        " ",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("▸{label} "),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::UNDERLINED),
                    ),
                ]);
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
                        format!(" {label}: "),
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
                    Span::styled(" [x]", Style::default().fg(Color::Cyan).patch(bg))
                } else {
                    Span::styled(" [ ]", Style::default().fg(Color::Cyan).patch(bg))
                };

                let line = Line::from(vec![
                    checkbox,
                    Span::styled("", bg),
                    Span::styled(label.clone(), bg),
                ]);
                Paragraph::new(line).render(area, buf);
            }
        }
    }
}

pub struct SettingsPrompt {
    pub text_state: TextState<'static>,
    pub label: String,
}

static KEYBINDS_ON: LazyLock<Paragraph<'static>> = LazyLock::new(|| {
    let dim = Style::default().dim();
    let bright = Style::default();
    let sep = Span::styled(" • ", dim);
    Paragraph::new(vec![
        Line::from(vec![
            Span::styled("↑/↓/j/k", bright),
            Span::styled(": Navigate", dim),
            sep.clone(),
            Span::styled("Tab", bright),
            Span::styled(": Sections", dim),
            sep.clone(),
            Span::styled("1/2/3", bright),
            Span::styled(": Jump", dim),
        ]),
        Line::from(vec![
            Span::styled("Space/Enter", bright),
            Span::styled(": Toggle", dim),
            sep.clone(),
            Span::styled("Enter", bright),
            Span::styled(": Edit", dim),
            sep.clone(),
            Span::styled("s", bright),
            Span::styled(": Save", dim),
            sep.clone(),
            Span::styled("Esc", bright),
            Span::styled(": Back", dim),
            sep.clone(),
            Span::styled("q", bright),
            Span::styled(": Quit", dim),
            sep.clone(),
            Span::styled("?", bright),
            Span::styled(": Disable Help", dim),
        ]),
    ])
    .alignment(Alignment::Center)
});

static KEYBINDS_OFF: LazyLock<Paragraph<'static>> = LazyLock::new(|| {
    let dim = Style::default().dim();
    let bright = Style::default();
    let line = Line::from(vec![Span::styled("?", bright), Span::styled(": Help", dim)]);
    Paragraph::new(line).alignment(Alignment::Center)
});

static SAVED_INDICATOR: LazyLock<Paragraph<'static>> = LazyLock::new(|| {
    Paragraph::new(Line::from(vec![Span::styled(
        "● Unsaved changes",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]))
});
