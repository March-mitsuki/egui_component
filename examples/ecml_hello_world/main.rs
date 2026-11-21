use egui_component::{ecml, theme::UiTheme};

#[derive(Default)]
struct State {
    name: String,
    count: i32,
    enabled: bool,
    dark: bool,
}
struct App {
    state: State,
    view: ecml::View<State>,
    theme: UiTheme,
}
impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // The template toggles `dark` in the state; the host picks the theme from it.
        // egui's own visuals (panel fill, scrollbars, selection) follow separately.
        let (theme, mut visuals) = if self.state.dark {
            (&self.theme.dark, egui::Visuals::dark())
        } else {
            (&self.theme.light, egui::Visuals::light())
        };
        visuals.panel_fill = theme.colors.bg.base;
        ui.ctx().set_visuals(visuals);
        egui::CentralPanel::default().show(ui, |ui| {
            self.view.show(ui, theme, &mut self.state);
        });
    }
}
fn main() -> eframe::Result {
    let view = ecml::View::<State>::builder()
        .model("name", |s| s.name.clone(), |s, value| s.name = value)
        .model("enabled", |s| s.enabled, |s, value| s.enabled = value)
        .bind("count", |s| s.count)
        .bind("can_increment", |s| s.enabled && s.count < 10)
        .bind("theme_label", |s| {
            if s.dark { "Light mode" } else { "Dark mode" }.to_owned()
        })
        .on("increment", |s| s.count += 1)
        .on("reset", |s| s.count = 0)
        .on("toggle_theme", |s| s.dark = !s.dark)
        .load(ecml::source!("examples/ecml_hello_world/counter.ecml"))
        .expect("initial ECML template must be valid");
    let app = App {
        state: State {
            enabled: true,
            dark: true,
            ..Default::default()
        },
        view,
        theme: UiTheme::from_json_bytes(include_bytes!("../theme.json")).unwrap(),
    };
    eframe::run_native(
        "ECML prototype — edit examples/ecml_hello_world/counter.ecml",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([480.0, 360.0]),
            ..Default::default()
        },
        Box::new(move |_| Ok(Box::new(app))),
    )
}
