use eframe::egui;
use egui_component::components::text;

pub fn show(ui: &mut egui::Ui, theme: &egui_component::theme::Theme) {
    text::render(ui, "Icon Button Component", text::Style::new(theme));
}
