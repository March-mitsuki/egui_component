use eframe::egui;
use egui_component::{
    components::{icon_button, text, tooltip},
    icon::ICONS,
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    ui.horizontal_wrapped(|ui| {
        for (label, image) in [
            ("Play", ICONS.play.clone()),
            ("Pause", ICONS.pause.clone()),
            ("Add", ICONS.add.clone()),
        ] {
            let response =
                icon_button::render(ui, image, icon_button::Style::new_outline_md(theme));
            if response.clicked() {
                egui_component::toast::create(
                    format!("{label} clicked"),
                    egui_component::toast::ToastKind::Info,
                );
            }
            tooltip::render(
                &response,
                |ui| {
                    text::render(ui, label, text::Style::new(theme));
                },
                &tooltip::Style::new(theme),
            );
        }
    });
}
