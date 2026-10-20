use eframe::egui;
use egui_component::{
    components::{button, text},
    theme::Theme,
};

pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    text::render(ui, "Variants", text::Style::new_heading(theme).size(14.0));
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        for (label, style) in [
            ("Solid", button::Style::new_solid_md(theme)),
            ("Surface", button::Style::new_surface_md(theme)),
            ("Outline", button::Style::new_outline_md(theme)),
            ("Ghost", button::Style::new_ghost_md(theme)),
        ] {
            if button::render(ui, label, style).clicked() {
                egui_component::toast::create(
                    format!("{label} clicked"),
                    egui_component::toast::ToastKind::Info,
                );
            }
        }
    });
    ui.add_space(20.0);
    text::render(ui, "States", text::Style::new_heading(theme).size(14.0));
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        button::render(
            ui,
            "Primary",
            button::Style::new_solid_md_color(theme, theme.colors.primary),
        );
        button::render(
            ui,
            "Disabled",
            button::Style::new_outline_md(theme).state(button::ButtonState::Disable),
        );
        button::render(
            ui,
            "Loading",
            button::Style::new_solid_md(theme).state(button::ButtonState::Loading),
        );
    });
}
