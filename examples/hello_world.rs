use eframe::egui;
use egui_component::components::{self, button};
use egui_component::theme::UiTheme;

fn main() -> eframe::Result {
    // Load theme from theme.json file
    let ui_theme =
        UiTheme::from_json_bytes(include_bytes!("theme.json")).expect("Failed to load theme.json");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui_component demo",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(MyApp {
                ui_theme,
                color_mode: ColorMode::Dark,
            }))
        }),
    )
}

enum ColorMode {
    Dark,
    Light,
}

struct MyApp {
    ui_theme: UiTheme,
    color_mode: ColorMode,
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let theme = match self.color_mode {
            ColorMode::Dark => {
                // set egui to dark mode
                ui.ctx().set_visuals(egui::Visuals::dark());

                // set egui_component theme to dark mode here too
                // you can even render components using a different theme
                &self.ui_theme.dark
            }
            ColorMode::Light => {
                ui.ctx().set_visuals(egui::Visuals::light());
                &self.ui_theme.light
            }
        };

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);
                ui.heading("Hello egui_component!");
                ui.add_space(20.0);

                let button_width = 128.0;

                // egui_component button
                if button::render(
                    ui,
                    "Toggle Theme",
                    button::Style::new_solid_md(theme).width(Some(button_width)),
                )
                .clicked()
                {
                    self.color_mode = match self.color_mode {
                        ColorMode::Dark => ColorMode::Light,
                        ColorMode::Light => ColorMode::Dark,
                    };
                }

                ui.add_space(24.0);

                // change egui spacing
                let style_origin = components::utils::change_style(ui, |s| {
                    s.spacing.item_spacing.y = 12.0;
                });

                // Native egui button
                if ui.button("Native egui button").clicked() {
                    println!("Button clicked!");
                }

                // egui_component button
                button::render(
                    ui,
                    "Solid",
                    button::Style::new_solid_md(theme).width(Some(button_width)),
                );
                button::render(
                    ui,
                    "Surface",
                    button::Style::new_surface_md(theme).width(Some(button_width)),
                );
                button::render(
                    ui,
                    "Outline",
                    button::Style::new_outline_md(theme).width(Some(button_width)),
                );
                button::render(
                    ui,
                    "Ghost",
                    button::Style::new_ghost_md(theme).width(Some(button_width)),
                );

                // restore egui spacing
                ui.set_style(style_origin);
            });
        });
    }
}
