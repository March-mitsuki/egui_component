use eframe::egui;
use egui_component::components::button;
use egui_component::theme::{UiTheme, Theme};

fn main() -> eframe::Result {
    // Load theme from theme.json file
    let ui_theme =
        UiTheme::from_json_bytes(include_bytes!("../theme.json")).expect("Failed to load theme.json");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0])
            .with_title("egui_component Storybook Gallery"),
        ..Default::default()
    };

    eframe::run_native(
        "egui_component Gallery",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(GalleryApp {
                ui_theme,
                color_mode: ColorMode::Dark,
                selected_tab: ComponentTab::Button,
                search_query: String::new(),
            }))
        }),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColorMode {
    Dark,
    Light,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComponentTab {
    Avatar,
    Button,
    Card,
    Dialog,
    Divider,
    Field,
    Form,
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

impl ComponentTab {
    fn name(&self) -> &'static str {
        match self {
            Self::Avatar => "Avatar",
            Self::Button => "Button",
            Self::Card => "Card",
            Self::Dialog => "Dialog",
            Self::Divider => "Divider",
            Self::Field => "Field",
            Self::Form => "Form",
            Self::Icon => "Icon",
            Self::IconButton => "IconButton",
            Self::IconMenuButton => "IconMenuButton",
            Self::MenuButton => "MenuButton",
            Self::ScrollArea => "ScrollArea",
            Self::Separator => "Separator",
            Self::Text => "Text",
            Self::TextEdit => "TextEdit",
            Self::Toast => "Toast",
            Self::Tooltip => "Tooltip",
        }
    }

    fn category(&self) -> &'static str {
        match self {
            Self::Avatar | Self::Icon | Self::Text => "Data Display",
            Self::Button | Self::IconButton | Self::IconMenuButton | Self::MenuButton => "Navigation & Action",
            Self::Card | Self::Divider | Self::ScrollArea | Self::Separator => "Layout & Structure",
            Self::Field | Self::Form | Self::TextEdit => "Inputs & Forms",
            Self::Dialog | Self::Toast | Self::Tooltip => "Feedback & Overlays",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::Avatar => "A visual representation of a user or entity, supporting initials and fallback styling.",
            Self::Button => "A highly customizable button supporting various variants (Solid, Surface, Outline, Ghost), sizes, loading states, and icons.",
            Self::Card => "A container for grouping related content and actions with customizable padding and rounded corners.",
            Self::Dialog => "A modal dialog or popup that sits on top of the main application window to focus user attention.",
            Self::Divider => "A visual line that separates content with optional custom text or styling.",
            Self::Field => "A labeled form field container that integrates input validation, descriptions, and error states.",
            Self::Form => "A structured container for laying out and managing complex interactive form inputs.",
            Self::Icon => "A utility for rendering beautiful vector/raster system icons consistently.",
            Self::IconButton => "A compact, icon-only button variant ideal for dense toolbars or quick actions.",
            Self::IconMenuButton => "A dropdown menu trigger styled specifically with a lead icon.",
            Self::MenuButton => "A trigger that opens dropdown or context menus with smooth states.",
            Self::ScrollArea => "A customizable scroll container supporting smooth scrollbars and premium aesthetics.",
            Self::Separator => "A structural separator separating elements or sections within a layout.",
            Self::Text => "A highly customized typography helper supporting sizes matching the design system.",
            Self::TextEdit => "A premium single/multiline text input supporting validation, placeholders, and focus rings.",
            Self::Toast => "A notification system for displaying temporary non-intrusive alert messages.",
            Self::Tooltip => "A lightweight popover displaying helpful information on hover or focus.",
        }
    }
}

struct GalleryApp {
    ui_theme: UiTheme,
    color_mode: ColorMode,
    selected_tab: ComponentTab,
    search_query: String,
}

fn render_header(
    color_mode: &mut ColorMode,
    ui: &mut egui::Ui,
    theme: &Theme,
) {
    ui.horizontal(|ui| {
        // Brand details
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("✨ egui_component")
                        .size(18.0)
                        .strong()
                        .color(theme.colors.primary.normal()),
                );
                ui.label(
                    egui::RichText::new("STORYBOOK GALLERY")
                        .size(10.0)
                        .strong()
                        .color(theme.colors.gray.l5),
                );
            });
            ui.label(
                egui::RichText::new("A library of headless, customizable egui components")
                    .size(12.0)
                    .color(theme.colors.fg.subtle),
            );
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Theme Toggle
            let button_text = match color_mode {
                ColorMode::Dark => "🌙 Dark Mode",
                ColorMode::Light => "☀️ Light Mode",
            };

            let primary_color = theme.colors.primary;
            if button::render(
                ui,
                button_text,
                button::Style::new_surface_md_color(theme, primary_color).width(Some(120.0)),
            )
            .clicked()
            {
                *color_mode = match color_mode {
                    ColorMode::Dark => ColorMode::Light,
                    ColorMode::Light => ColorMode::Dark,
                };
            }

            ui.add_space(16.0);

            // Small badge
            let version_color = theme.colors.gray.l4;
            ui.colored_label(version_color, "v0.1.0-alpha.2");
        });
    });
}

fn render_sidebar(
    selected_tab: &mut ComponentTab,
    search_query: &mut String,
    ui: &mut egui::Ui,
    theme: &Theme,
) {
    ui.vertical(|ui| {
        // Search Input
        ui.label(egui::RichText::new("Filter Components").size(11.0).strong().color(theme.colors.fg.subtle));
        ui.add_space(4.0);
        
        // Search Bar TextEdit
        ui.horizontal(|ui| {
            ui.text_edit_singleline(search_query);
            if !search_query.is_empty() {
                if ui.small_button("Clear").clicked() {
                    search_query.clear();
                }
            }
        });

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        // Group components by category
        let categories = ["Navigation & Action", "Inputs & Forms", "Layout & Structure", "Data Display", "Feedback & Overlays"];
        
        let all_tabs = [
            ComponentTab::Avatar,
            ComponentTab::Button,
            ComponentTab::Card,
            ComponentTab::Dialog,
            ComponentTab::Divider,
            ComponentTab::Field,
            ComponentTab::Form,
            ComponentTab::Icon,
            ComponentTab::IconButton,
            ComponentTab::IconMenuButton,
            ComponentTab::MenuButton,
            ComponentTab::ScrollArea,
            ComponentTab::Separator,
            ComponentTab::Text,
            ComponentTab::TextEdit,
            ComponentTab::Toast,
            ComponentTab::Tooltip,
        ];

        egui::ScrollArea::vertical().show(ui, |ui| {
            for category in categories {
                // Filter tabs within this category matching the search query
                let filtered_tabs: Vec<&ComponentTab> = all_tabs
                    .iter()
                    .filter(|t| t.category() == category)
                    .filter(|t| {
                        search_query.is_empty()
                            || t.name().to_lowercase().contains(&search_query.to_lowercase())
                    })
                    .collect();

                if filtered_tabs.is_empty() {
                    continue;
                }

                // Category header
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(category.to_uppercase())
                        .size(10.0)
                        .strong()
                        .color(theme.colors.gray.l5),
                );
                ui.add_space(4.0);

                // Render component navigation list items
                for tab in filtered_tabs {
                    let is_selected = *selected_tab == *tab;
                    
                    // Let's render a custom styled list item/button
                    let button_style = if is_selected {
                        button::Style::new_solid_sm_color(theme, theme.colors.primary).width(Some(200.0))
                    } else {
                        button::Style::new_ghost_sm(theme).width(Some(200.0))
                    };

                    if button::render(ui, tab.name(), button_style).clicked() {
                        *selected_tab = *tab;
                    }
                    ui.add_space(2.0);
                }
            }
        });
    });
}

fn render_main_content(
    active_tab: ComponentTab,
    ui: &mut egui::Ui,
    theme: &Theme,
) {
    ui.vertical(|ui| {
        // Page Header
        ui.horizontal(|ui| {
            ui.heading(
                egui::RichText::new(active_tab.name())
                    .size(24.0)
                    .strong()
                    .color(theme.colors.fg.base),
            );
            
            ui.add_space(8.0);
            
            // Add category tag
            let category_text = theme.colors.primary.normal();
            ui.label(
                egui::RichText::new(active_tab.category())
                    .size(11.0)
                    .color(category_text)
            );
        });
        
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(active_tab.description())
                .size(13.0)
                .color(theme.colors.fg.subtle),
        );
        
        ui.add_space(16.0);

        // Large visual showcase container
        ui.label(egui::RichText::new("Interactive Showcase").size(14.0).strong());
        ui.add_space(6.0);

        // Draw a beautiful container box representing the showcase
        let frame = egui::Frame::NONE
            .fill(theme.colors.bg.accent)
            .inner_margin(egui::Margin::same(24))
            .corner_radius(egui::CornerRadius::same(theme.corner_radius.md as u8))
            .stroke(egui::Stroke::new(1.0, theme.colors.gray.l2));

        frame.show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(
                    egui::RichText::new("🚧 Interactive Example Container")
                        .size(18.0)
                        .strong()
                        .color(theme.colors.fg.base),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(format!(
                        "This area is designated for live, fully-interactive examples showcasing the `{}` component.",
                        active_tab.name()
                    ))
                    .size(13.0)
                    .color(theme.colors.fg.subtle),
                );
                ui.add_space(20.0);

                // Add a tiny live demo widget to show that the theme and widgets are alive!
                ui.horizontal(|ui| {
                    ui.shrink_width_to_current();
                    // Render standard live components for demonstration depending on the active tab
                    match active_tab {
                        ComponentTab::Button => {
                            button::render(ui, "Primary Solid", button::Style::new_solid_md_color(theme, theme.colors.primary));
                            button::render(ui, "Outline Variant", button::Style::new_outline_md(theme));
                            button::render(ui, "Ghost Variant", button::Style::new_ghost_md(theme));
                        }
                        ComponentTab::Text => {
                            ui.label(egui::RichText::new("Large Title").size(theme.text_size.lg).strong());
                            ui.label(egui::RichText::new("Medium Body").size(theme.text_size.md));
                            ui.label(egui::RichText::new("Small Caption").size(theme.text_size.xs).color(theme.colors.fg.subtle));
                        }
                        _ => {
                            // Default nice button inside the placeholder
                            button::render(
                                ui,
                                "Interactive Demo Button",
                                button::Style::new_solid_md_color(theme, theme.colors.primary),
                            );
                        }
                    }
                });

                ui.add_space(40.0);
            });
        });

        ui.add_space(20.0);

        // API / Documentation Scaffold section below it
        ui.label(egui::RichText::new("API Properties Guide").size(14.0).strong());
        ui.add_space(6.0);

        // Table of mock props
        let doc_frame = egui::Frame::NONE
            .fill(theme.colors.bg.subtle)
            .inner_margin(egui::Margin::same(12))
            .corner_radius(egui::CornerRadius::same(theme.corner_radius.sm as u8));

        doc_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Style API Props").strong().color(theme.colors.primary.normal()));
            });
            ui.separator();
            ui.add_space(4.0);

            // Quick table representation using egui layouts
            egui::Grid::new("api_grid")
                .striped(true)
                .spacing([24.0, 8.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Property").strong());
                    ui.label(egui::RichText::new("Type").strong());
                    ui.label(egui::RichText::new("Description").strong());
                    ui.end_row();

                    ui.label("height");
                    ui.colored_label(theme.colors.orange.normal(), "f32");
                    ui.label("Defines the vertical height of the component layout.");
                    ui.end_row();

                    ui.label("palette");
                    ui.colored_label(theme.colors.orange.normal(), "Palette");
                    ui.label("Custom set of background, text, hover, active, and stroke color configuration.");
                    ui.end_row();

                    ui.label("corner_radius");
                    ui.colored_label(theme.colors.orange.normal(), "CornerRadius");
                    ui.label("Custom rounding parameters for the box container.");
                    ui.end_row();
                });
        });
    });
}

impl eframe::App for GalleryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
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

        // Top Header
        let header_frame = egui::Frame::NONE
            .fill(theme.colors.bg.subtle)
            .inner_margin(egui::Margin {
                left: 16,
                right: 16,
                top: 12,
                bottom: 12,
            })
            .stroke(egui::Stroke::new(1.0, theme.colors.gray.l2));

        header_frame.show(ui, |ui| {
            render_header(&mut self.color_mode, ui, theme);
        });

        // Main Layout Container
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(theme.colors.bg.base))
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    // Left Sidebar Panel
                    let sidebar_frame = egui::Frame::NONE
                        .fill(theme.colors.bg.subtle)
                        .inner_margin(egui::Margin {
                            left: 12,
                            right: 12,
                            top: 16,
                            bottom: 16,
                        })
                        .stroke(egui::Stroke::new(1.0, theme.colors.gray.l2));

                    ui.allocate_ui_with_layout(
                        egui::vec2(220.0, ui.available_height()),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            sidebar_frame.show(ui, |ui| {
                                render_sidebar(&mut self.selected_tab, &mut self.search_query, ui, theme);
                            });
                        },
                    );

                    ui.add_space(4.0);

                    // Right Main Panel
                    let main_frame = egui::Frame::NONE
                        .fill(theme.colors.bg.base)
                        .inner_margin(egui::Margin {
                            left: 16,
                            right: 16,
                            top: 16,
                            bottom: 16,
                        });

                    egui::ScrollArea::vertical()
                        .id_salt("main_scroll")
                        .show(ui, |ui| {
                            ui.allocate_ui_with_layout(
                                egui::vec2(ui.available_width(), ui.available_height()),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    main_frame.show(ui, |ui| {
                                        render_main_content(self.selected_tab, ui, theme);
                                    });
                                },
                            );
                        });
                });
            });
    }
}
