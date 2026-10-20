use eframe::egui;
use egui_component::{
    components::{button, menu_button},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    menu_button::render(
        ui,
        ui.make_persistent_id("actions-menu"),
        |ui| {
            button::render(
                ui,
                "Project actions ↓",
                button::Style::new_outline_md(theme),
            )
        },
        |ui| {
            for action in ["New project", "Duplicate", "Archive"] {
                if button::render(ui, action, button::Style::new_ghost_md(theme)).clicked() {
                    egui_component::toast::create(
                        action,
                        egui_component::toast::ToastKind::Info,
                    );
                }
            }
        },
        menu_button::Style::new(theme),
    );
}
