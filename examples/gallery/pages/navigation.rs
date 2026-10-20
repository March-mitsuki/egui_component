use eframe::egui;
use egui_component::{
    components::{navigation, text},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    let id = ui.make_persistent_id("nav-example");
    let mut selected = ui.data_mut(|data| {
        data.get_temp::<String>(id)
            .unwrap_or_else(|| "Overview".into())
    });
    for label in ["Overview", "Activity", "Settings"] {
        if navigation::render(ui, label, selected == label, navigation::Style::new(theme)).clicked()
        {
            selected = label.into();
        }
    }
    ui.add_space(12.0);
    text::render(
        ui,
        &format!("Current view: {selected}"),
        text::Style::new(theme),
    );
    ui.data_mut(|data| data.insert_temp(id, selected));
}
