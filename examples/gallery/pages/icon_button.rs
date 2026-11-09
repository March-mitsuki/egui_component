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
                    ui.ctx(),
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

    ui.horizontal(|ui| {
        let style_origin = (**ui.style()).clone();
        let mut style = style_origin.clone();
        style.spacing.item_spacing.x = 2.0;
        ui.set_style(style);

        icon_button::render(
            ui,
            ICONS.warning.clone(),
            icon_button::Style::new_outline_md(theme).corner_radius(egui::CornerRadius {
                ne: 0,
                nw: theme.corner_radius.md as u8,
                se: 0,
                sw: theme.corner_radius.md as u8,
            }),
        );
        icon_button::render(
            ui,
            ICONS.arrow_down.clone(),
            icon_button::Style::new_outline_md(theme)
                .corner_radius(egui::CornerRadius {
                    ne: theme.corner_radius.md as u8,
                    nw: 0,
                    se: theme.corner_radius.md as u8,
                    sw: 0,
                })
                .width(16.0),
        );

        ui.set_style(style_origin);
    });
}
