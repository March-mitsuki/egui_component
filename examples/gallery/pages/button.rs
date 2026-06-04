use eframe::egui;
use egui_component::{components::button, theme::Theme};

pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    button::render(ui, "Solid", button::Style::new_solid_md(theme));
    button::render(ui, "Surface", button::Style::new_surface_md(theme));
    button::render(ui, "Outline", button::Style::new_outline_md(theme));
    button::render(ui, "Ghost", button::Style::new_ghost_md(theme));
}
