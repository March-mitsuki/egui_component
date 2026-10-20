use eframe::egui;
use egui_component::{
    components::{divider, text},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    text::render(ui, "Content before the divider", text::Style::new(theme));
    ui.add_space(12.0);
    divider::render(
        ui,
        |ui| text::render(ui, "or continue", text::Style::new(theme).size(13.0)),
        divider::Style::new(),
    );
    ui.add_space(12.0);
    text::render(ui, "Content after the divider", text::Style::new(theme));
}
