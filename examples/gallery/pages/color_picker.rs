use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{color_picker, text},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    text::render(ui, locale.text("colorPicker:help"), text::Style::new(theme));
    let id = ui.make_persistent_id("color-picker-demo");
    let mut color = ui.data_mut(|data| {
        data.get_temp::<egui::Color32>(id)
            .unwrap_or(theme.colors.primary.l6)
    });
    color_picker::render(ui, id, &mut color, color_picker::Style::new(theme));
    text::render(ui, &color.to_hex(), text::Style::new(theme));
    ui.data_mut(|data| data.insert_temp(id, color));
    let id = ui.make_persistent_id("color-picker-alpha");
    let mut alpha = ui.data_mut(|data| {
        data.get_temp::<egui::Color32>(id)
            .unwrap_or(egui::Color32::from_rgba_unmultiplied(107, 101, 245, 150))
    });
    let error_message = |error: &color_picker::ColorInputError| {
        use color_picker::ColorInputError::*;
        locale
            .text(match error {
                Empty { .. } => "colorPicker:errorEmpty",
                InvalidHexLength { .. } => "colorPicker:errorHexLength",
                InvalidHexDigit => "colorPicker:errorHexDigit",
                InvalidNumber { .. } => "colorPicker:errorNumber",
                OutOfRange {
                    field: color_picker::ColorField::Alpha,
                } => "colorPicker:errorAlphaRange",
                OutOfRange { .. } => "colorPicker:errorRgbRange",
            })
            .to_owned()
    };
    color_picker::render_with_options(
        ui,
        id,
        &mut alpha,
        color_picker::Style::new(theme).alpha(true),
        color_picker::Options {
            error_message: Some(&error_message),
        },
    );
    ui.data_mut(|data| data.insert_temp(id, alpha));
    color_picker::render(
        ui,
        ui.make_persistent_id("color-disabled"),
        &mut color,
        color_picker::Style::new(theme).disabled(true),
    );
}
