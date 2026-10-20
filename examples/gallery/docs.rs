use crate::routes::Doc;

pub struct Section {
    pub id: &'static str,
    pub title: &'static str,
    pub body: &'static str,
    pub code: Option<(&'static str, &'static str)>,
}
pub fn sections(doc: Doc) -> &'static [Section] {
    match doc {
        Doc::GettingStarted => &[
            Section {
                id: "installation",
                title: "Installation",
                body: "Add egui_component to an existing egui application. Components render inside your current Ui; there is no application framework to adopt.",
                code: Some((
                    "toml",
                    "[dependencies]\negui = \"0.34\"\negui_component = \"0.1.0-alpha.2\"",
                )),
            },
            Section {
                id: "theme",
                title: "Load a theme",
                body: "Load your theme JSON once when the application starts. The gallery's examples/theme.json contains a complete light and dark theme you can use as a starting point.",
                code: Some((
                    "rs",
                    "use egui_component::theme::UiTheme;\n\nlet ui_theme = UiTheme::from_json_bytes(\n    include_bytes!(\"theme.json\"),\n)?;\nlet theme = &ui_theme.light;",
                )),
            },
            Section {
                id: "first-component",
                title: "Your first component",
                body: "Pick a style preset and render directly into your Ui. Every component returns or exposes the interaction state you need to connect it to your application.",
                code: Some((
                    "rs",
                    "use egui_component::components::button;\n\nif button::render(\n    ui, \"Create project\",\n    button::Style::new_solid_md(theme),\n).clicked() {\n    // Handle the action.\n}",
                )),
            },
            Section {
                id: "next-steps",
                title: "Where to go next",
                body: "Explore the live component examples, then customize presets to match your product. Components work in both native eframe applications and WebAssembly applications.",
                code: None,
            },
        ],
        Doc::Styling => &[
            Section {
                id: "presets",
                title: "Start with a preset",
                body: "Style constructors give you a consistent baseline. Buttons include solid, surface, outline, and ghost variants, with multiple sizes and theme color palettes.",
                code: Some((
                    "rs",
                    "let solid = button::Style::new_solid_md(theme);\nlet outline = button::Style::new_outline_md(theme);\nlet quiet = button::Style::new_ghost_sm(theme);",
                )),
            },
            Section {
                id: "customize",
                title: "Customize the details",
                body: "Style values are ordinary Rust structs. Use builder methods for common changes or edit public fields for precise control. Styling stays local to the component.",
                code: Some((
                    "rs",
                    "let mut style = button::Style::new_solid_md(theme)\n    .height(40.0)\n    .width(Some(180.0));\nstyle.corner_radius = egui::CornerRadius::same(12);\nbutton::render(ui, \"Make it yours\", style);",
                )),
            },
            Section {
                id: "compose",
                title: "Compose with egui",
                body: "Use egui layouts to arrange components. Cards, fields, and scroll areas accept rendering closures, so the composition remains in your hands.",
                code: Some((
                    "rs",
                    "use egui_component::components::{card, text};\n\ncard::render(ui, |ui| {\n    text::render(ui, \"Your workspace\",\n        text::Style::new_heading(theme));\n}, card::Style::new_md(theme));",
                )),
            },
        ],
        Doc::Theming => &[
            Section {
                id: "tokens",
                title: "A shared set of tokens",
                body: "Themes hold semantic foreground and background colors, color scales, typography sizes, corner radii, and component sizing. Presets translate those tokens into component styles.",
                code: Some((
                    "rs",
                    "let foreground = theme.colors.fg.base;\nlet background = theme.colors.bg.base;\nlet accent = theme.colors.primary.l6;",
                )),
            },
            Section {
                id: "brand-colors",
                title: "Our brand palette",
                body: "The gallery uses #6B65F5 on white with #202124 text in light mode, and #8B85FF on #101318 with #F6F7FB text in dark mode. Configure these in examples/theme.json. The optional foreground on a color scale controls text and icons on solid colored buttons; old theme files keep their existing behavior.",
                code: Some((
                    "rs",
                    "use egui::Color32;\n\nlet theme = &mut ui_theme.light;\ntheme.colors.primary.l6 = Color32::from_rgb(107, 101, 245);\ntheme.colors.bg.base = Color32::WHITE;\ntheme.colors.fg.base = Color32::from_rgb(32, 33, 36);\ntheme.colors.primary.foreground = Some(Color32::WHITE);",
                )),
            },
            Section {
                id: "color-mode",
                title: "Light and dark",
                body: "UiTheme contains both modes. Select the active theme when rendering. This website's header switches modes for the entire application, including every live demo.",
                code: Some((
                    "rs",
                    "let theme = if dark_mode {\n    &ui_theme.dark\n} else {\n    &ui_theme.light\n};\ntext::render(ui, \"Hello, theme\", text::Style::new(theme));",
                )),
            },
            Section {
                id: "fonts",
                title: "Fonts that travel with your app",
                body: "Bundle font bytes for portable native and Web typography. Apply font-manager updates before rendering each frame. The gallery bundles a CJK fallback for its multilingual demos.",
                code: Some((
                    "rs",
                    "use egui_component::consts::FONT_MANAGER;\n\nFONT_MANAGER.load_from_bytes(\n    \"my-font\", include_bytes!(\"font.ttf\"), 0,\n)?;\n// Before rendering each frame:\nFONT_MANAGER.apply(ui.ctx());",
                )),
            },
        ],
        Doc::Routing => &[
            Section {
                id: "typed-routes",
                title: "Define typed routes",
                body: "egui_component_router is a separate workspace crate. Implement Route for your page enum; the same navigation API runs on native desktop and Web. The router does not depend on the UI library.",
                code: Some((
                    "toml",
                    "# Within this repository\n[dependencies]\negui_component_router = { path = \"router\" }",
                )),
            },
            Section {
                id: "parameters",
                title: "Paths and search params",
                body: "PathPattern matches named segments and builds encoded paths. SearchParams handles duplicate keys and query encoding. The gallery uses typed component routes and the section query parameter for page navigation.",
                code: Some((
                    "rs",
                    "use egui_component_router::{PathPattern, SearchParams};\n\nlet pattern = PathPattern::new(\"/components/:name\")?;\nlet path = pattern.build(&[(\"name\", \"button\")])?;\nlet mut search = SearchParams::new();\nsearch.set(\"section\", \"usage\");",
                )),
            },
            Section {
                id: "history",
                title: "History on both platforms",
                body: "Native navigation stores locations in memory. Web uses pathname URLs and browser history. Search-only updates can push or replace an entry. Unknown paths map to the application's 404 page.",
                code: Some((
                    "rs",
                    "use egui_component_router::HistoryMode;\n\nlet mut search = router.search_params();\nsearch.set(\"section\", \"usage\");\nrouter.set_search_params(search, HistoryMode::Replace)?;",
                )),
            },
            Section {
                id: "deployment",
                title: "Deploying the Web app",
                body: "Serve existing assets normally and fall back to index.html for page requests. Trunk's SPA fallback is enabled for development. An application 404 page is rendered after the document loads; an HTTP 404 status requires server-side configuration.",
                code: Some((
                    "nginx",
                    "location / {\n    try_files $uri $uri/ /index.html;\n}",
                )),
            },
        ],
    }
}
