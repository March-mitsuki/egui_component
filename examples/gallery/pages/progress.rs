use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{progress, text},
    theme::Theme,
};

pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    text::render(
        ui,
        locale.text("component.progress:name"),
        text::Style::new_heading(theme),
    );
    for value in [0.0, 0.35, 0.75, 1.0] {
        ui.label(format!("{:.0}%", value * 100.0));
        progress::render(ui, Some(value), progress::Style::new(theme));
        ui.add_space(12.0);
    }
    ui.label(locale.text("demo.progress:indeterminate"));
    progress::render(ui, None, progress::Style::new(theme));
}
