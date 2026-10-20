use eframe::egui;
use egui_component::{
    components::icon_menu_button::{self, IconWithTooltip},
    icon::ICONS,
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    icon_menu_button::render(
        ui,
        ui.make_persistent_id("icon-actions"),
        vec![
            IconWithTooltip {
                icon: ICONS.play.clone(),
                tooltip: "Play".into(),
            },
            IconWithTooltip {
                icon: ICONS.pause.clone(),
                tooltip: "Pause".into(),
            },
        ],
        |index| {
            egui_component::toast::create(
                format!("Action {} selected", index + 1),
                egui_component::toast::ToastKind::Info,
            )
        },
        theme,
        icon_menu_button::Style::new(theme),
    );
}
