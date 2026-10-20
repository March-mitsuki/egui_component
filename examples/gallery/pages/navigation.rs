use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{navigation, text},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    let id = ui.make_persistent_id("nav-example");
    let mut selected = ui.data_mut(|data| data.get_temp::<usize>(id).unwrap_or(0));
    let labels = [
        locale.text("common:overview"),
        locale.text("demo.navigation:activity"),
        locale.text("demo.navigation:settings"),
    ];
    for (index, label) in labels.into_iter().enumerate() {
        if navigation::render(ui, label, selected == index, navigation::Style::new(theme)).clicked()
        {
            selected = index;
        }
    }
    ui.add_space(12.0);
    text::render(
        ui,
        &locale.format("demo.navigation:current", &[("selected", labels[selected])]),
        text::Style::new(theme),
    );
    ui.data_mut(|data| data.insert_temp(id, selected));
}
