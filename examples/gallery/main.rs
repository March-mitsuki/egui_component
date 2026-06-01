#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

use egui_component::consts::{DEFAULT_FONT_FAMILY_ALIAS, FONT_MANAGER};
use egui_component::theme::UiTheme;

#[cfg(not(target_arch = "wasm32"))]
mod logging;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let _logger_guard = logging::init_logging();

    FONT_MANAGER
        .load_from_bytes(
            DEFAULT_FONT_FAMILY_ALIAS,
            include_bytes!("../assets/NotoSansCJK-Regular.ttc"),
            0,
        )
        .unwrap();

    // Load theme from theme.json file
    let ui_theme = UiTheme::from_json_bytes(include_bytes!("../theme.json"))
        .expect("Failed to load theme.json");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0])
            .with_title("egui_component Storybook Gallery"),
        ..Default::default()
    };

    eframe::run_native(
        "egui_component Gallery",
        options,
        Box::new(move |_cc| Ok(Box::new(GalleryApp::new(ui_theme)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    // Web Panic Hook
    console_error_panic_hook::set_once();

    FONT_MANAGER
        .load_from_bytes(
            DEFAULT_FONT_FAMILY_ALIAS,
            include_bytes!("../assets/NotoSansCJK-Regular.ttc"),
            0,
        )
        .unwrap();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let start_result = eframe::WebRunner::new()
            .start(
                "the_canvas_id",
                web_options,
                Box::new(|_cc| {
                    let ui_theme = UiTheme::from_json_bytes(include_bytes!("../theme.json"))
                        .expect("Failed to load theme.json");

                    Ok(Box::new(GalleryApp::new(ui_theme)))
                }),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("loading_text"))
        {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        &format!("<p> The app has crashed. See the developer console for details. </p><p>{}</p>", e)
                    );
                }
            }
        }
    });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColorMode {
    Dark,
    Light,
}

struct GalleryApp {
    ui_theme: UiTheme,
    color_mode: ColorMode,
}

impl GalleryApp {
    fn new(ui_theme: UiTheme) -> Self {
        Self {
            ui_theme,
            color_mode: ColorMode::Dark,
        }
    }
}

impl eframe::App for GalleryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if ui.ctx().input(|i| i.viewport().close_requested()) {
            // prevent macOS touch bar
            std::process::exit(0);
        }

        // Set visual styles using theme state
        let _theme = match self.color_mode {
            ColorMode::Dark => {
                ui.ctx().set_visuals(egui::Visuals::dark());
                &self.ui_theme.dark
            }
            ColorMode::Light => {
                ui.ctx().set_visuals(egui::Visuals::light());
                &self.ui_theme.light
            }
        };

        egui::Panel::left("nav_panel").show_inside(ui, |ui| {
            ui.heading("Components");
            ui.separator();
            if ui.button("Button").clicked() {
                // handle navigation (to be implemented)
            }
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("egui_component Gallery SPA");
            ui.label("This app runs both natively and on the web!");

            ui.add_space(20.0);

            if ui.button("Toggle Theme").clicked() {
                self.color_mode = match self.color_mode {
                    ColorMode::Dark => ColorMode::Light,
                    ColorMode::Light => ColorMode::Dark,
                };
            }
        });
    }
}
