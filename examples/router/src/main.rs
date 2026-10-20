use eframe::egui;
use egui_component::{
    components::{button, separator, text},
    theme::UiTheme,
};
use egui_component_router::{Route, Router};

#[derive(Clone, Debug, PartialEq)]
enum Page {
    Home,
    Docs,
    Component,
    NotFound(String),
}
impl Route for Page {
    fn from_path(path: &str) -> Self {
        match path {
            "/" => Self::Home,
            "/docs" => Self::Docs,
            "/component" => Self::Component,
            other => Self::NotFound(other.into()),
        }
    }
    fn to_path(&self) -> String {
        match self {
            Self::Home => "/",
            Self::Docs => "/docs",
            Self::Component => "/component",
            Self::NotFound(path) => path,
        }
        .into()
    }
}

struct App {
    router: Router<Page>,
    theme: UiTheme,
    error: Option<String>,
}
impl App {
    fn new(
        cc: &eframe::CreationContext<'_>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let ctx = cc.egui_ctx.clone();
        Ok(Self {
            router: Router::new(Page::Home, move || ctx.request_repaint())?,
            theme: UiTheme::from_json_bytes(include_bytes!("../../theme.json"))?,
            error: None,
        })
    }
}
impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let theme = &self.theme.dark;
        ui.ctx().set_visuals(egui::Visuals::dark());
        egui_component::consts::FONT_MANAGER.apply(ui.ctx());
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.set_min_height(56.0);
                text::render(
                    ui,
                    "egui_component / router",
                    text::Style::new_heading(theme),
                );
                ui.add_space(24.0);
                for (page, label) in [
                    (Page::Home, "Home"),
                    (Page::Docs, "Docs"),
                    (Page::Component, "Component"),
                ] {
                    let style = if self.router.current() == page {
                        button::Style::new_solid_md(theme)
                    } else {
                        button::Style::new_ghost_md(theme)
                    };
                    if button::render(ui, label, style).clicked() {
                        self.error = self.router.navigate(page).err().map(|e| e.to_string());
                    }
                }
            });
            separator::render(ui, 1.0);
            ui.add_space(32.0);
            let (title, body) = match self.router.current() {
                Page::Home => (
                    "Home",
                    "Welcome! This application runs on native desktop and Web.",
                ),
                Page::Docs => (
                    "Docs",
                    "Define typed routes, then use the header to navigate between pages.",
                ),
                Page::Component => (
                    "Component",
                    "This header and page use egui_component buttons, text and separators.",
                ),
                Page::NotFound(_) => (
                    "404 — Page not found",
                    "Choose a page from the header to continue.",
                ),
            };
            text::render(ui, title, text::Style::new_heading(theme).size(32.0));
            ui.add_space(12.0);
            text::render(ui, body, text::Style::new(theme));
            ui.add_space(24.0);
            text::render(
                ui,
                &format!("Route: {}", self.router.current().to_path()),
                text::Style::new(theme),
            );
            ui.horizontal(|ui| {
                if button::render(ui, "Back", button::Style::new_outline_md(theme)).clicked() {
                    self.error = self.router.back().err().map(|e| e.to_string());
                }
                if button::render(ui, "Forward", button::Style::new_outline_md(theme)).clicked() {
                    self.error = self.router.forward().err().map(|e| e.to_string());
                }
            });
            if let Some(error) = &self.error {
                text::render(ui, error, text::Style::new(theme));
            }
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    eframe::run_native(
        "Router example",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 600.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)?))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;
    console_error_panic_hook::set_once();
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id("router_canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        let result = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(App::new(cc)?))),
            )
            .await;
        let status = document.get_element_by_id("status").unwrap();
        match result {
            Ok(()) => status.remove(),
            Err(error) => status.set_text_content(Some(&format!("Failed to start: {error:?}"))),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_paths_use_not_found_fallback() {
        assert_eq!(
            Page::from_path("/missing/nested"),
            Page::NotFound("/missing/nested".into())
        );
    }
    #[test]
    fn routes_round_trip_and_unknown_paths_survive() {
        for path in ["/", "/docs", "/component", "/missing"] {
            assert_eq!(Page::from_path(path).to_path(), path);
        }
    }
}
