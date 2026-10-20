use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{button, menu_button},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    menu_button::render(
        ui,
        ui.make_persistent_id("actions-menu"),
        |ui| {
            button::render(
                ui,
                locale.text("demo.menuButton:actions"),
                button::Style::new_outline_md(theme),
            )
        },
        |ui| {
            for action in [
                locale.text("demo.menuButton:new"),
                locale.text("demo.menuButton:duplicate"),
                locale.text("demo.menuButton:archive"),
            ] {
                if button::render(ui, action, button::Style::new_ghost_md(theme)).clicked() {
                    egui_component::toast::create(action, egui_component::toast::ToastKind::Info);
                }
            }
        },
        menu_button::Style::new(theme),
    );
}
