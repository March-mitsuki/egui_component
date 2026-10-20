use eframe::egui;
use egui_component::components::{button, scroll_area, separator, text};
use egui_component::theme::UiTheme;

use crate::pages;

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
    Code,
    Dialog,
    Divider,
    Field,
    Form,
    Icon,
    IconButton,
    IconMenuButton,
    MenuButton,
    ScrollArea,
    Select,
    Separator,
    Switch,
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
            Page::Code => "Code",
            Page::Dialog => "Dialog",
            Page::Divider => "Divider",
            Page::Field => "Field",
            Page::Form => "Form",
            Page::Icon => "Icon",
            Page::IconButton => "Icon Button",
            Page::IconMenuButton => "Icon Menu Button",
            Page::MenuButton => "Menu Button",
            Page::ScrollArea => "Scroll Area",
            Page::Select => "Select",
            Page::Separator => "Separator",
            Page::Switch => "Switch",
            Page::Text => "Text",
            Page::TextEdit => "Text Edit",
            Page::Toast => "Toast",
            Page::Tooltip => "Tooltip",
        }
    }
}

pub(crate) struct GalleryApp {
    ui_theme: UiTheme,
    color_mode: ColorMode,
    current_page: Page,
    form_page: pages::form::FormPage,
    text_page: pages::text::FontDemo,
}

impl GalleryApp {
    pub(crate) fn new(ui_theme: UiTheme) -> Self {
        Self {
            ui_theme,
            color_mode: ColorMode::Dark,
            current_page: Page::Button,
            form_page: Default::default(),
            text_page: Default::default(),
        }
    }
}

impl eframe::App for GalleryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.text_page.poll_download(ui.ctx());
        egui_component::consts::FONT_MANAGER.apply(ui.ctx());
        #[cfg(not(target_arch = "wasm32"))]
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
                    Page::Code,
                    Page::Dialog,
                    Page::Divider,
                    Page::Field,
                    Page::Form,
                    Page::Icon,
                    Page::IconButton,
                    Page::IconMenuButton,
                    Page::MenuButton,
                    Page::ScrollArea,
                    Page::Select,
                    Page::Separator,
                    Page::Switch,
                    Page::Text,
                    Page::TextEdit,
                    Page::Toast,
                    Page::Tooltip,
                ];

                scroll_area::render(
                    ui,
                    "nav_scroll",
                    scroll_area::ScrollAxes::Vertical,
                    |ui| {
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
                    },
                    scroll_area::Style::new_no_frame(),
                );
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
                    Page::Button => pages::button::show(ui, theme),
                    Page::Card => pages::card::show(ui, theme),
                    Page::Code => pages::code::show(ui, theme),
                    Page::Dialog => pages::dialog::show(ui, theme),
                    Page::Divider => pages::divider::show(ui, theme),
                    Page::Field => pages::field::show(ui, theme),
                    Page::Form => self.form_page.show(ui, theme),
                    Page::Icon => pages::icon::show(ui, theme),
                    Page::IconButton => pages::icon_button::show(ui, theme),
                    Page::IconMenuButton => pages::icon_menu_button::show(ui, theme),
                    Page::MenuButton => pages::menu_button::show(ui, theme),
                    Page::ScrollArea => pages::scroll_area::show(ui, theme),
                    Page::Select => pages::select::show(ui, theme),
                    Page::Separator => pages::separator::show(ui, theme),
                    Page::Switch => pages::switch::show(ui, theme),
                    Page::Text => self.text_page.show(ui, theme),
                    Page::TextEdit => pages::text_edit::show(ui, theme),
                    Page::Toast => pages::toast::show(ui, theme),
                    Page::Tooltip => pages::tooltip::show(ui, theme),
                }
            });
        egui_component::toast::renderer(ui, theme);
    }
}
