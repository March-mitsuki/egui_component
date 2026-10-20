use eframe::egui;
use egui_component::components::{button, text, window};
use egui_component::theme::UiTheme;

fn main() -> eframe::Result {
    // Load theme from theme.json file
    let ui_theme =
        UiTheme::from_json_bytes(include_bytes!("theme.json")).expect("Failed to load theme.json");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Custom Window Component Demo",
        options,
        Box::new(move |cc| {
            // Set up custom fonts from our font manager
            egui_component::consts::FONT_MANAGER.set_egui_fonts(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(MyApp {
                ui_theme,
                is_open: true,
            }))
        }),
    )
}

struct MyApp {
    ui_theme: UiTheme,
    is_open: bool,
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if ui.ctx().input(|i| i.viewport().close_requested()) {
            // prevent macOS touch bar crash report
            std::process::exit(0);
        }

        let theme = &self.ui_theme.light;

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("Custom Window Component Example");
            ui.add_space(8.0);

            egui::Window::new("Native Window").show(ui.ctx(), |ui| ui.button("hello"));

            // Re-open window button if closed
            if !self.is_open {
                // Using our own button component!
                if button::render(
                    ui,
                    "Re-open Window",
                    button::Style::new_solid_md(theme),
                ).clicked() {
                    self.is_open = true;
                }
            } else {
                ui.label("Use the custom header bar (the dark top bar) to drag, collapse, or close the window.");
            }

            let window_id = egui::Id::new("custom_drag_window");

            // Render the window using the new reusable component
            window::render(
                ui.ctx(),
                window_id,
                Some(&mut self.is_open),
                |ui, _state| {
                    // Custom Title closure:
                    // Title Text styled using our own text component's design tokens
                    // We extract font size, family, weight, and color from our theme via text::Style.
                    let text_style = text::Style::new_heading(theme);
                    let font_family = egui_component::consts::FONT_MANAGER.get_egui_font_family(
                        ui.ctx(),
                        &text_style.font_family,
                        text_style.font_weight,
                    );

                    let galley = ui.painter().layout_no_wrap(
                        "Drag Here to Move Window".to_owned(),
                        egui::FontId::new(text_style.size, font_family),
                        text_style.color,
                    );

                    // Allocate space with Sense::empty() so dragging falls through to the background drag sensor
                    let (rect, _) = ui.allocate_at_least(galley.size(), egui::Sense::empty());
                    ui.painter().galley(rect.min, galley, text_style.color);
                },
                |ui, _state| {
                    // Custom Content closure:
                    ui.vertical(|ui| {
                        ui.add_space(8.0);
                        ui.label("This is the window body content.");
                        ui.label("Try dragging this text area: it will not drag the window.");
                        ui.add_space(12.0);

                        // Using our own button component!
                        button::render(
                            ui,
                            "Click Me",
                            button::Style::new_solid_md(theme),
                        );
                    });
                },
                window::Style::new(theme)
                    .init_pos(egui::pos2(150.0, 150.0)),
            );
        });
    }
}
