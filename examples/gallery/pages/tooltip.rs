use eframe::egui;
use egui_component::{
    components::{button, text, tooltip},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    let response = button::render(
        ui,
        "Hover for a little context",
        button::Style::new_outline_md(theme),
    );
    tooltip::render(
        &response,
        |ui| {
            text::render(
                ui,
                "Helpful details, right where you need them.",
                text::Style::new(theme),
            );
        },
        &tooltip::Style::new(theme),
    );
}
