#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

use egui_component::consts::{DEFAULT_FONT_FAMILY_ALIAS, FONT_MANAGER};
use egui_component::theme::UiTheme;

#[cfg(not(target_arch = "wasm32"))]
mod logging;

mod app;
mod pages;

use app::GalleryApp;

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
        Box::new(move |cc| {
            FONT_MANAGER.set_egui_fonts(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            tracing::info!("eframe app init");

            Ok(Box::new(GalleryApp::new(ui_theme)))
        }),
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
