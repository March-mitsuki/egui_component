use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::icon_menu_button::{self, IconWithTooltip},
    icon::ICONS,
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    icon_menu_button::render(
        ui,
        ui.make_persistent_id("icon-actions"),
        vec![
            IconWithTooltip {
                icon: ICONS.play.clone(),
                tooltip: locale.text("common:play").into(),
            },
            IconWithTooltip {
                icon: ICONS.pause.clone(),
                tooltip: locale.text("common:pause").into(),
            },
        ],
        |index| {
            egui_component::toast::create(
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
