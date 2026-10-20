use eframe::egui;
use egui_component::{
    components::{separator, text},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    text::render(ui, "A section of content", text::Style::new(theme));
    ui.add_space(12.0);
    separator::render(ui, 1.0);
    ui.add_space(12.0);
    text::render(
        ui,
        "A new section, with a quiet boundary",
        text::Style::new(theme),
    );
}
