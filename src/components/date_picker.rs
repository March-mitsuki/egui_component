//! Select-like date picker backed by Calendar. Selection updates immediately;
//! Escape/outside clicks close the popup without rolling back a partial range.
//! On opening, a nonempty selection determines the initial visible month.
//!
//! ```
//! use egui_component::components::{calendar::{Month, Selection, YearMonth}, date_picker};
//! # fn show(ui: &mut egui::Ui, theme: &egui_component::theme::Theme) {
//! // Store selection and state in the application between frames.
//! let mut selection = Selection::Single(None);
//! let mut state = date_picker::State::new(YearMonth::new(2026, Month::November).unwrap());
//! let result = date_picker::render(ui, ui.make_persistent_id("birthday"),
//!     &mut selection, &mut state, date_picker::Style::new(theme), Default::default());
//! if result.response.changed() { /* consume the selected date */ }
//! # }
//! ```
use super::{
    calendar::{self, RangeSelection, Selection, YearMonth},
    select,
};
use crate::theme::Theme;
use egui::{Id, Response, Ui};

#[derive(Clone, Debug)]
pub struct State {
    pub open: bool,
    pub calendar: calendar::State,
    was_open: bool,
}
impl State {
    pub fn new(month: YearMonth) -> Self {
        Self {
            open: false,
            calendar: calendar::State::new(month),
            was_open: false,
        }
    }
}
#[derive(Clone)]
pub struct Style {
    /// Trigger appearance and popup frame/alignment/gap. Menu height and close
    /// behavior are managed by Date Picker rather than Select's item-menu options.
    pub trigger: select::Style,
    pub calendar: calendar::Style,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        let mut calendar = calendar::Style::new(theme);
        // The popup frame owns the outer border.
        calendar.border_stroke = egui::Stroke::NONE;
        Self {
            trigger: select::Style::new(theme, Some(theme.colors.primary), select::SelectSize::Md)
                .width(Some(280.0)),
            calendar,
        }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.trigger.disabled = disabled;
        self.calendar.disabled = disabled;
        self
    }
}
pub struct Options<'a> {
    pub calendar: calendar::Options<'a>,
    pub placeholder: &'a str,
    /// Called for nonempty selections, including an unfinished range.
    pub format_selection: Option<&'a dyn Fn(&Selection) -> String>,
    pub close_on_selection: bool,
}
impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            calendar: Default::default(),
            placeholder: "Select date",
            format_selection: None,
            close_on_selection: true,
        }
    }
}
pub struct DatePickerResponse {
    pub response: Response,
    pub selection_finished: bool,
    pub open_changed: bool,
}
pub fn default_format(selection: &Selection) -> String {
    match selection {
        Selection::Single(Some(d)) => d.to_string(),
        Selection::Range(RangeSelection::Start(d)) => format!("{d} – …"),
        Selection::Range(RangeSelection::Complete { start, end }) => {
            format!("{} – {}", start.min(end), start.max(end))
        }
        _ => String::new(),
    }
}
fn anchor(selection: &Selection) -> Option<calendar::Date> {
    match selection {
        Selection::Single(d) => *d,
        Selection::Range(RangeSelection::Start(d)) => Some(*d),
        Selection::Range(RangeSelection::Complete { start, end }) => Some(*start.min(end)),
        _ => None,
    }
}
pub fn render(
    ui: &mut Ui,
    id: Id,
    selection: &mut Selection,
    state: &mut State,
    mut style: Style,
    options: Options<'_>,
) -> DatePickerResponse {
    let before = *selection;
    let was_open = state.was_open;
    let enabled = ui.is_enabled() && !style.trigger.disabled;
    if !enabled {
        state.open = false;
    }
    style.trigger.placeholder = options.placeholder.to_owned();
    let label = if anchor(selection).is_some() {
        options
            .format_selection
            .map_or_else(|| default_format(selection), |f| f(selection))
    } else {
        String::new()
    };
    let mut response = ui
        .push_id(id, |ui| {
            ui.add_enabled_ui(enabled, |ui| {
                select::render_trigger(ui, &label, &style.trigger, state.open)
            })
            .inner
        })
        .inner;
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_has_popup(egui::accesskit::HasPopup::Dialog);
    });
    if enabled && response.clicked() {
        state.open = !state.open;
    }
    if state.open && !was_open {
        if let Some(date) = anchor(selection) {
            state.calendar = calendar::State::new(YearMonth::from_date(date));
        }
    }
    let mut finished = false;
    // A Calendar select opens a separate foreground popup. Its clicks are not
    // outside-clicks for the containing date picker, even when it closes this frame.
    let child_popup_open = state.calendar.caption_popup_open;
    egui::Popup::from_response(&response)
        .id(id.with("popup"))
        .open_bool(&mut state.open)
        .align(style.trigger.popup_align)
        .gap(style.trigger.popup_gap)
        .frame(style.trigger.popup_frame)
        // Interior clicks (including range starts and navigation) must not close the picker.
        .close_behavior(if child_popup_open {
            egui::PopupCloseBehavior::IgnoreClicks
        } else {
            egui::PopupCloseBehavior::CloseOnClickOutside
        })
        .show(|ui| {
            // Like `aria-haspopup="dialog"` on the trigger: the calendar is a
            // non-modal dialog.
            ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
                node.set_role(egui::Role::Dialog);
            });
            let result = calendar::render(
                ui,
                id.with("calendar"),
                selection,
                &mut state.calendar,
                style.calendar,
                options.calendar,
            );
            finished = result.selection_finished;
        });
    if finished && options.close_on_selection {
        state.open = false;
    }
    if was_open && !state.open {
        response.request_focus();
    }
    if before != *selection {
        response.mark_changed();
    }
    let open_changed = was_open != state.open;
    state.was_open = state.open;
    DatePickerResponse {
        response,
        selection_finished: finished,
        open_changed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use calendar::{Date, Month};
    use egui::{Event, Modifiers, PointerButton, Pos2, Rect, Vec2};
    struct Harness {
        ctx: egui::Context,
        state: State,
        selection: Selection,
        disabled: bool,
        close: bool,
    }
    impl Harness {
        fn new(range: bool) -> Self {
            Self {
                ctx: Default::default(),
                state: State::new(YearMonth::new(2026, Month::November).unwrap()),
                selection: if range {
                    Selection::Range(RangeSelection::Empty)
                } else {
                    Selection::Single(None)
                },
                disabled: false,
                close: true,
            }
        }
        fn frame(&mut self, events: Vec<Event>) -> (DatePickerResponse, egui::FullOutput) {
            let theme =
                crate::theme::UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
                    .unwrap();
            let mut result = None;
            let mut output = self.ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 800.0))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    result = Some(render(
                        ui,
                        Id::unique("picker"),
                        &mut self.selection,
                        &mut self.state,
                        Style::new(&theme.light).disabled(self.disabled),
                        Options {
                            close_on_selection: self.close,
                            calendar: calendar::Options {
                                caption_layout: calendar::CaptionLayout::Dropdowns,
                                year_range: Some(2026..=2028),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    ));
                },
            );
            output.textures_delta.clear();
            (result.unwrap(), output)
        }
        fn click(&mut self, pos: Pos2) -> (DatePickerResponse, egui::FullOutput) {
            self.frame(vec![Event::PointerMoved(pos)]);
            self.frame(vec![Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            }]);
            self.frame(vec![Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            }])
        }
        fn open(&mut self) -> egui::FullOutput {
            let (r, _) = self.frame(vec![]);
            let (r, _) = self.click(r.response.rect.center());
            assert!(r.open_changed && self.state.open);
            self.frame(vec![]).1
        }
    }
    fn position(output: &egui::FullOutput, label: &str) -> Pos2 {
        output
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::epaint::Shape::Text(t) if t.galley.text() == label => {
                    Some(t.pos + t.galley.size() / 2.0)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing label {label}"))
    }
    #[test]
    fn single_commits_closes_and_reopens_at_selection() {
        let mut h = Harness::new(false);
        let output = h.open();
        let (r, _) = h.click(position(&output, "17"));
        assert!(r.response.changed() && r.selection_finished && r.open_changed);
        assert!(!h.state.open);
        h.state.calendar.visible_month = YearMonth::new(2020, Month::January).unwrap();
        h.open();
        assert_eq!(
            h.state.calendar.visible_month,
            YearMonth::new(2026, Month::November).unwrap()
        );
    }
    #[test]
    fn range_navigation_does_not_close_parent_and_escape_keeps_start() {
        let mut h = Harness::new(true);
        let output = h.open();
        let (r, _) = h.click(position(&output, "17"));
        assert!(r.response.changed() && !r.selection_finished && h.state.open);
        let output = h.frame(vec![]).1;
        h.click(position(&output, "November"));
        let output = h.frame(vec![]).1;
        h.click(position(&output, "January"));
        h.frame(vec![]);
        assert!(h.state.open, "month selection must not close the picker");
        assert_eq!(h.state.calendar.visible_month.month(), Month::January);
        let (r, _) = h.frame(vec![Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }]);
        assert!(r.open_changed && !h.state.open);
        assert_eq!(
            h.selection,
            Selection::Range(RangeSelection::Start(
                Date::from_calendar_date(2026, Month::November, 17).unwrap()
            ))
        );
    }
    #[test]
    fn range_finishes_on_second_click_and_can_stay_open() {
        let mut h = Harness::new(true);
        h.close = false;
        let output = h.open();
        h.click(position(&output, "17"));
        let output = h.frame(vec![]).1;
        let (r, _) = h.click(position(&output, "20"));
        assert!(r.selection_finished && r.response.changed() && h.state.open);
        h.click(Pos2::new(900.0, 700.0));
        assert!(!h.state.open);
    }
    #[test]
    fn disabled_trigger_cannot_open() {
        let mut h = Harness::new(false);
        h.disabled = true;
        let (r, _) = h.frame(vec![]);
        h.click(r.response.rect.center());
        assert!(!h.state.open);
        h.state.open = true;
        h.frame(vec![]);
        assert!(!h.state.open);
    }
    #[test]
    fn range_default_closes_only_on_completion() {
        let mut h = Harness::new(true);
        let output = h.open();
        h.click(position(&output, "17"));
        assert!(h.state.open);
        let output = h.frame(vec![]).1;
        let (r, _) = h.click(position(&output, "20"));
        assert!(r.selection_finished && r.open_changed && !h.state.open);
    }
    #[test]
    fn keyboard_opens_and_escape_closes_without_changing_value() {
        let mut h = Harness::new(false);
        let (r, _) = h.frame(vec![]);
        r.response.request_focus();
        h.frame(vec![]);
        let event = |key| Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        let (r, _) = h.frame(vec![event(egui::Key::Enter)]);
        assert!(h.state.open && r.open_changed && !r.response.changed());
        let (r, _) = h.frame(vec![event(egui::Key::Escape)]);
        assert!(!h.state.open && r.open_changed && !r.response.changed());
    }
}
