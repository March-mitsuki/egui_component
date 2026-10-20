use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{button, card, text},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    card::render(
        ui,
        |ui| {
            text::render(
                ui,
                locale.text("demo.card:title"),
                text::Style::new_heading(theme),
            );
            text::render(ui, locale.text("demo.card:body"), text::Style::new(theme));
            ui.add_space(12.0);
            if button::render(
                ui,
                locale.text("common:createProject"),
                button::Style::new_solid_md(theme),
            )
            .clicked()
            {
                egui_component::toast::create(
                    locale.text("demo.card:created"),
                    egui_component::toast::ToastKind::Success,
                );
            }
        },
        card::Style::new_md(theme).shadow(egui::Shadow::NONE),
    );
}
