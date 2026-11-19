use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{
        calendar::{
            self, CaptionLayout, CaptionOrder, Date, Month, RangeSelection, Selection, YearMonth,
        },
        date_picker::{self, Options, State, Style},
        text,
    },
    theme::Theme,
};

pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    let today = Date::from_calendar_date(2026, Month::November, 17).unwrap();
    let labels = |label| super::calendar::localized_label(locale, label);
    let format_date = |d: Date| match locale {
        Locale::En => d.to_string(),
        _ => format!("{}年{}月{}日", d.year(), d.month() as u8, d.day()),
    };
    let format_selection = |selection: &Selection| match selection {
        Selection::Single(Some(d)) => format_date(*d),
        Selection::Range(RangeSelection::Start(d)) => format!(
            "{} – {}",
            format_date(*d),
            locale.text("datePicker:pending")
        ),
        Selection::Range(RangeSelection::Complete { start, end }) => {
            format!("{} – {}", format_date(*start), format_date(*end))
        }
        _ => String::new(),
    };
    for (key, range, months) in [
        ("datePicker:single", false, 1),
        ("datePicker:rangeSingle", true, 1),
        ("datePicker:rangeDouble", true, 2),
    ] {
        text::render(ui, locale.text(key), text::Style::new(theme));
        let id = ui.make_persistent_id(key);
        let (mut selection, mut state) = ui
            .data_mut(|d| d.get_temp::<(Selection, State)>(id))
            .unwrap_or_else(|| {
                (
                    if range {
                        Selection::Range(RangeSelection::Empty)
                    } else {
                        Selection::Single(None)
                    },
                    State::new(YearMonth::from_date(today)),
                )
            });
        let mut style = Style::new(theme);
        style.trigger.width = Some(if range { 360.0 } else { 280.0 });
        if locale != Locale::En {
            style.calendar.year_select_width = 112.0;
            style.calendar.month_select_width = 72.0;
        }
        date_picker::render(
            ui,
            id,
            &mut selection,
            &mut state,
            style,
            Options {
                placeholder: locale.text(if range {
                    "datePicker:rangePlaceholder"
                } else {
                    "datePicker:placeholder"
                }),
                format_selection: Some(&format_selection),
                calendar: calendar::Options {
                    number_of_months: months,
                    caption_layout: CaptionLayout::Dropdowns,
                    caption_order: if locale == Locale::En {
                        CaptionOrder::MonthYear
                    } else {
                        CaptionOrder::YearMonth
                    },
                    year_range: Some(1900..=2100),
                    fixed_weeks: true,
                    today: Some(today),
                    labels: Some(&labels),
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        ui.data_mut(|d| d.insert_temp(id, (selection, state)));
        ui.add_space(16.0);
    }
}
