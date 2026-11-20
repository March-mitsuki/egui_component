use egui_component::{ecml, theme::UiTheme};

#[derive(Default)]
struct State {
    name: String,
    count: i32,
    enabled: bool,
}
struct App {
    state: State,
    view: ecml::View<State>,
    theme: UiTheme,
}
impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx().set_visuals(egui::Visuals::dark());
        egui::CentralPanel::default().show(ui, |ui| {
            self.view.show(ui, &self.theme.dark, &mut self.state);
        });
    }
}
fn main() -> eframe::Result {
    let view = ecml::View::<State>::builder()
        .model("name", |s| s.name.clone(), |s, value| s.name = value)
        .model("enabled", |s| s.enabled, |s, value| s.enabled = value)
        .bind("count", |s| s.count)
        .bind("can_increment", |s| s.enabled && s.count < 10)
        .on("increment", |s| s.count += 1)
        .on("reset", |s| s.count = 0)
        .load(ecml::source!("examples/ecml_hello_world/counter.ecml"))
        .expect("initial ECML template must be valid");
    let app = App {
        state: State {
            enabled: true,
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
