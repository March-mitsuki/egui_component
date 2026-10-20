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
                title: "docs.installation:title",
                body: "docs.installation:body",
                code: Some((
                    "toml",
                    "[dependencies]\negui = \"0.34\"\negui_component = \"0.1.0-alpha.2\"",
                )),
            },
            Section {
                id: "theme",
                title: "docs.theme:title",
                body: "docs.theme:body",
                code: Some((
                    "rs",
                    "use egui_component::theme::UiTheme;\n\nlet ui_theme = UiTheme::from_json_bytes(\n    include_bytes!(\"theme.json\"),\n)?;\nlet theme = &ui_theme.light;",
                )),
            },
            Section {
                id: "first-component",
                title: "docs.firstcomponent:title",
                body: "docs.firstcomponent:body",
                code: Some((
                    "rs",
                    "use egui_component::components::button;\n\nif button::render(\n    ui, \"Create project\",\n    button::Style::new_solid_md(theme),\n).clicked() {\n    // Handle the action.\n}",
                )),
            },
            Section {
                id: "next-steps",
                title: "docs.nextsteps:title",
                body: "docs.nextsteps:body",
                code: None,
            },
        ],
        Doc::Styling => &[
            Section {
                id: "presets",
                title: "docs.presets:title",
                body: "docs.presets:body",
                code: Some((
                    "rs",
                    "let solid = button::Style::new_solid_md(theme);\nlet outline = button::Style::new_outline_md(theme);\nlet quiet = button::Style::new_ghost_sm(theme);",
                )),
            },
            Section {
                id: "customize",
                title: "docs.customize:title",
                body: "docs.customize:body",
                code: Some((
                    "rs",
                    "let mut style = button::Style::new_solid_md(theme)\n    .height(40.0)\n    .width(Some(180.0));\nstyle.corner_radius = egui::CornerRadius::same(12);\nbutton::render(ui, \"Make it yours\", style);",
                )),
            },
            Section {
                id: "compose",
                title: "docs.compose:title",
                body: "docs.compose:body",
                code: Some((
                    "rs",
                    "use egui_component::components::{card, text};\n\ncard::render(ui, |ui| {\n    text::render(ui, \"Your workspace\",\n        text::Style::new_heading(theme));\n}, card::Style::new_md(theme));",
                )),
            },
        ],
        Doc::Theming => &[
            Section {
                id: "tokens",
                title: "docs.tokens:title",
                body: "docs.tokens:body",
                code: Some((
                    "rs",
                    "let foreground = theme.colors.fg.base;\nlet background = theme.colors.bg.base;\nlet accent = theme.colors.primary.l6;",
                )),
            },
            Section {
                id: "brand-colors",
                title: "docs.brandcolors:title",
                body: "docs.brandcolors:body",
                code: Some((
                    "rs",
                    "use egui::Color32;\n\nlet theme = &mut ui_theme.light;\ntheme.colors.primary.l6 = Color32::from_rgb(107, 101, 245);\ntheme.colors.bg.base = Color32::WHITE;\ntheme.colors.fg.base = Color32::from_rgb(32, 33, 36);\ntheme.colors.primary.foreground = Some(Color32::WHITE);",
                )),
            },
            Section {
                id: "color-mode",
                title: "docs.colormode:title",
                body: "docs.colormode:body",
                code: Some((
                    "rs",
                    "let theme = if dark_mode {\n    &ui_theme.dark\n} else {\n    &ui_theme.light\n};\ntext::render(ui, \"Hello, theme\", text::Style::new(theme));",
                )),
            },
            Section {
                id: "fonts",
                title: "docs.fonts:title",
                body: "docs.fonts:body",
                code: Some((
                    "rs",
                    "use egui_component::consts::FONT_MANAGER;\n\nFONT_MANAGER.load_from_bytes(\n    \"my-font\", include_bytes!(\"font.ttf\"), 0,\n)?;\n// Before rendering each frame:\nFONT_MANAGER.apply(ui.ctx());",
                )),
            },
        ],
        Doc::Routing => &[
            Section {
                id: "typed-routes",
                title: "docs.typedroutes:title",
                body: "docs.typedroutes:body",
                code: Some((
                    "toml",
                    "# Within this repository\n[dependencies]\negui_component_router = { path = \"router\" }",
                )),
            },
            Section {
                id: "parameters",
                title: "docs.parameters:title",
                body: "docs.parameters:body",
                code: Some((
                    "rs",
                    "use egui_component_router::{PathPattern, SearchParams};\n\nlet pattern = PathPattern::new(\"/components/:name\")?;\nlet path = pattern.build(&[(\"name\", \"button\")])?;\nlet mut search = SearchParams::new();\nsearch.set(\"section\", \"usage\");",
                )),
            },
            Section {
                id: "history",
                title: "docs.history:title",
                body: "docs.history:body",
                code: Some((
                    "rs",
                    "use egui_component_router::HistoryMode;\n\nlet mut search = router.search_params();\nsearch.set(\"section\", \"usage\");\nrouter.set_search_params(search, HistoryMode::Replace)?;",
                )),
            },
            Section {
                id: "deployment",
                title: "docs.deployment:title",
                body: "docs.deployment:body",
                code: Some((
                    "nginx",
                    "location / {\n    try_files $uri $uri/ /index.html;\n}",
                )),
            },
        ],
    }
}
