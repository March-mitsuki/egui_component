use eframe::egui;
use egui_component::{components::avatar, theme::Theme};
pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    ui.horizontal(|ui| {
        avatar::render(ui, "", "Alex", avatar::Style::new_sm(theme));
        avatar::render(ui, "", "Morgan", avatar::Style::new_md(theme));
        avatar::render(ui, "", "Sam", avatar::Style::new_lg(theme));
    });
}
