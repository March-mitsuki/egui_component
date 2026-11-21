use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{accordion, separator, text},
    theme::Theme,
};

pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    let style_origin = (**ui.style()).clone();
    let mut style = style_origin.clone();
    style.spacing.item_spacing.y = 0.0;
    ui.set_style(style);

    let id = ui.make_persistent_id("accordion-basic");
    accordion::render(
        ui,
        id,
        locale.text("accordion:basic"),
        None,
        |ui| {
            ui.label(locale.text("accordion:body"));
        },
        accordion::Style::new(theme),
        accordion::Options::default()
            .default_open(true)
            .description(locale.text("accordion:description")),
    );
    separator::render(ui, 1.0);

    let id = ui.make_persistent_id("accordion-second");
    accordion::render(
        ui,
        id,
        locale.text("accordion:advanced"),
        None,
        |ui| {
            ui.label(locale.text("accordion:independent"));
            let _ = ui.button(locale.text("accordion:action"));
        },
        accordion::Style::new(theme).show_body_guide(true),
        Default::default(),
    );
    separator::render(ui, 1.0);

    let id = ui.make_persistent_id("accordion-disabled");
    accordion::render(
        ui,
        id,
        locale.text("common:disabledHeading"),
        None,
        |ui| {
            ui.label("Disabled");
        },
        accordion::Style::new(theme),
        accordion::Options::default().disabled(true),
    );

    ui.add_space(20.0);
    text::render(
        ui,
        locale.text("accordion:exclusive"),
        text::Style::new(theme),
        Default::default(),
    );
    ui.add_space(10.0);

    let group = ui.make_persistent_id("exclusive-accordion");
    let mut selected = ui.data_mut(|data| data.get_temp::<Option<usize>>(group).unwrap_or(Some(0)));
    for (index, title) in [
        locale.text("accordion:basic"),
        locale.text("accordion:advanced"),
    ]
    .into_iter()
    .enumerate()
    {
        if index > 0 {
            separator::render(ui, 1.0);
        }
        let mut open = selected == Some(index);
        accordion::render(
            ui,
            group.with(index),
            title,
            Some(&mut open),
            |ui| {
                ui.label(locale.text("accordion:body"));
            },
            accordion::Style::new(theme),
            Default::default(),
        );

        if open {
            selected = Some(index);
        } else if selected == Some(index) {
            selected = None;
        }
    }
    ui.data_mut(|data| data.insert_temp(group, selected));

    ui.set_style(style_origin);
}
