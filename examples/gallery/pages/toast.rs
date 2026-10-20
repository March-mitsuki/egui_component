use eframe::egui;
use egui_component::{
    components::{button, text},
    theme::Theme,
    toast,
};

pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    text::render(ui, "Toast", text::Style::new_heading(theme));
    text::render(
        ui,
        "Click to show a notification. Notifications disappear after three seconds.",
        text::Style::new(theme),
    );
    ui.horizontal(|ui| {
        for (label, kind) in [
            ("Info", toast::ToastKind::Info),
            ("Success", toast::ToastKind::Success),
            ("Warning", toast::ToastKind::Warning),
            ("Error", toast::ToastKind::Error),
        ] {
            if button::render(ui, label, button::Style::new_solid_md(theme)).clicked() {
                toast::create(format!("{label}: notification example"), kind);
            }
        }
    });
}
