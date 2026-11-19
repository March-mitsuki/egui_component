use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{text, text_edit},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    let id = ui.make_persistent_id("text-edit-value");
    let mut value = ui.data_mut(|data| data.get_temp::<String>(id).unwrap_or_default());
    text::render(
        ui,
        locale.text("common:projectName"),
        text::Style::new(theme),
    );
    text_edit::render(
        ui,
        &mut value,
        text_edit::Style::new_outline_md(theme)
            .placeholder(locale.text("demo.textEdit:placeholder")),
    );
    ui.add_space(12.0);
    text::render(
        ui,
        locale.text("demo.textEdit:quiet"),
        text::Style::new(theme),
    );
    text_edit::render(
        ui,
        &mut value,
        text_edit::Style::new_subtle_md(theme).placeholder(locale.text("demo.textEdit:sameValue")),
    );
    ui.data_mut(|data| data.insert_temp(id, value));
    ui.add_space(12.0);
    let id = ui.make_persistent_id("password-demo");
    let mut value = ui.data_mut(|data| data.get_temp::<String>(id).unwrap_or_default());
    text::render(
        ui,
        locale.text("demo.textEdit:password"),
        text::Style::new(theme),
    );
    text_edit::render(
        ui,
        &mut value,
        text_edit::Style::new_outline_md(theme)
            .input_type(text_edit::InputType::Password)
            .placeholder(locale.text("demo.textEdit:passwordPlaceholder")),
    );
    ui.data_mut(|data| data.insert_temp(id, value));
    for (label, sample, style) in [
        (
            "demo.textEdit:readOnly",
            "demo.textEdit:readOnlyValue",
            text_edit::Style::new_outline_md(theme).read_only(true),
        ),
        (
            "common:disabled",
            "demo.textEdit:disabledValue",
            text_edit::Style::new_outline_md(theme).disabled(true),
        ),
    ] {
        ui.add_space(12.0);
        text::render(ui, locale.text(label), text::Style::new(theme));
        let mut value = locale.text(sample).to_string();
        text_edit::render(ui, &mut value, style);
    }
    ui.add_space(16.0);
    text::render(
        ui,
        locale.text("textEdit:multiline"),
        text::Style::new_heading(theme).size(16.0),
    );
    let id = ui.make_persistent_id("multiline-demo");
    let mut value = ui.data_mut(|data| data.get_temp::<String>(id).unwrap_or_default());
    text::render(
        ui,
        locale.text("demo.textEdit:multilineHint"),
        text::Style::new(theme)
            .size(13.0)
            .color(theme.colors.fg.subtle),
    );
    ui.add_space(4.0);
    text_edit::render_multiline(
        ui,
        &mut value,
        3,
        text_edit::Style::new_outline_md(theme)
            .placeholder(locale.text("demo.textEdit:placeholder")),
    );
    ui.data_mut(|data| data.insert_temp(id, value));
}
