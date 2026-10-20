#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_arch = "wasm32"))]
use eframe::egui;

use egui_component::consts::FONT_MANAGER;
use egui_component::theme::UiTheme;

#[cfg(not(target_arch = "wasm32"))]
mod logging;

mod app;
mod docs;
mod pages;
mod routes;

use app::GalleryApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let _logger_guard = logging::init_logging();

    FONT_MANAGER
        .load_from_bytes(
            "gallery-cjk",
            include_bytes!("../assets/NotoSansCJK-Regular.ttc"),
            0,
        )
        .unwrap();
    FONT_MANAGER.set_fallback_chain(&["gallery-cjk"]);

    // Load theme from theme.json file
    let ui_theme = UiTheme::from_json_bytes(include_bytes!("../theme.json"))
        .expect("Failed to load theme.json");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1360.0, 880.0])
            .with_title("egui_component")
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!(
                    "../../assets/icons/egui_component-icon-256-light.png"
                ))
                .expect("valid gallery logo"),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "egui_component Gallery",
        options,
        Box::new(move |cc| {
            FONT_MANAGER.set_egui_fonts(&cc.egui_ctx);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            tracing::info!("eframe app init");

            Ok(Box::new(GalleryApp::new(ui_theme, &cc.egui_ctx)?))
        }),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast as _;
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    // Web Panic Hook
    console_error_panic_hook::set_once();

    FONT_MANAGER
        .load_from_bytes(
            "gallery-cjk",
            include_bytes!("../assets/NotoSansCJK-Regular.ttc"),
            0,
        )
        .unwrap();
    FONT_MANAGER.set_fallback_chain(&["gallery-cjk"]);

    let web_options = eframe::WebOptions::default();
    let canvas = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("the_canvas_id"))
        .expect("Gallery canvas is missing")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .expect("Gallery element must be a canvas");

    wasm_bindgen_futures::spawn_local(async {
        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    FONT_MANAGER.set_egui_fonts(&cc.egui_ctx);
                    egui_extras::install_image_loaders(&cc.egui_ctx);
                    let ui_theme = UiTheme::from_json_bytes(include_bytes!("../theme.json"))
                        .expect("Failed to load theme.json");

                    Ok(Box::new(GalleryApp::new(ui_theme, &cc.egui_ctx)?))
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
                    loading_text.set_text_content(Some(&format!(
                        "The app has crashed. See the developer console for details. {e:?}"
                    )));
                }
            }
        }
    });
}
