use eframe::egui;
use egui_component::{
    components::{text, text_edit},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    let id = ui.make_persistent_id("text-edit-value");
    let mut value = ui.data_mut(|data| data.get_temp::<String>(id).unwrap_or_default());
    text::render(ui, "Project name", text::Style::new(theme));
    text_edit::render(
        ui,
        &mut value,
        text_edit::Style::new_outline_md(theme).placeholder("Your next idea"),
    );
    ui.add_space(12.0);
    text::render(ui, "A quieter style", text::Style::new(theme));
    text_edit::render(
        ui,
        &mut value,
        text_edit::Style::new_subtle_md(theme).placeholder("Same value, different style"),
    );
    ui.data_mut(|data| data.insert_temp(id, value));
}
