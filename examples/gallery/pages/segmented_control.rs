use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{segmented_control, text},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    text::render(
        ui,
        locale.text("segmentedControl:help"),
        text::Style::new(theme),
    );
    let id = ui.make_persistent_id("segments-demo");
    let mut selected = ui.data_mut(|data| data.get_temp::<usize>(id).unwrap_or(0));
    let choices = [
        (0, "Light"),
        (1, "Dark"),
        (2, locale.text("generator:compare")),
    ];
    segmented_control::render(
        ui,
        &mut selected,
        &choices,
        segmented_control::Style::new(theme),
    );
    ui.data_mut(|data| data.insert_temp(id, selected));
    segmented_control::render(
        ui,
        &mut selected,
        &choices,
        segmented_control::Style::new(theme).disabled(true),
    );
}
