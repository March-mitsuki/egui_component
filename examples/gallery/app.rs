use crate::{
    docs, pages,
    routes::{Component, Doc, Page},
};
use eframe::egui::{self, Margin, Stroke};
use egui_component::{
    components::{button, card, code, navigation, scroll_area, switch, text, text_edit},
    theme::{Theme, UiTheme},
};
use egui_component_router::{HistoryMode, Router};

pub(crate) struct GalleryApp {
    router: Router<Page>,
    ui_theme: UiTheme,
    dark: bool,
    filter: String,
    mobile_nav: bool,
    active_section: String,
    pending_section: Option<String>,
    last_url: String,
    last_page: Option<Page>,
    reset_main_scroll: bool,
    error: Option<String>,
    form_page: pages::form::FormPage,
    text_page: pages::text::FontDemo,
    demo_name: String,
    notifications: bool,
    saved: bool,
}

impl GalleryApp {
    pub(crate) fn new(
        ui_theme: UiTheme,
        ctx: &egui::Context,
    ) -> Result<Self, egui_component_router::NavigationError> {
        let ctx = ctx.clone();
        Ok(Self {
            router: Router::new(Page::Home, move || ctx.request_repaint())?,
            ui_theme,
            dark: false,
            filter: String::new(),
            mobile_nav: false,
            active_section: "overview".into(),
            pending_section: None,
            last_url: String::new(),
            last_page: None,
            reset_main_scroll: true,
            error: None,
            form_page: Default::default(),
            text_page: Default::default(),
            demo_name: "My next project".into(),
            notifications: true,
            saved: false,
        })
    }
    fn go(&mut self, page: Page) {
        self.error = self.router.navigate(page).err().map(|e| e.to_string());
        self.mobile_nav = false;
        self.filter.clear();
    }
    fn jump(&mut self, id: &str) {
        let mut params = self.router.search_params();
        params.set("section", id);
        self.error = self
            .router
            .set_search_params(params, HistoryMode::Replace)
            .err()
            .map(|e| e.to_string());
        self.pending_section = Some(id.into());
    }
    fn heading(&mut self, ui: &mut egui::Ui, id: &str, title: &str, theme: &Theme) {
        ui.add_space(28.0);
        let response = text::render(
            ui,
            title,
            text::Style::new_heading(theme).size(if id == "overview" { 38.0 } else { 23.0 }),
        );
        if response.rect.top() <= ui.clip_rect().top() + 96.0 {
            self.active_section = id.into();
        }
        if self.pending_section.as_deref() == Some(id) {
            ui.scroll_to_rect(response.rect.expand(16.0), Some(egui::Align::Min));
            self.pending_section = None;
            self.active_section = id.into();
        }
        ui.add_space(12.0);
    }
    fn header(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        egui::Panel::top("site-header")
            .show_separator_line(false)
            .exact_size(if ui.available_width() < 620.0 {
                106.0
            } else {
                68.0
            })
            .resizable(false)
            .frame(
                egui::Frame::NONE
                    .fill(theme.colors.bg.base)
                    .inner_margin(Margin::symmetric(20, 12)),
            )
            .show_inside(ui, |ui| {
                let narrow = ui.available_width() < 580.0;
                ui.horizontal(|ui| {
                    if button::render(
                        ui,
                        "e",
                        button::Style::new_solid_md_color(theme, theme.colors.primary),
                    )
                    .clicked()
                    {
                        self.go(Page::Home);
                    }
                    text::render(
                        ui,
                        "egui_component",
                        text::Style::new_heading(theme).size(17.0),
                    );
                    if !narrow {
                        ui.add_space(28.0);
                        self.top_links(ui, theme);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if button::render(
                            ui,
                            if self.dark { "Light" } else { "Dark" },
                            button::Style::new_ghost_sm(theme),
                        )
                        .clicked()
                        {
                            self.dark = !self.dark;
                        }
                        if !narrow
                            && button::render(ui, "GitHub ↗", button::Style::new_ghost_sm(theme))
                                .clicked()
                        {
                            ui.ctx().open_url(egui::OpenUrl::new_tab(
                                "https://github.com/March-mitsuki/egui_component",
                            ));
                        }
                    });
                });
                if narrow {
                    ui.horizontal(|ui| self.top_links(ui, theme));
                }
            });
    }
    fn top_links(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        let current = self.router.current();
        for (page, name, selected) in [
            (Page::Home, "Home", current == Page::Home),
            (
                Page::Docs(Doc::GettingStarted),
                "Docs",
                matches!(current, Page::Docs(_)),
            ),
            (
                Page::Component(Component::Button),
                "Components",
                matches!(current, Page::Component(_)),
            ),
        ] {
            let mut style = button::Style::new_ghost_md(theme);
            if selected {
                style.palette.normal_bg = theme.colors.bg.accent;
                style.palette.text_color = theme.colors.fg.accent;
            }
            if button::render(ui, name, style).clicked() {
                self.go(page);
            }
        }
    }
    fn sidebar(&mut self, ui: &mut egui::Ui, theme: &Theme, page: &Page) {
        ui.add_space(12.0);
        eyebrow(
            ui,
            if matches!(page, Page::Docs(_)) {
                "DOCUMENTATION"
            } else {
                "THE COMPONENTS"
            },
            theme,
        );
        ui.add_space(16.0);
        if matches!(page, Page::Component(_)) {
            text_edit::render(
                ui,
                &mut self.filter,
                text_edit::Style::new_outline_sm(theme).placeholder("Find a component…"),
            );
            ui.add_space(16.0);
            let filter = self.filter.to_lowercase();
            let mut found = false;
            for group in [
                "Actions", "Inputs", "Layout", "Content", "Identity", "Feedback",
            ] {
                let items: Vec<_> = Component::ALL
                    .iter()
                    .copied()
                    .filter(|c| c.group() == group && c.name().to_lowercase().contains(&filter))
                    .collect();
                if items.is_empty() {
                    continue;
                }
                found = true;
                ui.add_space(12.0);
                muted(ui, group, theme);
                ui.add_space(4.0);
                for item in items {
                    if navigation::render(
                        ui,
                        item.name(),
                        *page == Page::Component(item),
                        navigation::Style::new(theme),
                    )
                    .clicked()
                    {
                        self.go(Page::Component(item));
                    }
                }
            }
            if !found {
                muted(ui, "No components found.", theme);
            }
        } else {
            for doc in Doc::ALL {
                if navigation::render(
                    ui,
                    doc.title(),
                    *page == Page::Docs(*doc),
                    navigation::Style::new(theme),
                )
                .clicked()
                {
                    self.go(Page::Docs(*doc));
                }
            }
            ui.add_space(28.0);
            eyebrow(ui, "EXPLORE", theme);
            ui.add_space(8.0);
            if navigation::render(
                ui,
                "Component library ↗",
                false,
                navigation::Style::new(theme),
            )
            .clicked()
            {
                self.go(Page::Component(Component::Button));
            }
        }
        ui.add_space(28.0);
        muted(ui, "Built with egui_component.", theme);
    }
    fn outline(page: &Page) -> Vec<(&'static str, &'static str)> {
        let mut result = vec![("overview", "Overview")];
        match page {
            Page::Docs(doc) => result.extend(docs::sections(*doc).iter().map(|s| (s.id, s.title))),
            Page::Component(_) => result.extend([
                ("preview", "Live preview"),
                ("usage", "Source & usage"),
                ("customization", "Customization"),
            ]),
            _ => {}
        }
        result
    }
    fn toc(&mut self, ui: &mut egui::Ui, theme: &Theme, page: &Page) {
        ui.add_space(14.0);
        eyebrow(ui, "ON THIS PAGE", theme);
        ui.add_space(12.0);
        for (id, title) in Self::outline(page) {
            let mut style = navigation::Style::new(theme);
            style.text.size = 13.0;
            if navigation::render(ui, title, self.active_section == id, style).clicked() {
                self.jump(id);
            }
        }
        ui.add_space(26.0);
        ui.add_space(18.0);
        muted(ui, "Made for immediate mode.", theme);
        ui.add_space(10.0);
        if button::render(ui, "View on GitHub ↗", button::Style::new_ghost_sm(theme)).clicked() {
            ui.ctx().open_url(egui::OpenUrl::new_tab(
                "https://github.com/March-mitsuki/egui_component",
            ));
        }
    }
    fn document(&mut self, ui: &mut egui::Ui, theme: &Theme, page: Page) {
        let width = ui.available_width();
        let wide = width >= 1100.0;
        let sidebar = width >= 760.0;
        if sidebar {
            egui::Panel::left("site-sidebar")
                .show_separator_line(false)
                .exact_size(218.0)
                .resizable(false)
                .frame(egui::Frame::NONE.fill(theme.colors.bg.base))
                .show_inside(ui, |ui| {
                    scroll_area::render(
                        ui,
                        "sidebar-scroll",
                        scroll_area::ScrollAxes::Vertical,
                        |ui| self.sidebar(ui, theme, &page),
                        scroll_area::Style::new_no_frame()
                            .auto_shrink([false, false])
                            .padding(Margin::same(16)),
                    )
                });
        }
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(theme.colors.bg.base))
            .show_inside(ui, |ui| {
                if !sidebar {
                    egui::Frame::NONE.inner_margin(Margin::symmetric(18, 12)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if button::render(ui, if self.mobile_nav { "Close navigation" } else { "Browse pages" }, button::Style::new_outline_sm(theme)).clicked() { self.mobile_nav = !self.mobile_nav; }
                        muted(ui, "Docs / Components", theme);
                    });
                    if self.mobile_nav {
                        scroll_area::render(ui, "mobile-nav", scroll_area::ScrollAxes::Vertical, |ui| self.sidebar(ui, theme, &page), scroll_area::Style::new_no_frame().max_height(260.0));
                    }
                    });
                }
                let body_rect = ui.available_rect_before_wrap();
                let outline_width = if wide { 210.0 } else { 0.0 };
                main_scroll(ui, self.reset_main_scroll, |ui| {
                    let main_width = (ui.available_width() - outline_width).max(0.0);
                    let content_width = main_width.min(820.0);
                    let inset = ((main_width - content_width) / 2.0).max(0.0);
                    ui.horizontal(|ui| {
                    if inset > 0.0 { ui.add_space((inset - ui.spacing().item_spacing.x).max(0.0)); }
                    ui.vertical(|ui| {
                    ui.set_width(content_width);
                    let (label, title, description) = match page {
                        Page::Docs(doc) => ("DOCUMENTATION", doc.title(), doc.description()),
                        Page::Component(c) => ("COMPONENTS", c.name(), c.description()),
                        _ => unreachable!(),
                    };
                    eyebrow(ui, label, theme);
                    self.heading(ui, "overview", title, theme);
                    // A larger page title while keeping section headings compact.
                    paragraph(ui, description, theme);
                    ui.add_space(20.0);
                    if !wide {
                        ui.horizontal_wrapped(|ui| {
                            for (id, title) in Self::outline(&page) {
                                if button::render(ui, title, button::Style::new_ghost_sm(theme)).clicked() { self.jump(id); }
                            }
                        });
                        ui.add_space(12.0);
                    }
                    match page {
                        Page::Docs(doc) => {
                            for section in docs::sections(doc) {
                                self.heading(ui, section.id, section.title, theme);
                                paragraph(ui, section.body, theme);
                                if let Some((language, source)) = section.code { ui.add_space(16.0); code_block(ui, source, language, theme, section.id, 360.0); }
                            }
                            ui.add_space(32.0);
                            if button::render(ui, "Explore components →", button::Style::new_solid_md(theme)).clicked() { self.go(Page::Component(Component::Button)); }
                        }
                        Page::Component(component) => {
                            self.heading(ui, "preview", "Live preview", theme);
                            muted(ui, "Real components. Try them out.", theme);
                            ui.add_space(14.0);
                            card::render(ui, |ui| {
                                ui.set_min_width((ui.available_width() - 2.0).max(0.0));
                                ui.push_id(component, |ui| {
                                    scroll_area::render(ui, "demo-viewport", scroll_area::ScrollAxes::Both,
                                        |ui| self.demo(ui, theme, component), scroll_area::Style::new_no_frame().min_height(240.0).max_height(360.0));
                                });
                            }, surface(theme));
                            self.heading(ui, "usage", "Source & usage", theme);
                            paragraph(ui, "This is the source of the live example above. Copy it into your application and adapt the style to your needs.", theme);
                            ui.add_space(16.0);
                            code_block(ui, component.source(), "rs", theme, component.slug(), 300.0);
                            self.heading(ui, "customization", "Make it yours", theme);
                            paragraph(ui, "Use the theme-aware presets as a starting point. Public style fields and builder methods let you adjust spacing, colors, typography, and shape without changing your application's layout.", theme);
                            ui.add_space(16.0);
                            if button::render(ui, "Read the styling guide →", button::Style::new_outline_md(theme)).clicked() { self.go(Page::Docs(Doc::Styling)); }
                            ui.add_space(32.0);
                            let index = Component::ALL.iter().position(|c| *c == component).unwrap();
                            ui.horizontal_wrapped(|ui| {
                                if index > 0 && button::render(ui, &format!("← {}", Component::ALL[index-1].name()), button::Style::new_ghost_md(theme)).clicked() { self.go(Page::Component(Component::ALL[index-1])); }
                                if index+1 < Component::ALL.len() && button::render(ui, &format!("{} →", Component::ALL[index+1].name()), button::Style::new_ghost_md(theme)).clicked() { self.go(Page::Component(Component::ALL[index+1])); }
                            });
                        }
                        _ => {}
                    }
                    ui.add_space(48.0);
                                ui.add_space(18.0);
                    muted(ui, "egui_component  /  Open source. Made with Rust.", theme);
                    ui.add_space(48.0);
                    });
                    });
                }, scroll_area::Style::new_no_frame()
                    .auto_shrink([false, false])
                    .padding(Margin::same(if sidebar { 32 } else { 18 })));
                if wide {
                    let rect = egui::Rect::from_min_max(
                        egui::pos2(body_rect.right() - 210.0, body_rect.top() + 32.0),
                        egui::pos2(body_rect.right() - 24.0, body_rect.bottom()),
                    );
                    let mut outline_ui = ui.new_child(egui::UiBuilder::new()
                        .id_salt("page-outline").max_rect(rect));
                    outline_ui.set_clip_rect(rect.intersect(ui.clip_rect()));
                    self.toc(&mut outline_ui, theme, &page);
                }
            });
    }
    fn demo(&mut self, ui: &mut egui::Ui, theme: &Theme, component: Component) {
        match component {
            Component::Avatar => pages::avatar::show(ui, theme),
            Component::Button => pages::button::show(ui, theme),
            Component::Card => pages::card::show(ui, theme),
            Component::Code => pages::code::show(ui, theme),
            Component::Dialog => pages::dialog::show(ui, theme),
            Component::Divider => pages::divider::show(ui, theme),
            Component::Field => pages::field::show(ui, theme),
            Component::Form => self.form_page.show(ui, theme),
            Component::Icon => pages::icon::show(ui, theme),
            Component::IconButton => pages::icon_button::show(ui, theme),
            Component::IconMenuButton => pages::icon_menu_button::show(ui, theme),
            Component::MenuButton => pages::menu_button::show(ui, theme),
            Component::Navigation => pages::navigation::show(ui, theme),
            Component::ScrollArea => pages::scroll_area::show(ui, theme),
            Component::Select => pages::select::show(ui, theme),
            Component::Separator => pages::separator::show(ui, theme),
            Component::Switch => pages::switch::show(ui, theme),
            Component::Text => self.text_page.show(ui, theme),
            Component::TextEdit => pages::text_edit::show(ui, theme),
            Component::Toast => pages::toast::show(ui, theme),
            Component::Tooltip => pages::tooltip::show(ui, theme),
        }
    }
    fn home(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(theme.colors.bg.base))
            .show_inside(ui, |ui| {
                main_scroll(ui, self.reset_main_scroll, |ui| {
                    let max = 1120.0_f32.min(ui.available_width());
                    let offset = ((ui.available_width() - max) / 2.0).max(0.0);
                    ui.horizontal(|ui| { if offset > 0.0 { ui.add_space((offset - ui.spacing().item_spacing.x).max(0.0)); } ui.vertical(|ui| {
                        ui.set_width(max);
                        ui.add_space(if max > 700.0 { 52.0 } else { 24.0 });
                        eyebrow(ui, "A SMALL LIBRARY. ROOM FOR BIG IDEAS.", theme);
                        ui.add_space(22.0);
                        if max > 850.0 {
                            ui.columns(2, |columns| {
                                self.hero(&mut columns[0], theme, max);
                                columns[1].add_space(8.0);
                                self.showcase(&mut columns[1], theme);
                            });
                        } else {
                            self.hero(ui, theme, max);
                            ui.add_space(28.0);
                            self.showcase(ui, theme);
                        }
                        ui.add_space(44.0);
                                        ui.add_space(26.0);
                        ui.horizontal_wrapped(|ui| {
                            eyebrow(ui, "RUST AT HEART", theme); ui.add_space(22.0);
                            muted(ui, "Native + Web", theme); ui.add_space(22.0);
                            muted(ui, "Light + dark", theme); ui.add_space(22.0);
                            muted(ui, "MIT licensed", theme);
                        });
                        ui.add_space(42.0);
                        text::render(ui, "Your interface. Your rules.", text::Style::new_heading(theme).size(30.0));
                        ui.add_space(10.0);
                        paragraph(ui, "Thoughtful defaults, flexible building blocks. Start small and make something that feels like you.", theme);
                        ui.add_space(24.0);
                        if max > 700.0 {
                            ui.columns(3, |columns| {
                                feature(&mut columns[0], "01 / COMPOSE", "Fits your application", "Render directly into your egui Ui. Keep your architecture, state, and layout.", theme);
                                feature(&mut columns[1], "02 / CUSTOMIZE", "A style of your own", "Start with a preset, then shape every color, corner, and interaction.", theme);
                                feature(&mut columns[2], "03 / SHIP", "Go where Rust goes", "Build for desktop and Web with the same components and typed navigation.", theme);
                            });
                        } else {
                            feature(ui, "01 / COMPOSE", "Fits your application", "Render directly into your egui Ui. Keep your architecture, state, and layout.", theme);
                            ui.add_space(12.0);
                            feature(ui, "02 / CUSTOMIZE", "A style of your own", "Start with a preset, then shape every color, corner, and interaction.", theme);
                            ui.add_space(12.0);
                            feature(ui, "03 / SHIP", "Go where Rust goes", "Build for desktop and Web with the same components and typed navigation.", theme);
                        }
                        ui.add_space(48.0);
                        ui.horizontal_wrapped(|ui| {
                            text::render(ui, "A few building blocks", text::Style::new_heading(theme).size(26.0));
                            if button::render(ui, "Explore the library →", button::Style::new_ghost_md(theme)).clicked() { self.go(Page::Component(Component::Button)); }
                        });
                        ui.add_space(18.0);
                        ui.horizontal_wrapped(|ui| {
                            for c in [Component::Button, Component::Form, Component::Select, Component::Code, Component::Switch] {
                                if button::render(ui, c.name(), button::Style::new_outline_md(theme)).clicked() { self.go(Page::Component(c)); }
                            }
                        });
                        ui.add_space(52.0);
                                        ui.add_space(24.0);
                        muted(ui, "egui_component · Built with its own components · MIT license", theme);
                        ui.add_space(26.0);
                    }); });
                }, scroll_area::Style::new_no_frame().auto_shrink([false, false]).padding(Margin::same(24)));
            });
    }
    fn hero(&mut self, ui: &mut egui::Ui, theme: &Theme, width: f32) {
        text::render(
            ui,
            "Build interfaces.\nKeep your freedom.",
            text::Style::new_heading(theme).size(if width > 850.0 { 48.0 } else { 38.0 }),
        );
        ui.add_space(20.0);
        paragraph(
            ui,
            "Composable components for egui. Beautiful defaults, precise styling, and no framework standing between you and your next idea.",
            theme,
        );
        ui.add_space(24.0);
        ui.horizontal_wrapped(|ui| {
            if button::render(ui, "Get started →", button::Style::new_solid_lg(theme)).clicked() {
                self.go(Page::Docs(Doc::GettingStarted));
            }
            if button::render(
                ui,
                "Browse components",
                button::Style::new_outline_lg(theme),
            )
            .clicked()
            {
                self.go(Page::Component(Component::Button));
            }
        });
        ui.add_space(20.0);
        muted(
            ui,
            "v0.1.0-alpha.2   /   Open source, from the start.",
            theme,
        );
    }
    fn showcase(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        card::render(
            ui,
            |ui| {
                ui.set_min_width((ui.available_width() - 2.0).max(0.0));
                eyebrow(ui, "LIVE / MADE WITH EGUI_COMPONENT", theme);
                ui.add_space(22.0);
                text::render(
                    ui,
                    "A little space to create.",
                    text::Style::new_heading(theme).size(23.0),
                );
                ui.add_space(8.0);
                muted(ui, "Try the controls. This is the real thing.", theme);
                ui.add_space(24.0);
                text::render(ui, "Project name", text::Style::new(theme).size(13.0));
                text_edit::render(
                    ui,
                    &mut self.demo_name,
                    text_edit::Style::new_outline_lg(theme),
                );
                ui.add_space(18.0);
                ui.horizontal(|ui| {
                    text::render(
                        ui,
                        "Keep me in the loop",
                        text::Style::new(theme).size(14.0),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        switch::render(ui, &mut self.notifications, switch::Style::new(theme));
                    });
                });
                ui.add_space(20.0);
                ui.add_space(18.0);
                ui.horizontal_wrapped(|ui| {
                    if button::render(
                        ui,
                        if self.saved {
                            "Created ✓"
                        } else {
                            "Create project"
                        },
                        button::Style::new_solid_md_color(theme, theme.colors.primary),
                    )
                    .clicked()
                    {
                        self.saved = !self.saved;
                    }
                    muted(
                        ui,
                        if self.saved {
                            "Your idea has a home."
                        } else {
                            "One idea. Endless possibilities."
                        },
                        theme,
                    );
                });
            },
            surface(theme).inner_margin(Margin::same(26)),
        );
    }
}

impl GalleryApp {
    fn render(&mut self, ui: &mut egui::Ui) {
        #[cfg(not(target_arch = "wasm32"))]
        if ui.ctx().input(|i| i.viewport().close_requested()) {
            // prevent macOS touch bar crash report
            std::process::exit(0);
        }

        self.text_page.poll_download(ui.ctx());
        egui_component::consts::FONT_MANAGER.apply(ui.ctx());
        let theme = if self.dark {
            self.ui_theme.dark.clone()
        } else {
            self.ui_theme.light.clone()
        };
        let mut visuals = if self.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.panel_fill = theme.colors.bg.base;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, theme.colors.gray.l1);
        ui.ctx().set_visuals(visuals);
        ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
        let page = self.router.current();
        self.reset_main_scroll = self.last_page.as_ref() != Some(&page);
        self.last_page = Some(page.clone());
        let url = self.router.location().to_url();
        if self.last_url != url {
            self.pending_section = self
                .router
                .search_params()
                .get("section")
                .map(str::to_owned);
            self.last_url = url;
            self.active_section = "overview".into();
        }
        let previous_section = self.active_section.clone();
        self.header(ui, &theme);
        match page {
            Page::Home => self.home(ui, &theme),
            page @ (Page::Docs(_) | Page::Component(_)) => self.document(ui, &theme, page),
            Page::NotFound(path) => {
                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::NONE
                            .fill(theme.colors.bg.base)
                            .inner_margin(Margin::same(36)),
                    )
                    .show_inside(ui, |ui| {
                        ui.add_space(60.0);
                        eyebrow(ui, "404 / A SMALL DETOUR", &theme);
                        ui.add_space(18.0);
                        text::render(
                            ui,
                            "This page isn't here.",
                            text::Style::new_heading(&theme).size(38.0),
                        );
                        paragraph(
                            ui,
                            &format!("We couldn't find {path}. Let's get you back to building."),
                            &theme,
                        );
                        ui.add_space(24.0);
                        if button::render(ui, "Back to home →", button::Style::new_solid_md(&theme))
                            .clicked()
                        {
                            self.go(Page::Home);
                        }
                    });
            }
        }
        if let Some(error) = &self.error {
            egui::Window::new("Navigation error").show(ui.ctx(), |ui| {
                text::render(ui, error, text::Style::new(&theme));
            });
        }
        if self.active_section != previous_section {
            ui.ctx().request_repaint();
        }
        egui_component::toast::renderer(ui, &theme);
    }
}
impl eframe::App for GalleryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.render(ui);
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;
    fn red_rects(shapes: &[egui::epaint::ClippedShape]) -> Vec<egui::Rect> {
        fn collect(shape: &egui::Shape, rects: &mut Vec<egui::Rect>) {
            match shape {
                egui::Shape::Rect(r) if r.stroke.color == egui::Color32::RED => rects.push(r.rect),
                egui::Shape::Vec(shapes) => {
                    for shape in shapes {
                        collect(shape, rects);
                    }
                }
                _ => {}
            }
        }
        let mut rects = Vec::new();
        for shape in shapes {
            collect(&shape.shape, &mut rects);
        }
        rects
    }
    #[test]
    fn header_clicks_have_no_debug_error_borders() {
        let ctx = egui::Context::default();
        let theme = UiTheme::from_json_bytes(include_bytes!("../theme.json")).unwrap();
        let mut app = GalleryApp::new(theme, &ctx).unwrap();
        let mut events = Vec::new();
        let frame = |events: Vec<egui::Event>, app: &mut GalleryApp| {
            ctx.run_ui(
                egui::RawInput {
                    events,
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1280.0, 800.0),
                    )),
                    ..Default::default()
                },
                |ui| app.render(ui),
            )
        };
        for _ in 0..2 {
            frame(events.clone(), &mut app);
        }
        for (pos, expected) in [
            (egui::pos2(341.0, 29.0), Page::Docs(Doc::GettingStarted)),
            (egui::pos2(432.0, 29.0), Page::Component(Component::Button)),
            (egui::pos2(270.0, 29.0), Page::Home),
        ] {
            for pressed in [true, false] {
                events = vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ];
                let output = frame(events.clone(), &mut app);
                assert!(
                    red_rects(&output.shapes).is_empty(),
                    "debug borders clicking {pos:?} on {:?}: {:?}",
                    app.router.current(),
                    red_rects(&output.shapes)
                );
            }
            let output = frame(Vec::new(), &mut app);
            assert_eq!(app.router.current(), expected);
            assert!(red_rects(&output.shapes).is_empty());
        }
    }
    #[test]
    fn navigation_has_no_debug_error_borders() {
        let ctx = egui::Context::default();
        let theme = UiTheme::from_json_bytes(include_bytes!("../theme.json")).unwrap();
        let mut app = GalleryApp::new(theme, &ctx).unwrap();
        for page in [
            Page::Home,
            Page::Docs(Doc::GettingStarted),
            Page::Component(Component::Button),
            Page::Docs(Doc::Styling),
            Page::Home,
        ] {
            app.go(page.clone());
            for _ in 0..2 {
                let output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1280.0, 800.0),
                        )),
                        ..Default::default()
                    },
                    |ui| app.render(ui),
                );
                assert!(
                    red_rects(&output.shapes).is_empty(),
                    "debug borders on {page:?}: {:?}",
                    red_rects(&output.shapes)
                );
            }
        }
    }
}

/// One stable viewport for all main pages. Page changes reset its offset, rather
/// than replacing the scrollbar's ID at the same screen location.
fn main_scroll(
    ui: &mut egui::Ui,
    reset: bool,
    render: impl FnOnce(&mut egui::Ui),
    mut style: scroll_area::Style,
) {
    if reset {
        style = style.scroll_offset(egui::Vec2::ZERO);
    }
    ui.scope_builder(
        egui::UiBuilder::new().id(egui::Id::new("site-main-viewport")),
        |ui| {
            scroll_area::render(
                ui,
                "site-main-scroll",
                scroll_area::ScrollAxes::Vertical,
                render,
                style,
            );
        },
    );
}

fn eyebrow(ui: &mut egui::Ui, label: &str, theme: &Theme) {
    text::render(
        ui,
        label,
        text::Style::new_heading(theme)
            .size(11.0)
            .color(theme.colors.primary.l6),
    );
}
fn muted(ui: &mut egui::Ui, label: &str, theme: &Theme) {
    text::render(
        ui,
        label,
        text::Style::new(theme)
            .size(13.0)
            .color(theme.colors.gray.l6),
    );
}
fn paragraph(ui: &mut egui::Ui, label: &str, theme: &Theme) {
    text::render(ui, label, text::Style::new(theme).size(15.0));
}
fn surface(theme: &Theme) -> card::Style {
    card::Style::new_md(theme)
        .shadow(egui::Shadow::NONE)
        .stroke(Stroke::new(1.0_f32, theme.colors.gray.l1))
        .corner_radius(egui::CornerRadius::same(12))
}
fn code_block(
    ui: &mut egui::Ui,
    source: &str,
    language: &str,
    theme: &Theme,
    id: &str,
    height: f32,
) {
    card::render(
        ui,
        |ui| {
            ui.horizontal(|ui| {
                eyebrow(ui, language, theme);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if button::render(ui, "Copy", button::Style::new_ghost_sm(theme)).clicked() {
                        ui.ctx().copy_text(source.to_owned());
                    }
                });
            });
            ui.add_space(8.0);
            scroll_area::render(
                ui,
                ("source", id),
                scroll_area::ScrollAxes::Both,
                |ui| {
                    code::render(ui, source, language, code::Style::new(theme).size(13.0));
                },
                scroll_area::Style::new_no_frame()
                    .max_height(height)
                    .min_height((source.lines().count() as f32 * 18.0 + 20.0).min(height))
                    .auto_shrink([false, true]),
            );
        },
        surface(theme).fill(theme.colors.bg.subtle),
    );
}
fn feature(ui: &mut egui::Ui, number: &str, title: &str, body: &str, theme: &Theme) {
    card::render(
        ui,
        |ui| {
            ui.set_min_width((ui.available_width() - 2.0).max(0.0));
            ui.set_min_height(150.0);
            eyebrow(ui, number, theme);
            ui.add_space(18.0);
            text::render(ui, title, text::Style::new_heading(theme).size(20.0));
            ui.add_space(8.0);
            paragraph(ui, body, theme);
        },
        surface(theme).inner_margin(Margin::same(20)),
    );
}
