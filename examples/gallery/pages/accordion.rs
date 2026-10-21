use crate::i18n::Locale;
use eframe::egui;
use egui_component::{components::accordion, theme::Theme};

pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    let id = ui.make_persistent_id("accordion-basic");
    accordion::render(
        ui,
        id,
        locale.text("accordion:basic"),
        Some(locale.text("accordion:description")),
        None,
        |ui| {
            ui.label(locale.text("accordion:body"));
        },
        accordion::Style::new(theme).default_open(true),
    );
    ui.add_space(12.0);
    let id = ui.make_persistent_id("accordion-second");
    accordion::render(
        ui,
        id,
        locale.text("accordion:advanced"),
        None,
        None,
        |ui| {
            ui.label(locale.text("accordion:independent"));
            let _ = ui.button(locale.text("accordion:action"));
        },
        accordion::Style::new(theme),
    );
    ui.add_space(12.0);
    let id = ui.make_persistent_id("accordion-disabled");
    accordion::render(
        ui,
        id,
        locale.text("common:disabledHeading"),
        None,
        None,
        |ui| {
            ui.label("Disabled");
        },
        accordion::Style::new(theme).disabled(true),
    );
    ui.add_space(20.0);
    egui_component::components::text::render(
        ui,
        locale.text("accordion:exclusive"),
        egui_component::components::text::Style::new(theme),
    );
    let group = ui.make_persistent_id("exclusive-accordion");
    let mut selected = ui.data_mut(|data| data.get_temp::<Option<usize>>(group).unwrap_or(Some(0)));
    for (index, title) in [
        locale.text("accordion:basic"),
        locale.text("accordion:advanced"),
    ]
    .into_iter()
    .enumerate()
    {
        let mut open = selected == Some(index);
        accordion::render(
            ui,
            group.with(index),
            title,
            None,
            Some(&mut open),
            |ui| {
                ui.label(locale.text("accordion:body"));
            },
            accordion::Style::new(theme),
        );
        if open {
            selected = Some(index);
        } else if selected == Some(index) {
            selected = None;
        }
    }
    ui.data_mut(|data| data.insert_temp(group, selected));
}
