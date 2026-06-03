#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

use egui_component::components::{button, separator, text};
use egui_component::consts::{DEFAULT_FONT_FAMILY_ALIAS, FONT_MANAGER};
use egui_component::theme::UiTheme;

#[cfg(not(target_arch = "wasm32"))]
mod logging;

mod pages;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColorMode {
    Dark,
    Light,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Page {
    Avatar,
    Button,
    Card,
    Dialog,
    Divider,
    Field,
    Icon,
    IconButton,
    IconMenuButton,
    MenuButton,
    ScrollArea,
    Separator,
    Text,
    TextEdit,
    Toast,
    Tooltip,
}

impl Page {
    fn name(&self) -> &'static str {
        match self {
            Page::Avatar => "Avatar",
            Page::Button => "Button",
            Page::Card => "Card",
            Page::Dialog => "Dialog",
            Page::Divider => "Divider",
            Page::Field => "Field",
            Page::Icon => "Icon",
            Page::IconButton => "Icon Button",
            Page::IconMenuButton => "Icon Menu Button",
            Page::MenuButton => "Menu Button",
            Page::ScrollArea => "Scroll Area",
            Page::Separator => "Separator",
            Page::Text => "Text",
            Page::TextEdit => "Text Edit",
            Page::Toast => "Toast",
            Page::Tooltip => "Tooltip",
        }
    }
}

struct GalleryApp {
    ui_theme: UiTheme,
    color_mode: ColorMode,
    current_page: Page,
}

impl GalleryApp {
    fn new(ui_theme: UiTheme) -> Self {
        Self {
            ui_theme,
            color_mode: ColorMode::Dark,
            current_page: Page::Button,
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
        let theme = match self.color_mode {
            ColorMode::Dark => {
                ui.ctx().set_visuals(egui::Visuals::dark());
                &self.ui_theme.dark
            }
            ColorMode::Light => {
                ui.ctx().set_visuals(egui::Visuals::light());
                &self.ui_theme.light
            }
        };
        let header_height: f32 = 56.0;
        let panel_frame = egui::Frame::NONE
            .fill(theme.colors.bg.base)
            .inner_margin(egui::Margin::same(12));

        egui::Panel::left("nav_panel")
            .frame(panel_frame)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.set_height(header_height);
                    text::render(ui, "Components", text::Style::new_heading(theme));
                });
                separator::render(ui, 1.0);
                ui.add_space(8.0);

                let pages = [
                    Page::Avatar,
                    Page::Button,
                    Page::Card,
                    Page::Dialog,
                    Page::Divider,
                    Page::Field,
                    Page::Icon,
                    Page::IconButton,
                    Page::IconMenuButton,
                    Page::MenuButton,
                    Page::ScrollArea,
                    Page::Separator,
                    Page::Text,
                    Page::TextEdit,
                    Page::Toast,
                    Page::Tooltip,
                ];

                for page in pages {
                    let label = page.name();
                    let is_selected = self.current_page == page;

                    let nav_btn_style = if is_selected {
                        button::Style::new_solid_md(theme).width(Some(ui.available_width()))
                    } else {
                        button::Style::new_ghost_md(theme).width(Some(ui.available_width()))
                    };
                    if button::render(ui, label, nav_btn_style).clicked() {
                        self.current_page = page;
                    }
                }
            });

        egui::CentralPanel::default()
            .frame(panel_frame)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.set_height(header_height);
                    text::render(
                        ui,
                        &format!("组件 Component - {}", self.current_page.name()),
                        text::Style::new_heading(theme),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if button::render(ui, "Toggle Theme", button::Style::new_outline_sm(theme))
                            .clicked()
                        {
                            self.color_mode = match self.color_mode {
                                ColorMode::Dark => ColorMode::Light,
                                ColorMode::Light => ColorMode::Dark,
                            };
                        }
                    });
                });
                separator::render(ui, 1.0);
                ui.add_space(8.0);

                match self.current_page {
                    Page::Avatar => pages::avatar::show(ui, theme),
                    Page::Button => pages::button::show(ui),
                    Page::Card => pages::card::show(ui),
                    Page::Dialog => pages::dialog::show(ui),
                    Page::Divider => pages::divider::show(ui),
                    Page::Field => pages::field::show(ui),
                    Page::Icon => pages::icon::show(ui),
                    Page::IconButton => pages::icon_button::show(ui),
                    Page::IconMenuButton => pages::icon_menu_button::show(ui),
                    Page::MenuButton => pages::menu_button::show(ui),
                    Page::ScrollArea => pages::scroll_area::show(ui),
                    Page::Separator => pages::separator::show(ui),
                    Page::Text => pages::text::show(ui),
                    Page::TextEdit => pages::text_edit::show(ui),
                    Page::Toast => pages::toast::show(ui),
                    Page::Tooltip => pages::tooltip::show(ui),
                }
            });
    }
}
