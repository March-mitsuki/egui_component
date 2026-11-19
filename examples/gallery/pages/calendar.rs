use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{
        calendar::{
            self, CaptionLayout, CaptionOrder, Date, Label, Month, Options, RangeSelection,
            Selection, State, Weekday, YearMonth,
        },
        text,
    },
    theme::Theme,
};

pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    // Fixed demo date: applications supply their own clock/timezone policy.
    let today = Date::from_calendar_date(2026, Month::November, 17).unwrap();
    let labels = |label| localized_label(locale, label);
    // Selection mode, visible month count, and caption controls are independent.
    for (key, range, months, caption_layout, disable_weekends) in [
        ("calendar:single", false, 1, CaptionLayout::Dropdowns, false),
        ("calendar:range", true, 2, CaptionLayout::Label, true),
        (
            "calendar:rangeSingle",
            true,
            1,
            CaptionLayout::Dropdowns,
            false,
        ),
        (
            "calendar:rangeDropdowns",
            true,
            2,
            CaptionLayout::Dropdowns,
            false,
        ),
    ] {
        text::render(ui, locale.text(key), text::Style::new(theme));
        let id = ui.make_persistent_id(("calendar-demo", key));
        let (mut selection, mut state) = ui
            .data_mut(|data| data.get_temp::<(Selection, State)>(id))
            .unwrap_or_else(|| {
                (
                    if range {
                        Selection::Range(RangeSelection::Empty)
                    } else {
                        Selection::Single(Some(today))
                    },
                    State::new(YearMonth::from_date(today)),
                )
            });
        let weekends = |d: Date| matches!(d.weekday(), Weekday::Saturday | Weekday::Sunday);
        let mut style = calendar::Style::new(theme);
        // Localized year labels include 年; numeric month labels need less room.
        // Keep widths fixed for each locale instead of measuring the selected text.
        if matches!(locale, Locale::Zh | Locale::Ja) {
            style.year_select_width = 112.0;
            style.month_select_width = 72.0;
        }
        calendar::render(
            ui,
            id,
            &mut selection,
            &mut state,
            style,
            Options {
                number_of_months: months,
                fixed_weeks: true,
                caption_layout,
                caption_order: if locale == Locale::En {
                    CaptionOrder::MonthYear
                } else {
                    CaptionOrder::YearMonth
                },
                year_range: Some(1900..=2100),
                is_date_disabled: if disable_weekends {
                    Some(&weekends)
                } else {
                    None
                },
                today: Some(today),
                labels: Some(&labels),
                ..Default::default()
            },
        );
        ui.data_mut(|data| data.insert_temp(id, (selection, state)));
        ui.add_space(16.0);
    }
}

pub(super) fn localized_label(locale: Locale, label: Label) -> String {
    match locale {
        Locale::En => calendar::default_label(label),
        Locale::Zh | Locale::Ja => match label {
            Label::MonthTitle(m) => format!("{}年{}月", m.year(), m.month() as u8),
            Label::MonthName(m) => format!("{}月", m as u8),
            Label::Year(y) => format!("{y}年"),
            Label::WeekdayShort(w) => {
                let names = if locale == Locale::Zh {
                    ["一", "二", "三", "四", "五", "六", "日"]
                } else {
                    ["月", "火", "水", "木", "金", "土", "日"]
                };
                names[w.number_days_from_monday() as usize].into()
            }
            Label::DateAccessible(d) => format!("{}年{}月{}日", d.year(), d.month() as u8, d.day()),
            Label::PreviousMonth => if locale == Locale::Zh {
                "上个月"
            } else {
                "前の月"
            }
            .into(),
            Label::NextMonth => if locale == Locale::Zh {
                "下个月"
            } else {
                "次の月"
            }
            .into(),
            Label::ChooseMonth => if locale == Locale::Zh {
                "选择月份"
            } else {
                "月を選択"
            }
            .into(),
            Label::ChooseYear => if locale == Locale::Zh {
                "选择年份"
            } else {
                "年を選択"
            }
            .into(),
            other => calendar::default_label(other),
        },
    }
}
