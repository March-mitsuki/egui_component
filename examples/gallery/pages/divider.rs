use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{divider, text},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    text::render(
        ui,
        locale.text("demo.divider:before"),
        text::Style::new(theme),
        Default::default(),
    );
    ui.add_space(12.0);
    divider::render(
        ui,
        |ui| {
            text::render(
                ui,
                locale.text("demo.divider:continue"),
                text::Style::new(theme).size(13.0),
                Default::default(),
            )
        },
        divider::Style::new(),
    );
    ui.add_space(12.0);
    text::render(
        ui,
        locale.text("demo.divider:after"),
        text::Style::new(theme),
        Default::default(),
    );
}
