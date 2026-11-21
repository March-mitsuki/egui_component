use eframe::egui;
use egui_component::{components::avatar, theme::Theme};
pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    ui.horizontal(|ui| {
        avatar::render(
            ui,
            "https://avatars.githubusercontent.com/u/67355676?v=4",
            "March-mitsuki",
            avatar::Style::new_sm(theme),
            Default::default(),
        );
        avatar::render(
            ui,
            "https://avatars.githubusercontent.com/u/67355676?v=4",
            "March-mitsuki",
            avatar::Style::new_md(theme),
            Default::default(),
        );
        avatar::render(
            ui,
            "https://avatars.githubusercontent.com/u/67355676?v=4",
            "March-mitsuki",
            avatar::Style::new_lg(theme),
            Default::default(),
        );
    });
}
