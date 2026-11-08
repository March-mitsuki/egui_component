use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::icon_menu_button::{self, IconWithTooltip},
    icon::ICONS,
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    let ctx = ui.ctx().clone();
    icon_menu_button::render(
        ui,
        ui.make_persistent_id("icon-actions"),
        vec![
            IconWithTooltip {
                icon: ICONS.success.clone(),
                tooltip: locale.text("common:success").into(),
            },
            IconWithTooltip {
                icon: ICONS.warning.clone(),
                tooltip: locale.text("common:warning").into(),
            },
            IconWithTooltip {
                icon: ICONS.error.clone(),
                tooltip: locale.text("common:error").into(),
            },
        ],
        |index| {
            egui_component::toast::create(
                &ctx,
                locale.format(
                    "demo.iconMenuButton:selected",
                    &[("index", &(index + 1).to_string())],
                ),
                egui_component::toast::ToastKind::Info,
            )
        },
        theme,
        icon_menu_button::Style::new(theme),
    );
}
