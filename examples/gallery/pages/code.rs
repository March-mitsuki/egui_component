use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{code, scroll_area, text},
    theme::Theme,
};

pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    text::render(
        ui,
        locale.text("demo.code:heading"),
        text::Style::new_heading(theme),
    );
    ui.add_space(12.0);

    scroll_area::render(
        ui,
        "code_scroll",
        scroll_area::ScrollAxes::Both,
        |ui| {
            text::render(ui, "Rust", text::Style::new(theme));
            code::render(
                ui,
                r#"// Display a highlighted code snippet
fn main() {
    let message = "Hello, gallery!";
    println!("{message}");
}"#,
                "rs",
                code::Style::new(theme),
            );
            ui.add_space(20.0);
            text::render(ui, "JSON", text::Style::new(theme));
            code::render(
                ui,
                r#"{
  "component": "Code",
  "editable": false,
  "languages": ["rust", "json", "python"]
}"#,
                "json",
                code::Style::new(theme),
            );
        },
        scroll_area::Style::new_no_frame(),
    );
}
