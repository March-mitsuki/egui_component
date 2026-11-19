//! Date-only calendar. Applications own selection, navigation and localized labels.
//! Range endpoints are inclusive. No system clock or locale is consulted.
//!
//! ```
//! use egui_component::components::calendar::{self, Date, Month, Selection, State, YearMonth};
//! # fn show(ui: &mut egui::Ui, theme: &egui_component::theme::Theme) {
//! let initial = Date::from_calendar_date(2026, Month::November, 17).unwrap();
//! // Keep these values in application state across frames.
//! let mut selection = Selection::Single(Some(initial));
//! let mut state = State::new(YearMonth::from_date(initial));
//! let result = calendar::render(ui, ui.make_persistent_id("calendar"),
//!     &mut selection, &mut state, calendar::Style::new(theme));
//! if result.selection_finished {
//!     // A containing date picker may close its popup here.
//! }
//! # }
//! ```
//!
//! Arrow keys move by day/week, Home/End move to week edges, Page Up/Down
//! change month (clamping the day), and Enter/Space select. Disabled dates are
//! skipped. `Response::changed` only reports selection changes; navigation is
//! reported separately. External changes to selection never reset the month.
//! Hover previews do not mutate selection. A range cannot cross a disabled day;
//! an invalid endpoint leaves the start pending. Clicking a completed range
//! starts a new selection. Calendar does not clear selection automatically.
use super::{icon_button, select, text};
use crate::theme::{ButtonSize, ButtonVariant, Theme};
use egui::{Align2, Color32, FontId, Id, Key, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};
pub use time::{Date, Month, Weekday};

/// A validated Gregorian year and month.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct YearMonth(Date);
impl YearMonth {
    pub fn new(year: i32, month: Month) -> Result<Self, time::error::ComponentRange> {
        Date::from_calendar_date(year, month, 1).map(Self)
    }
    pub fn from_date(date: Date) -> Self {
        Self(date.replace_day(1).unwrap())
    }
    pub fn year(self) -> i32 {
        self.0.year()
    }
    pub fn month(self) -> Month {
        self.0.month()
    }
    pub fn first_day(self) -> Date {
        self.0
    }
    pub fn checked_add_months(self, delta: i32) -> Option<Self> {
        let n = self
            .year()
            .checked_mul(12)?
            .checked_add(self.month() as i32 - 1)?
            .checked_add(delta)?;
        Self::new(
            n.div_euclid(12),
            Month::try_from((n.rem_euclid(12) + 1) as u8).ok()?,
        )
        .ok()
    }
    fn days(self) -> u8 {
        self.0.month().length(self.year())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    Single(Option<Date>),
    Range(RangeSelection),
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RangeSelection {
    #[default]
    Empty,
    Start(Date),
    /// Inclusive endpoints; rendering also accepts externally supplied reversed endpoints.
    Complete {
        start: Date,
        end: Date,
    },
}
#[derive(Clone, Debug)]
pub struct State {
    pub visible_month: YearMonth,
    focused: Option<Date>,
    pub(crate) caption_popup_open: bool,
}
impl State {
    pub fn new(visible_month: YearMonth) -> Self {
        Self {
            visible_month,
            focused: None,
            caption_popup_open: false,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CaptionLayout {
    #[default]
    Label,
    Dropdowns,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CaptionOrder {
    #[default]
    MonthYear,
    YearMonth,
}
#[derive(Clone, Copy, Debug)]
pub enum Label {
    MonthTitle(YearMonth),
    MonthName(Month),
    Year(i32),
    WeekdayShort(Weekday),
    DayNumber(Date),
    DateAccessible(Date),
    PreviousMonth,
    NextMonth,
    ChooseMonth,
    ChooseYear,
}
/// English fallback, also usable by applications for labels they do not override.
pub fn default_label(label: Label) -> String {
    match label {
        Label::MonthTitle(m) => format!("{} {}", m.month(), m.year()),
        Label::MonthName(m) => m.to_string(),
        Label::Year(y) => y.to_string(),
        Label::WeekdayShort(w) => {
            ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"][w.number_days_from_monday() as usize].into()
        }
        Label::DayNumber(d) => d.day().to_string(),
        Label::DateAccessible(d) => d.to_string(),
        Label::PreviousMonth => "Previous month".into(),
        Label::NextMonth => "Next month".into(),
        Label::ChooseMonth => "Choose month".into(),
        Label::ChooseYear => "Choose year".into(),
    }
}
pub struct Options<'a> {
    pub first_weekday: Weekday,
    /// Clamped to 1..=12; months beyond Date::MAX are omitted.
    pub number_of_months: usize,
    pub fixed_weeks: bool,
    pub show_outside_days: bool,
    pub caption_layout: CaptionLayout,
    pub caption_order: CaptionOrder,
    /// Inclusive dropdown year range. None uses visible year +/- 100.
    pub year_range: Option<std::ops::RangeInclusive<i32>>,
    pub min_date: Option<Date>,
    pub max_date: Option<Date>,
    pub is_date_disabled: Option<&'a dyn Fn(Date) -> bool>,
    pub today: Option<Date>,
    pub labels: Option<&'a dyn Fn(Label) -> String>,
}
impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            first_weekday: Weekday::Sunday,
            number_of_months: 1,
            fixed_weeks: false,
            show_outside_days: true,
            caption_layout: CaptionLayout::Label,
            caption_order: CaptionOrder::MonthYear,
            year_range: None,
            min_date: None,
            max_date: None,
            is_date_disabled: None,
            today: None,
            labels: None,
        }
    }
}
impl Options<'_> {
    fn label(&self, label: Label) -> String {
        self.labels
            .map_or_else(|| default_label(label), |f| f(label))
    }
    fn enabled(&self, d: Date) -> bool {
        self.min_date.is_none_or(|min| d >= min)
            && self.max_date.is_none_or(|max| d <= max)
            && self.is_date_disabled.is_none_or(|f| !f(d))
    }
    fn valid_range(&self, a: Date, b: Date) -> bool {
        let (start, end) = ordered(a, b);
        if !self.enabled(start) || !self.enabled(end) {
            return false;
        }
        if self.is_date_disabled.is_none() {
            return true;
        }
        let mut d = start;
        while d < end {
            d = d.next_day().unwrap();
            if !self.enabled(d) {
                return false;
            }
        }
        true
    }
}
#[derive(Clone)]
pub struct Style {
    pub theme: Theme,
    /// Fixed trigger widths, independent of the selected labels.
    pub month_select_width: f32,
    pub year_select_width: f32,
    /// Minimum date-cell side length. Cells grow uniformly when the header needs more width.
    pub day_size: f32,
    pub row_gap: f32,
    pub month_gap: f32,
    /// Spacing between header controls, including navigation and the caption.
    pub header_gap: f32,
    pub padding: i8,
    pub text_size: f32,
    pub corner_radius: egui::CornerRadius,
    pub background: Color32,
    pub text_color: Color32,
    pub muted_text_color: Color32,
    /// Dates belonging to the previous or next month.
    pub outside_text_color: Color32,
    pub disabled_text_color: Color32,
    pub hover_bg: Color32,
    pub selected_bg: Color32,
    pub selected_text_color: Color32,
    pub range_bg: Color32,
    pub today_stroke: Stroke,
    pub border_stroke: Stroke,
    pub focus_color: Color32,
    pub disabled: bool,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            theme: theme.clone(),
            month_select_width: 112.0,
            year_select_width: 72.0,
            day_size: 32.0,
            row_gap: 0.0,
            month_gap: 24.0,
            header_gap: 4.0,
            padding: 12,
            text_size: 14.0,
            corner_radius: egui::CornerRadius::same(8),
            background: theme.colors.bg.base,
            text_color: theme.colors.fg.base,
            muted_text_color: theme.colors.gray.l9,
            outside_text_color: theme.colors.fg.base.gamma_multiply(0.4),
            disabled_text_color: theme.colors.fg.base.gamma_multiply(0.25),
            hover_bg: theme.colors.fg.base.gamma_multiply(0.08),
            selected_bg: theme.colors.primary.normal(),
            selected_text_color: theme
                .colors
                .primary
                .to_button_palette(theme, ButtonVariant::Solid)
                .text_color,
            range_bg: theme.colors.primary.normal().gamma_multiply(0.12),
            today_stroke: Stroke::new(1.0, theme.colors.gray.l6),
            border_stroke: Stroke::new(1.0, theme.colors.gray.l3),
            focus_color: theme.colors.primary.normal(),
            disabled: false,
        }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
pub struct CalendarResponse {
    pub response: Response,
    /// A single date or both range endpoints were activated this frame, even if unchanged.
    pub selection_finished: bool,
    pub month_changed: bool,
}
fn ordered(a: Date, b: Date) -> (Date, Date) {
    (a.min(b), a.max(b))
}
fn activate(selection: &mut Selection, d: Date, options: &Options<'_>) -> bool {
    if !options.enabled(d) {
        return false;
    }
    match selection {
        Selection::Single(value) => {
            *value = Some(d);
            true
        }
        Selection::Range(range) => match *range {
            RangeSelection::Start(start) => {
                if !options.valid_range(start, d) {
                    return false;
                }
                let (start, end) = ordered(start, d);
                *range = RangeSelection::Complete { start, end };
                true
            }
            _ => {
                *range = RangeSelection::Start(d);
                false
            }
        },
    }
}
fn bounds(selection: Selection, preview: Option<Date>) -> Option<(Date, Date)> {
    match selection {
        Selection::Single(Some(d)) => Some((d, d)),
        Selection::Range(RangeSelection::Start(d)) => Some(ordered(d, preview.unwrap_or(d))),
        Selection::Range(RangeSelection::Complete { start, end }) => Some(ordered(start, end)),
        _ => None,
    }
}
pub fn render(
    ui: &mut Ui,
    id: Id,
    selection: &mut Selection,
    state: &mut State,
    style: Style,
) -> CalendarResponse {
    render_with_options(ui, id, selection, state, style, Options::default())
}

pub fn render_with_options(
    ui: &mut Ui,
    id: Id,
    selection: &mut Selection,
    state: &mut State,
    style: Style,
    options: Options<'_>,
) -> CalendarResponse {
    state.caption_popup_open = false;
    let before = *selection;
    let month_before = state.visible_month;
    let mut finished = false;
    let mut all_cells = Vec::new();
    let mut response = ui
        .push_id(id, |ui| {
            ui.add_enabled_ui(!style.disabled, |ui| {
                egui::Frame::new()
                    .fill(style.background)
                    .stroke(style.border_stroke)
                    .corner_radius(style.corner_radius)
                    .inner_margin(style.padding)
                    .show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = style.month_gap;
                            let count = options.number_of_months.clamp(1, 12);
                            let first = state.visible_month;
                            let last = first
                                .checked_add_months(count as i32 - 1)
                                .unwrap_or(YearMonth::from_date(Date::MAX));
                            for index in 0..count {
                                let Some(month) = first.checked_add_months(index as i32) else {
                                    break;
                                };
                                ui.push_id(index, |ui| {
                                    ui.vertical(|ui| {
                                        let width = (style.day_size * 7.0).max(
                                            if options.caption_layout == CaptionLayout::Dropdowns {
                                                style.month_select_width
                                                    + style.year_select_width
                                                    + 48.0
                                                    + 3.0 * style.header_gap
                                            } else {
                                                48.0 + 2.0 * style.header_gap
                                            },
                                        );
                                        let cell_size = width / 7.0;
                                        ui.set_width(width);
                                        let (header, _) = ui.allocate_exact_size(
                                            Vec2::new(width, style.day_size.max(24.0)),
                                            Sense::hover(),
                                        );
                                        let nav_size = Vec2::splat(24.0);
                                        let left = Rect::from_center_size(
                                            egui::pos2(header.left() + 12.0, header.center().y),
                                            nav_size,
                                        );
                                        let right = Rect::from_center_size(
                                            egui::pos2(header.right() - 12.0, header.center().y),
                                            nav_size,
                                        );
                                        if index == 0 {
                                            let prev = first.checked_add_months(-1);
                                            ui.scope_builder(
                                                egui::UiBuilder::new().max_rect(left).layout(
                                                    egui::Layout::left_to_right(
                                                        egui::Align::Center,
                                                    ),
                                                ),
                                                |ui| {
                                                    if navigation_button(
                                                        ui,
                                                        false,
                                                        prev.is_some(),
                                                        &style,
                                                        &options,
                                                    )
                                                    .clicked()
                                                    {
                                                        state.visible_month = prev.unwrap();
                                                    }
                                                },
                                            );
                                        }
                                        if month == last {
                                            let next = last
                                                .checked_add_months(1)
                                                .and_then(|_| first.checked_add_months(1));
                                            ui.scope_builder(
                                                egui::UiBuilder::new().max_rect(right).layout(
                                                    egui::Layout::left_to_right(
                                                        egui::Align::Center,
                                                    ),
                                                ),
                                                |ui| {
                                                    if navigation_button(
                                                        ui,
                                                        true,
                                                        next.is_some(),
                                                        &style,
                                                        &options,
                                                    )
                                                    .clicked()
                                                    {
                                                        state.visible_month = next.unwrap();
                                                    }
                                                },
                                            );
                                        }
                                        let caption_width =
                                            if options.caption_layout == CaptionLayout::Dropdowns {
                                                style.month_select_width
                                                    + style.year_select_width
                                                    + style.header_gap
                                            } else {
                                                width - 48.0 - 2.0 * style.header_gap
                                            };
                                        let caption_rect = Rect::from_center_size(
                                            header.center(),
                                            Vec2::new(caption_width, header.height()),
                                        );
                                        let layout =
                                            if options.caption_layout == CaptionLayout::Label {
                                                egui::Layout::centered_and_justified(
                                                    egui::Direction::LeftToRight,
                                                )
                                            } else {
                                                egui::Layout::left_to_right(egui::Align::Center)
                                            };
                                        ui.scope_builder(
                                            egui::UiBuilder::new()
                                                .max_rect(caption_rect)
                                                .layout(layout),
                                            |ui| {
                                                ui.spacing_mut().item_spacing.x = style.header_gap;
                                                caption(ui, month, index, state, &style, &options);
                                            },
                                        );
                                        let offset =
                                            (month.first_day().weekday().number_days_from_monday()
                                                as i32
                                                - options.first_weekday.number_days_from_monday()
                                                    as i32)
                                                .rem_euclid(7);
                                        let rows = if options.fixed_weeks {
                                            6
                                        } else {
                                            (offset + month.days() as i32 + 6) / 7
                                        };
                                        let (rect, _) = ui.allocate_exact_size(
                                            Vec2::new(
                                                width,
                                                cell_size
                                                    + rows as f32 * (cell_size + style.row_gap),
                                            ),
                                            Sense::hover(),
                                        );
                                        let font = FontId::proportional(style.text_size);
                                        let mut weekday = options.first_weekday;
                                        for col in 0..7 {
                                            let center = rect.min
                                                + Vec2::new(
                                                    (col as f32 + 0.5) * cell_size,
                                                    cell_size / 2.0,
                                                );
                                            ui.painter().text(
                                                center,
                                                Align2::CENTER_CENTER,
                                                options.label(Label::WeekdayShort(weekday)),
                                                font.clone(),
                                                style.muted_text_color,
                                            );
                                            weekday = weekday.next();
                                        }
                                        let mut cells = Vec::new();
                                        for cell in 0..rows * 7 {
                                            let Some(d) = month.first_day().checked_add(
                                                time::Duration::days((cell - offset) as i64),
                                            ) else {
                                                continue;
                                            };
                                            let outside = YearMonth::from_date(d) != month;
                                            if outside
                                                && (!options.show_outside_days
                                                    || (YearMonth::from_date(d) >= first
                                                        && YearMonth::from_date(d) <= last))
                                            {
                                                continue;
                                            }
                                            let min = rect.min
                                                + Vec2::new(
                                                    (cell % 7) as f32 * cell_size,
                                                    cell_size
                                                        + (cell / 7) as f32
                                                            * (cell_size + style.row_gap),
                                                );
                                            let cell_rect =
                                                Rect::from_min_size(min, Vec2::splat(cell_size));
                                            let enabled = ui.is_enabled() && options.enabled(d);
                                            let response = ui.interact(
                                                cell_rect,
                                                id.with((index, d.to_julian_day())),
                                                if enabled {
                                                    Sense::click()
                                                } else {
                                                    Sense::hover()
                                                },
                                            );
                                            response.widget_info(|| {
                                                egui::WidgetInfo::labeled(
                                                    egui::Role::Button,
                                                    enabled,
                                                    options.label(Label::DateAccessible(d)),
                                                )
                                            });
                                            if enabled && response.clicked() {
                                                finished |= activate(selection, d, &options);
                                                state.focused = Some(d);
                                                response.request_focus();
                                                if outside {
                                                    state.visible_month = YearMonth::from_date(d);
                                                }
                                            }
                                            if enabled && response.has_focus() {
                                                ui.memory_mut(|m| {
                                                    m.set_focus_lock_filter(
                                                        response.id,
                                                        egui::EventFilter {
                                                            horizontal_arrows: true,
                                                            vertical_arrows: true,
                                                            ..Default::default()
                                                        },
                                                    )
                                                });
                                                state.focused = Some(d);
                                                if let Some(target) =
                                                    keyboard_target(ui, d, options.first_weekday)
                                                        .and_then(|target| {
                                                            next_enabled(target, d, &options)
                                                        })
                                                {
                                                    state.focused = Some(target);
                                                    let target_month = YearMonth::from_date(target);
                                                    if target_month < first || target_month > last {
                                                        state.visible_month = target_month;
                                                    }
                                                    ui.memory_mut(|m| {
                                                        m.request_focus(id.with((
                                                            if target_month >= first
                                                                && target_month <= last
                                                            {
                                                                (target_month.year() - first.year())
                                                                    * 12
                                                                    + target_month.month() as i32
                                                                    - first.month() as i32
                                                            } else {
                                                                0
                                                            }
                                                                as usize,
                                                            target.to_julian_day(),
                                                        )))
                                                    });
                                                }
                                            }
                                            cells.push((d, outside, enabled, cell_rect, response));
                                        }
                                        all_cells.extend(cells);
                                    });
                                });
                            }
                        });
                    })
                    .response
            })
            .inner
        })
        .inner;
    let preview = all_cells
        .iter()
        .find(|(_, _, enabled, _, r)| *enabled && r.hovered())
        .map(|c| c.0)
        .filter(|d| match *selection {
            Selection::Range(RangeSelection::Start(start)) => options.valid_range(start, *d),
            _ => false,
        });
    let selected = bounds(*selection, preview);
    for (d, outside, enabled, cell_rect, response) in all_cells {
        let in_range = selected.is_some_and(|(a, b)| d >= a && d <= b);
        let endpoint = selected.is_some_and(|(a, b)| d == a || d == b);
        if in_range && matches!(selection, Selection::Range(_)) {
            let (a, b) = selected.unwrap();
            let mut band = cell_rect;
            if d == a {
                band.min.x = band.center().x;
            }
            if d == b {
                band.max.x = band.center().x;
            }
            let mut radius = egui::CornerRadius::ZERO;
            if d.weekday() == options.first_weekday {
                radius.nw = style.corner_radius.nw;
                radius.sw = style.corner_radius.sw;
            }
            if d.weekday() == options.first_weekday.previous() {
                radius.ne = style.corner_radius.ne;
                radius.se = style.corner_radius.se;
            }
            ui.painter().rect_filled(band, radius, style.range_bg);
        }
        if endpoint {
            ui.painter()
                .rect_filled(cell_rect, style.corner_radius, style.selected_bg);
        } else if enabled && response.hovered() {
            ui.painter()
                .rect_filled(cell_rect, style.corner_radius, style.hover_bg);
        }
        if options.today == Some(d) {
            ui.painter().rect_stroke(
                cell_rect.shrink(1.0),
                style.corner_radius,
                style.today_stroke,
                StrokeKind::Inside,
            );
        }
        let color = if !enabled {
            style.disabled_text_color
        } else if endpoint {
            style.selected_text_color
        } else if outside {
            style.outside_text_color
        } else {
            style.text_color
        };
        ui.painter().text(
            cell_rect.center(),
            Align2::CENTER_CENTER,
            options.label(Label::DayNumber(d)),
            FontId::proportional(style.text_size),
            color,
        );
        super::focus::ring(
            ui,
            &response,
            cell_rect,
            style.corner_radius,
            style.focus_color,
            super::focus::Placement::Inside,
        );
    }
    if before != *selection {
        response.mark_changed();
    }
    CalendarResponse {
        response,
        selection_finished: finished,
        month_changed: month_before != state.visible_month,
    }
}
fn navigation_button(
    ui: &mut Ui,
    next: bool,
    enabled: bool,
    style: &Style,
    options: &Options<'_>,
) -> Response {
    let mut button_style =
        icon_button::Style::new(&style.theme, None, ButtonSize::Sm, ButtonVariant::Ghost);
    button_style.width = 24.0;
    button_style.height = 24.0;
    let icon = if next {
        egui::include_image!("../../assets/lucide/chevron-right.svg")
    } else {
        egui::include_image!("../../assets/lucide/chevron-left.svg")
    };
    let label = options.label(if next {
        Label::NextMonth
    } else {
        Label::PreviousMonth
    });
    let response = ui
        .add_enabled_ui(enabled, |ui| {
            icon_button::render(ui, icon, &label, button_style)
        })
        .inner;
    response.on_hover_text(label)
}

fn caption(
    ui: &mut Ui,
    month: YearMonth,
    index: usize,
    state: &mut State,
    style: &Style,
    options: &Options<'_>,
) {
    if options.caption_layout == CaptionLayout::Label {
        text::render(
            ui,
            &options.label(Label::MonthTitle(month)),
            text::Style::new(&style.theme)
                .size(style.text_size + 2.0)
                .color(style.text_color)
                .selectable(false),
        );
        return;
    }
    let mut year = month.year();
    let mut m = month.month();
    let range = options
        .year_range
        .clone()
        .unwrap_or(year.saturating_sub(100)..=year.saturating_add(100));
    for year_first in match options.caption_order {
        CaptionOrder::MonthYear => [false, true],
        CaptionOrder::YearMonth => [true, false],
    } {
        let width = if year_first {
            style.year_select_width
        } else {
            style.month_select_width
        };
        let select_style = select::Style::new(
            &style.theme,
            Some(style.theme.colors.primary),
            select::SelectSize::Sm,
        )
        .width(Some(width));
        let label = options.label(if year_first {
            Label::Year(year)
        } else {
            Label::MonthName(m)
        });
        let popup_id = ui.make_persistent_id(if year_first { "year" } else { "month" });
        let response = select::render(ui, popup_id, &label, select_style.clone(), |ui| {
            if year_first {
                let start = (*range.start()).max(Date::MIN.year());
                let end = (*range.end()).min(Date::MAX.year());
                for value in start..=end {
                    if select::render_item(
                        ui,
                        &options.label(Label::Year(value)),
                        year == value,
                        &select_style,
                        &style.theme,
                    )
                    .clicked()
                    {
                        year = value;
                        ui.close();
                    }
                }
            } else {
                for n in 1..=12 {
                    let value = Month::try_from(n).unwrap();
                    if select::render_item(
                        ui,
                        &options.label(Label::MonthName(value)),
                        m == value,
                        &select_style,
                        &style.theme,
                    )
                    .clicked()
                    {
                        m = value;
                        ui.close();
                    }
                }
            }
        });
        state.caption_popup_open |= egui::Popup::is_id_open(ui.ctx(), popup_id);
        response.on_hover_text(options.label(if year_first {
            Label::ChooseYear
        } else {
            Label::ChooseMonth
        }));
    }

    if (year, m) != (month.year(), month.month()) {
        if let Some(first) = YearMonth::new(year, m)
            .ok()
            .and_then(|m| m.checked_add_months(-(index as i32)))
        {
            state.visible_month = first;
        }
    }
}
fn keyboard_target(ui: &mut Ui, d: Date, first_weekday: Weekday) -> Option<Date> {
    let key = [
        Key::ArrowLeft,
        Key::ArrowRight,
        Key::ArrowUp,
        Key::ArrowDown,
        Key::Home,
        Key::End,
        Key::PageUp,
        Key::PageDown,
    ]
    .into_iter()
    .find(|key| ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, *key)))?;
    let offset = (d.weekday().number_days_from_monday() as i64
        - first_weekday.number_days_from_monday() as i64)
        .rem_euclid(7);
    let days = match key {
        Key::ArrowLeft => -1,
        Key::ArrowRight => 1,
        Key::ArrowUp => -7,
        Key::ArrowDown => 7,
        Key::Home => -offset,
        Key::End => 6 - offset,
        _ => {
            let m = YearMonth::from_date(d).checked_add_months(if key == Key::PageUp {
                -1
            } else {
                1
            })?;
            return m.first_day().replace_day(d.day().min(m.days())).ok();
        }
    };
    d.checked_add(time::Duration::days(days))
}

fn next_enabled(mut target: Date, from: Date, options: &Options<'_>) -> Option<Date> {
    let forward = target >= from;
    // Stay within the caller's selectable bounds even with a custom predicate.
    loop {
        if options.min_date.is_some_and(|min| target < min)
            || options.max_date.is_some_and(|max| target > max)
        {
            return None;
        }
        if options.enabled(target) {
            return Some(target);
        }
        target = if forward {
            target.next_day()?
        } else {
            target.previous_day()?
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn date(y: i32, m: u8, d: u8) -> Date {
        Date::from_calendar_date(y, Month::try_from(m).unwrap(), d).unwrap()
    }
    #[test]
    fn month_arithmetic_handles_leap_year_and_limits() {
        let feb = YearMonth::from_date(date(2024, 2, 29));
        assert_eq!(feb.days(), 29);
        assert_eq!(
            feb.checked_add_months(-2).unwrap(),
            YearMonth::from_date(date(2023, 12, 1))
        );
        assert!(
            YearMonth::from_date(Date::MIN)
                .checked_add_months(-1)
                .is_none()
        );
        assert!(
            YearMonth::from_date(Date::MAX)
                .checked_add_months(1)
                .is_none()
        );
    }
    #[test]
    fn range_is_inclusive_sorted_and_restarts() {
        let mut selected = Selection::Range(RangeSelection::Empty);
        let a = date(2026, 11, 17);
        let b = date(2026, 12, 2);
        assert!(!activate(&mut selected, b, &Options::default()));
        assert!(activate(&mut selected, a, &Options::default()));
        assert_eq!(
            selected,
            Selection::Range(RangeSelection::Complete { start: a, end: b })
        );
        assert!(!activate(&mut selected, a, &Options::default()));
        assert!(activate(&mut selected, a, &Options::default()));
        assert_eq!(
            selected,
            Selection::Range(RangeSelection::Complete { start: a, end: a })
        );
    }
    #[test]
    fn forbidden_interior_keeps_pending_start() {
        let a = date(2026, 11, 17);
        let b = date(2026, 11, 20);
        let disabled = |d| d == date(2026, 11, 18);
        let options = Options {
            is_date_disabled: Some(&disabled),
            ..Default::default()
        };
        let mut selected = Selection::Range(RangeSelection::Start(a));
        assert!(!activate(&mut selected, b, &options));
        assert_eq!(selected, Selection::Range(RangeSelection::Start(a)));
        assert!(!options.valid_range(b, a));
    }
    #[test]
    fn bounds_and_repeated_single_activation() {
        let d = date(2026, 11, 17);
        let options = Options {
            min_date: Some(d),
            max_date: Some(d),
            ..Default::default()
        };
        let mut selected = Selection::Single(Some(d));
        assert!(activate(&mut selected, d, &options));
        assert!(!activate(&mut selected, d.next_day().unwrap(), &options));
        assert_eq!(selected, Selection::Single(Some(d)));
    }
    fn frame(
        ctx: &egui::Context,
        selection: &mut Selection,
        state: &mut State,
        events: Vec<egui::Event>,
        disabled: bool,
    ) -> (CalendarResponse, egui::FullOutput) {
        let theme =
            crate::theme::UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
                .unwrap();
        let mut result = None;
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(800.0, 600.0),
                )),
                events,
                ..Default::default()
            },
            |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    result = Some(render_with_options(
                        ui,
                        Id::unique("test-calendar"),
                        selection,
                        state,
                        Style::new(&theme.light).disabled(disabled),
                        Options {
                            number_of_months: 2,
                            ..Default::default()
                        },
                    ));
                });
            },
        );
        output.textures_delta.clear();
        (result.unwrap(), output)
    }
    #[test]
    fn keyboard_selects_and_moves_between_months() {
        let ctx = egui::Context::default();
        let d = date(2026, 11, 30);
        let mut state = State::new(YearMonth::from_date(d));
        let mut selection = Selection::Single(None);
        frame(&ctx, &mut selection, &mut state, vec![], false);
        ctx.memory_mut(|m| {
            m.request_focus(Id::unique("test-calendar").with((0usize, d.to_julian_day())))
        });
        frame(&ctx, &mut selection, &mut state, vec![], false);
        let key = |key| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        frame(
            &ctx,
            &mut selection,
            &mut state,
            vec![key(Key::ArrowRight)],
            false,
        );
        let (r, _) = frame(
            &ctx,
            &mut selection,
            &mut state,
            vec![key(Key::Enter)],
            false,
        );
        assert_eq!(selection, Selection::Single(Some(date(2026, 12, 1))));
        assert!(r.selection_finished && r.response.changed());
        let (r, _) = frame(
            &ctx,
            &mut selection,
            &mut state,
            vec![key(Key::Enter)],
            false,
        );
        assert!(r.selection_finished && !r.response.changed());
    }
    #[test]
    fn disabled_calendar_ignores_keyboard_activation() {
        let ctx = egui::Context::default();
        let d = date(2026, 11, 17);
        let mut state = State::new(YearMonth::from_date(d));
        let mut selection = Selection::Single(None);
        frame(&ctx, &mut selection, &mut state, vec![], true);
        ctx.memory_mut(|m| {
            m.request_focus(Id::unique("test-calendar").with((0usize, d.to_julian_day())))
        });
        let (r, _) = frame(
            &ctx,
            &mut selection,
            &mut state,
            vec![egui::Event::Key {
                key: Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            true,
        );
        assert_eq!(selection, Selection::Single(None));
        assert!(!r.selection_finished);
    }
    #[test]
    fn pointer_range_preview_does_not_commit_and_layout_is_compact() {
        let ctx = egui::Context::default();
        let mut state = State::new(YearMonth::from_date(date(2026, 11, 1)));
        let mut selection = Selection::Range(RangeSelection::Empty);
        let (response, output) = frame(&ctx, &mut selection, &mut state, vec![], false);
        assert!(
            response.response.rect.width() < 550.0,
            "{:?}",
            response.response.rect
        );
        assert!(
            response.response.rect.height() < 350.0,
            "{:?}",
            response.response.rect
        );
        let position = |text: &str, occurrence: usize| {
            output
                .shapes
                .iter()
                .filter_map(|s| match &s.shape {
                    egui::epaint::Shape::Text(t) if t.galley.text() == text => {
                        Some(t.pos + t.galley.size() / 2.0)
                    }
                    _ => None,
                })
                .nth(occurrence)
                .unwrap()
        };
        let november = position("November 2026", 0);
        let december = position("December 2026", 0);
        assert!((november.y - december.y).abs() < 1.0);
        // Wednesday is the center column of each Sunday-first month grid.
        assert!((november.x - position("We", 0).x).abs() < 1.0);
        assert!((december.x - position("We", 1).x).abs() < 1.0);
        let start = position("17", 0);
        let end = position("17", 1);
        let click = |ctx: &egui::Context, selection: &mut Selection, state: &mut State, pos| {
            frame(
                ctx,
                selection,
                state,
                vec![egui::Event::PointerMoved(pos)],
                false,
            );
            frame(
                ctx,
                selection,
                state,
                vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                }],
                false,
            );
            frame(
                ctx,
                selection,
                state,
                vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                }],
                false,
            )
            .0
        };
        let r = click(&ctx, &mut selection, &mut state, start);
        assert!(r.response.changed() && !r.selection_finished);
        let r = frame(
            &ctx,
            &mut selection,
            &mut state,
            vec![egui::Event::PointerMoved(end)],
            false,
        )
        .0;
        assert!(!r.response.changed());
        assert_eq!(
            selection,
            Selection::Range(RangeSelection::Start(date(2026, 11, 17)))
        );
        let r = click(&ctx, &mut selection, &mut state, end);
        assert!(r.response.changed() && r.selection_finished);
        assert_eq!(
            selection,
            Selection::Range(RangeSelection::Complete {
                start: date(2026, 11, 17),
                end: date(2026, 12, 17)
            })
        );
    }
    #[test]
    fn dropdowns_localization_and_extreme_dates_render() {
        let ctx = egui::Context::default();
        let theme =
            crate::theme::UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
                .unwrap();
        for initial in [Date::MIN, date(2026, 11, 17), Date::MAX] {
            let mut state = State::new(YearMonth::from_date(initial));
            let mut selection = Selection::Single(None);
            let labels = |label| match label {
                Label::MonthName(_) => "Localized".to_owned(),
                other => default_label(other),
            };
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(800.0, 600.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    let r = render_with_options(
                        ui,
                        Id::unique("dropdown"),
                        &mut selection,
                        &mut state,
                        Style::new(&theme.light),
                        Options {
                            caption_layout: CaptionLayout::Dropdowns,
                            labels: Some(&labels),
                            fixed_weeks: true,
                            ..Default::default()
                        },
                    );
                    assert!(r.response.rect.width() < 300.0, "{:?}", r.response.rect);
                    assert!(r.response.rect.height() < 350.0, "{:?}", r.response.rect);
                },
            );
            output.textures_delta.clear();
            assert!(output.shapes.iter().any(|s| matches!(&s.shape, egui::epaint::Shape::Text(t) if t.galley.text() == "Localized")));
        }
    }
    #[test]
    fn range_dropdown_jumps_preserve_start_and_link_months() {
        let theme =
            crate::theme::UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
                .unwrap();
        for (months, panel) in [(1, 0), (2, 0), (2, 1)] {
            let ctx = egui::Context::default();
            let start = date(2026, 11, 17);
            let mut selection = Selection::Range(RangeSelection::Start(start));
            let mut state = State::new(YearMonth::from_date(start));
            let mut render_frame = |events| {
                let mut response = None;
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(
                            egui::Pos2::ZERO,
                            Vec2::new(1000.0, 800.0),
                        )),
                        events,
                        ..Default::default()
                    },
                    |ui| {
                        response = Some(render_with_options(
                            ui,
                            Id::unique("range-jump"),
                            &mut selection,
                            &mut state,
                            Style::new(&theme.light),
                            Options {
                                number_of_months: months,
                                caption_layout: CaptionLayout::Dropdowns,
                                year_range: Some(2026..=2028),
                                ..Default::default()
                            },
                        ));
                    },
                );
                output.textures_delta.clear();
                assert!(!response.unwrap().response.changed());
                output
            };
            let position = |output: &egui::FullOutput, text: &str, occurrence: usize| {
                output
                    .shapes
                    .iter()
                    .filter_map(|s| match &s.shape {
                        egui::epaint::Shape::Text(t) if t.galley.text() == text => {
                            Some(t.pos + t.galley.size() / 2.0)
                        }
                        _ => None,
                    })
                    .nth(occurrence)
                    .unwrap_or_else(|| panic!("missing label: {text}"))
            };
            let mut output = render_frame(vec![]);
            let mut click = |pos| {
                render_frame(vec![egui::Event::PointerMoved(pos)]);
                render_frame(vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                }]);
                render_frame(vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                }]);
                render_frame(vec![])
            };
            output = click(position(
                &output,
                if panel == 0 { "November" } else { "December" },
                0,
            ));
            output = click(position(&output, "January", 0));
            // Right-panel January makes the left panel December of the prior year.
            output = click(position(&output, "2026", 0));
            click(position(&output, "2028", 0));
            assert_eq!(selection, Selection::Range(RangeSelection::Start(start)));
            let target = YearMonth::from_date(date(2028, 1, 1));
            assert_eq!(
                state.visible_month,
                target.checked_add_months(-(panel as i32)).unwrap()
            );
        }
    }
}
