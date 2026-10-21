use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{field, text_edit},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    let id = ui.make_persistent_id("field-value");
    let mut value = ui.data_mut(|data| data.get_temp::<String>(id).unwrap_or_default());
    field::render(
        ui,
        locale.text("demo.field:name"),
        |ui| {
            text_edit::render(
                ui,
                &mut value,
                text_edit::Style::new_outline_md(theme)
                    .placeholder(locale.text("demo.field:placeholder")),
            );
        },
        field::Style::new(theme).direction(field::FieldDirection::Vertical),
    );
    ui.data_mut(|data| data.insert_temp(id, value));
}
