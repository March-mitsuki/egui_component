use eframe::egui;
use egui_component::{
    components::{button, separator, text},
    theme::UiTheme,
};
use egui_component_router::{HistoryMode, PathPattern, Route, Router, SearchParams};

#[derive(Clone, Debug, PartialEq)]
enum Page {
    Home,
    Docs,
    Component,
    ComponentDetail { name: String },
    NotFound(String),
}
impl Route for Page {
    fn from_path(path: &str) -> Self {
        match path {
            "/" => Self::Home,
            "/docs" => Self::Docs,
            "/component" => Self::Component,
            other => {
                let pattern = PathPattern::new("/component/:name").expect("valid pattern");
                match pattern.match_path(other) {
                    Some(params) => Self::ComponentDetail {
                        name: params.get("name").unwrap().into(),
                    },
                    None => Self::NotFound(other.into()),
                }
            }
        }
    }
    fn to_path(&self) -> String {
        if let Self::ComponentDetail { name } = self {
            return PathPattern::new("/component/:name")
                .unwrap()
                .build(&[("name", name)])
                .expect("valid component name");
        }
        match self {
            Self::ComponentDetail { .. } => unreachable!(),
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
        #[cfg(not(target_arch = "wasm32"))]
        if ui.ctx().input(|i| i.viewport().close_requested()) {
            // prevent macOS touch bar crash report
            std::process::exit(0);
        }

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
                    let selected = self.router.current() == page ||
                        (page == Page::Component && matches!(self.router.current(), Page::ComponentDetail { .. }));
                    let style = if selected {
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
                Page::ComponentDetail { .. } => (
                    "Component details",
                    "The component name is decoded from the path; the active tab is stored in search params.",
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
                &format!("URL: {}", self.router.location().to_url()),
                text::Style::new(theme),
            );
            if matches!(self.router.current(), Page::Component | Page::ComponentDetail { .. }) {
                ui.add_space(16.0);
                ui.horizontal_wrapped(|ui| {
                    for name in ["button", "text edit", "a/b"] {
                        if button::render(ui, name, button::Style::new_outline_md(theme)).clicked() {
                            self.error = self.router.navigate_with_search(
                                Page::ComponentDetail { name: name.into() },
                                SearchParams::parse("tab=preview"),
                            ).err().map(|e| e.to_string());
                        }
                    }
                });
                if let Page::ComponentDetail { name } = self.router.current() {
                    text::render(ui, &format!("Path param — name: {name}"), text::Style::new(theme));
                    let params = self.router.search_params();
                    let tab = params.get("tab").unwrap_or("preview");
                    text::render(ui, &format!("Search param — tab: {tab}"), text::Style::new(theme));
                    text::render(ui, if tab == "code" { "Code tab content" } else { "Preview tab content" }, text::Style::new(theme));
                    ui.horizontal_wrapped(|ui| {
                        for tab in ["preview", "code"] {
                            if button::render(ui, tab, button::Style::new_outline_md(theme)).clicked() {
                                let mut params = self.router.search_params();
                                params.set("tab", tab);
                                self.error = self.router.set_search_params(params, HistoryMode::Push).err().map(|e| e.to_string());
                            }
                        }
                        if button::render(ui, "Replace with code", button::Style::new_outline_md(theme)).clicked() {
                            let mut params = self.router.search_params();
                            params.set("tab", "code");
                            self.error = self.router.set_search_params(params, HistoryMode::Replace).err().map(|e| e.to_string());
                        }
                        if button::render(ui, "Clear tab", button::Style::new_outline_md(theme)).clicked() {
                            let mut params = self.router.search_params();
                            params.remove("tab");
                            self.error = self.router.set_search_params(params, HistoryMode::Push).err().map(|e| e.to_string());
                        }
                    });
                }
                ui.add_space(16.0);
            }
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
    fn detail_routes_decode_and_generate_encoded_paths() {
        for name in ["button", "text edit", "你好/世界", "a?b#c+%"] {
            let route = Page::ComponentDetail { name: name.into() };
            assert_eq!(Page::from_path(&route.to_path()), route);
        }
        assert!(matches!(
            Page::from_path("/component/%FF"),
            Page::NotFound(_)
        ));
        assert!(matches!(
            Page::from_path("/component/a/b"),
            Page::NotFound(_)
        ));
    }
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
