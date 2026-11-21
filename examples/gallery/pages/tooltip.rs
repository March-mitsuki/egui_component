use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{button, text, tooltip},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    let response = button::render(
        ui,
        locale.text("demo.tooltip:trigger"),
        button::Style::new_outline_md(theme),
        Default::default(),
    );
    tooltip::render(
        &response,
        |ui| {
            text::render(
                ui,
                locale.text("demo.tooltip:body"),
                text::Style::new(theme),
                Default::default(),
            );
        },
        &tooltip::Style::new(theme),
        Default::default(),
    );
}
