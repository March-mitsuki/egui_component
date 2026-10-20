use eframe::egui;
use egui_component::{
    components::{scroll_area, text},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    scroll_area::render(
        ui,
        "scroll-example",
        scroll_area::ScrollAxes::Vertical,
        |ui| {
            for index in 1..=24 {
                text::render(
                    ui,
                    &format!("Item {index:02}  /  A little more room to explore"),
                    text::Style::new(theme),
                );
                ui.add_space(8.0);
            }
        },
        scroll_area::Style::new(theme).max_height(200.0),
    );
}
