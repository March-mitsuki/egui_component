use egui_component::ecml;

/// Application state stays in Rust; templates only see what is registered.
#[derive(Default)]
pub struct State {
    pub name: String,
    pub count: i32,
    pub enabled: bool,
}

pub fn builder() -> ecml::Builder<State> {
    ecml::View::<State>::builder()
        // Readable and writable: e-model, interpolation, :disabled.
        .model("name", |s| &s.name, |s, value| s.name = value)
        .model("enabled", |s| &s.enabled, |s, value| s.enabled = value)
        // Read-only values and calculations.
        .bind("count", |s| s.count)
        .bind("can_increment", |s| s.enabled && s.count < 10)
        // Handlers for @click.
        .on("increment", |s| s.count += 1)
        .on("reset", |s| s.count = 0)
}
