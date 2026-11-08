use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{icon_button, text, tooltip},
    icon::ICONS,
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    ui.horizontal_wrapped(|ui| {
        for (label, image) in [
            (locale.text("common:success"), ICONS.success.clone()),
            (locale.text("common:warning"), ICONS.warning.clone()),
            (locale.text("common:error"), ICONS.error.clone()),
        ] {
            let response =
                icon_button::render(ui, image, icon_button::Style::new_outline_md(theme));
            if response.clicked() {
                egui_component::toast::create(
                    locale.format("common:clicked", &[("label", label)]),
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
