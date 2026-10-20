use eframe::egui;
use egui_component::{
    components::{button, dialog, text},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    let id = ui.make_persistent_id("dialog-open");
    let mut open = ui.data_mut(|data| data.get_temp::<bool>(id).unwrap_or(false));
    if button::render(ui, "Open dialog", button::Style::new_outline_md(theme)).clicked() {
        open = true;
    }
    if open {
        dialog::render(
            ui.ctx(),
            |ui| {
                dialog::render_header(ui, theme, "A moment to focus", || open = false);
                text::render(
                    ui,
                    "Dialogs give an important decision its own space.",
                    text::Style::new(theme),
                );
                ui.add_space(16.0);
                if button::render(ui, "Got it", button::Style::new_solid_md(theme)).clicked() {
                    open = false;
                }
            },
            dialog::Style::new(theme)
                .title("Gallery dialog")
                .size((280.0, 0.0)),
        );
    }
    ui.data_mut(|data| data.insert_temp(id, open));
}
