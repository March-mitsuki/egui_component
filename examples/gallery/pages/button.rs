use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{button, text},
    theme::Theme,
};

pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    text::render(
        ui,
        locale.text("demo.button:variants"),
        text::Style::new_heading(theme).size(14.0),
    );
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        for (label, style) in [
            (
                locale.text("demo.button:solid"),
                button::Style::new_solid_md(theme),
            ),
            (
                locale.text("demo.button:surface"),
                button::Style::new_surface_md(theme),
            ),
            (
                locale.text("demo.button:outline"),
                button::Style::new_outline_md(theme),
            ),
            (
                locale.text("demo.button:ghost"),
                button::Style::new_ghost_md(theme),
            ),
        ] {
            if button::render(ui, label, style).clicked() {
                egui_component::toast::create(
                    locale.format("common:clicked", &[("label", label)]),
                    egui_component::toast::ToastKind::Info,
                );
            }
        }
    });
    ui.add_space(20.0);
    text::render(
        ui,
        locale.text("demo.button:states"),
        text::Style::new_heading(theme).size(14.0),
    );
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        button::render(
            ui,
            locale.text("common:primary"),
            button::Style::new_solid_md_color(theme, theme.colors.primary),
        );
        button::render(
            ui,
            locale.text("common:disabled"),
            button::Style::new_outline_md(theme).state(button::ButtonState::Disable),
        );
        button::render(
            ui,
            locale.text("common:loading"),
            button::Style::new_solid_md(theme).state(button::ButtonState::Loading),
        );
    });
}
