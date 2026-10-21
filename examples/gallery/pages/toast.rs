use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{button, text},
    theme::Theme,
    toast,
};

pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    text::render(
        ui,
        locale.text("component.toast:name"),
        text::Style::new_heading(theme),
    );
    text::render(
        ui,
        locale.text("demo.toast:description"),
        text::Style::new(theme),
    );
    ui.horizontal(|ui| {
        for (label, kind) in [
            (locale.text("common:info"), toast::ToastKind::Info),
            (locale.text("common:success"), toast::ToastKind::Success),
            (locale.text("common:warning"), toast::ToastKind::Warning),
            (locale.text("common:error"), toast::ToastKind::Error),
        ] {
            if button::render(ui, label, button::Style::new_solid_md(theme)).clicked() {
                toast::create(
                    locale.format("demo.toast:message", &[("label", label)]),
                    kind,
                );
            }
        }
    });
    if button::render(
        ui,
        locale.text("demo.toast:custom"),
        button::Style::new_solid_md(theme),
    )
    .clicked()
    {
        let label = locale.text("demo.toast:custom").to_owned();
        toast::custom(
            toast::ToastKind::Info,
            Some(std::time::Duration::from_secs(6)),
            move |ui, theme| {
                ui.label(&label);
                egui_component::components::progress::render(
                    ui,
                    None,
                    egui_component::components::progress::Style::new(theme),
                );
            },
        );
    }
}
