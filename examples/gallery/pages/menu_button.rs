use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{button, divider, icon_button, menu_button, text},
    icon::ICONS,
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
            ui.set_width(240.0);
            for action in [
                locale.text("demo.menuButton:new"),
                locale.text("demo.menuButton:duplicate"),
                locale.text("demo.menuButton:archive"),
            ] {
                if button::render(
                    ui,
                    action,
                    button::Style::new_ghost_md(theme).width(Some(ui.available_width())),
                )
                .clicked()
                {
                    egui_component::toast::create(
                        ui.ctx(),
                        action,
                        egui_component::toast::ToastKind::Info,
                    );
                }
            }
        },
        menu_button::Style::new(theme),
    );

    divider::render(
        ui,
        |ui| {
            text::render(
                ui,
                locale.text("demo.menuButton:orUseIconButton"),
                text::Style::new(theme),
            )
        },
        divider::Style::new(),
    );

    menu_button::render(
        ui,
        ui.make_persistent_id("actions-icon-menu"),
        |ui| {
            icon_button::render(
                ui,
                ICONS.arrow_down.clone(),
                icon_button::Style::new_outline_md(theme),
            )
        },
        |ui| {
            ui.set_width(240.0);
            for action in [
                locale.text("demo.menuButton:new"),
                locale.text("demo.menuButton:duplicate"),
                locale.text("demo.menuButton:archive"),
            ] {
                if button::render(
                    ui,
                    action,
                    button::Style::new_ghost_md(theme).width(Some(ui.available_width())),
                )
                .clicked()
                {
                    egui_component::toast::create(
                        ui.ctx(),
                        action,
                        egui_component::toast::ToastKind::Info,
                    );
                }
            }
        },
        menu_button::Style::new(theme),
    );
}
