use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{separator, text},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    text::render(
        ui,
        locale.text("demo.separator:first"),
        text::Style::new(theme),
    );
    ui.add_space(12.0);
    separator::render(ui, 1.0);
    ui.add_space(12.0);
    text::render(
        ui,
        locale.text("demo.separator:second"),
        text::Style::new(theme),
    );
}
