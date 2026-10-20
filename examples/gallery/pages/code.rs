use eframe::egui;
use egui_component::{
    components::{code, scroll_area, text},
    theme::Theme,
};

pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    text::render(
        ui,
        "Read-only syntax highlighting",
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
