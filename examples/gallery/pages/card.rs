use eframe::egui;
use egui_component::{
    components::{button, card, text},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    card::render(
        ui,
        |ui| {
            text::render(
                ui,
                "A place for your next idea",
                text::Style::new_heading(theme),
            );
            text::render(
                ui,
                "Compose content and controls on a shared surface.",
                text::Style::new(theme),
            );
            ui.add_space(12.0);
            if button::render(ui, "Create project", button::Style::new_solid_md(theme)).clicked() {
                egui_component::toast::create(
                    "Project created",
                    egui_component::toast::ToastKind::Success,
                );
            }
        },
        card::Style::new_md(theme).shadow(egui::Shadow::NONE),
    );
}
