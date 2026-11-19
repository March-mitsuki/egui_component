//! Live, editable ECML demos for the gallery's ECML documentation page.
//! Each template file is both the shown source and the rendered demo.
mod state;

use crate::{i18n::Locale, routes::EcmlTopic};
use eframe::egui;
use egui_component::{
    components::{button, card, text, text_edit},
    ecml,
    theme::Theme,
};

const STATE_SOURCE: &str = include_str!("state.rs");

/// The code shown on a topic page, as `(language, source)`.
pub fn example(topic: EcmlTopic) -> Option<(&'static str, &'static str)> {
    Some(match topic {
        EcmlTopic::Setup => (
            "rs",
            "// Cargo.toml: egui_component = { version = \"...\", features = [\"ecml\"] }\nuse egui_component::ecml;\n\nlet mut view = ecml::View::<State>::builder()\n    .model(\"name\", |s| &s.name, |s, value| s.name = value)\n    .on(\"save\", |s| s.save())\n    .load(ecml::source!(\"ui/form.ecml\"))?;\n\n// In your existing egui pass, with any Ui and Theme:\nview.show(ui, theme, &mut state);",
        ),
        EcmlTopic::Bindings => ("rs", STATE_SOURCE),
        EcmlTopic::Syntax => (
            "ecml",
            "<layout item_spacing.y=\"8\">\n    <!-- one root, quoted values, closed tags -->\n    <text size=\"heading\">Title</text>\n    <text_edit id=\"name\" e-model=\"name\" />\n    <button :disabled=\"!can_save\" @click=\"save\">Save &amp; close</button>\n</layout>",
        ),
        EcmlTopic::Sources => (
            "rs",
            "// Debug native: file + hot reload. Release and WASM: embedded.\nlet source = ecml::source!(\"ui/form.ecml\");\n// Explicit overrides:\nlet source = ecml::Source::inline(\"form.ecml\", text);\nlet source = ecml::Source::file(\"ui/form.ecml\"); // native only\n// Queue a replacement for the next frame:\nview.replace(new_text);",
        ),
        EcmlTopic::Diagnostics => (
            "rs",
            "let view = builder\n    .error_window(false) // show errors in your own UI\n    .load(source)?;\nif let Some(error) = view.last_error() {\n    // file:line:column: reason; the last valid tree stays visible\n}\nlet focused = view.response(\"name\").is_some_and(|r| r.has_focus());",
        ),
        EcmlTopic::Loops => (
            "rs",
            "// Rust decides which rows to show and in what order; rows are found by key.\n.list(\n    \"todos\",\n    |s| Box::new(s.todos.iter().filter(move |t| !(s.hide_done && t.done))),\n    |s, id| s.todos.iter_mut().find(|t| t.id == *id),\n    |t| t.id,\n    |row| {\n        row.model(\"done\", |t| &t.done, |t, value| t.done = value)\n            .model(\"title\", |t| &t.title, |t, value| t.title = value)\n            .on(\"remove\", |s, id| s.todos.retain(|t| t.id != *id))\n    },\n)",
        ),
        EcmlTopic::NestedLoops => (
            "rs",
            "// A row registers a child list: read gets the parent row, find searches it.\n.list(\"projects\", |s| Box::new(s.projects.iter()),\n      |s, name| s.projects.iter_mut().find(|p| p.name == *name), |p| p.name.clone(),\n      |project| project\n          .bind(\"name\", |_, p| p.name.clone())\n          .list(\"tasks\",\n                |_, p| Box::new(p.tasks.iter()),\n                |p, id| p.tasks.iter_mut().find(|t| t.id == *id),\n                |t| t.id,\n                |task| task\n                    .model(\"done\", |t| &t.done, |t, value| t.done = value)\n                    // Handlers receive the key path (project name, task id).\n                    .on(\"remove\", |s, (name, id)| { /* ... */ })))",
        ),
        EcmlTopic::Text
        | EcmlTopic::Button
        | EcmlTopic::TextEdit
        | EcmlTopic::Switch
        | EcmlTopic::Checkbox
        | EcmlTopic::Layout
        | EcmlTopic::Conditionals
        | EcmlTopic::Fragment => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Demo {
    Counter,
    Text,
    Button,
    TextEdit,
    Switch,
    Checkbox,
    Layout,
    Todos,
    Projects,
    Conditionals,
    Fragment,
}
impl Demo {
    pub const ALL: [Self; 11] = [
        Self::Counter,
        Self::Todos,
        Self::Projects,
        Self::Conditionals,
        Self::Text,
        Self::Button,
        Self::TextEdit,
        Self::Switch,
        Self::Checkbox,
        Self::Layout,
        Self::Fragment,
    ];
    /// The live demo on a topic page, if any.
    pub fn for_topic(topic: EcmlTopic) -> Option<Self> {
        Some(match topic {
            EcmlTopic::Bindings => Self::Counter,
            EcmlTopic::Text => Self::Text,
            EcmlTopic::Button => Self::Button,
            EcmlTopic::TextEdit => Self::TextEdit,
            EcmlTopic::Switch => Self::Switch,
            EcmlTopic::Checkbox => Self::Checkbox,
            EcmlTopic::Layout => Self::Layout,
            EcmlTopic::Loops => Self::Todos,
            EcmlTopic::NestedLoops => Self::Projects,
            EcmlTopic::Conditionals => Self::Conditionals,
            EcmlTopic::Fragment => Self::Fragment,
            EcmlTopic::Setup | EcmlTopic::Syntax | EcmlTopic::Sources | EcmlTopic::Diagnostics => {
                return None;
            }
        })
    }
    fn file(self) -> &'static str {
        match self {
            Self::Counter => "counter.ecml",
            Self::Text => "text.ecml",
            Self::Button => "button.ecml",
            Self::TextEdit => "text_edit.ecml",
            Self::Switch => "switch.ecml",
            Self::Checkbox => "checkbox.ecml",
            Self::Layout => "layout.ecml",
            Self::Todos => "todos.ecml",
            Self::Projects => "projects.ecml",
            Self::Conditionals => "conditionals.ecml",
            Self::Fragment => "fragment.ecml",
        }
    }
    fn template(self) -> &'static str {
        match self {
            Self::Counter => include_str!("templates/counter.ecml"),
            Self::Text => include_str!("templates/text.ecml"),
            Self::Button => include_str!("templates/button.ecml"),
            Self::TextEdit => include_str!("templates/text_edit.ecml"),
            Self::Switch => include_str!("templates/switch.ecml"),
            Self::Checkbox => include_str!("templates/checkbox.ecml"),
            Self::Layout => include_str!("templates/layout.ecml"),
            Self::Todos => include_str!("templates/todos.ecml"),
            Self::Projects => include_str!("templates/projects.ecml"),
            Self::Conditionals => include_str!("templates/conditionals.ecml"),
            Self::Fragment => include_str!("templates/fragment.ecml"),
        }
    }
    fn load(self) -> Result<ecml::View<state::State>, ecml::Error> {
        state::builder()
            // Errors are shown inline below the editor instead of in a floating window.
            .error_window(false)
            .load(ecml::Source::inline(self.file(), self.template()))
    }
}

struct Playground {
    demo: Demo,
    text: String,
    view: ecml::View<state::State>,
}

/// One shared state for every demo: editing the name in one demo updates the others.
pub struct EcmlDocs {
    state: state::State,
    playgrounds: Vec<Playground>,
}
impl EcmlDocs {
    pub fn new() -> Self {
        Self {
            state: state::State {
                enabled: true,
                todos: ["Read the ECML guide", "Try e-for", "Hide done rows"]
                    .into_iter()
                    .zip(1..)
                    .map(|(title, id)| state::Todo {
                        id,
                        title: title.into(),
                        done: id == 1,
                    })
                    .collect(),
                projects: [
                    ("Website", &["Draft copy", "Pick a font"][..]),
                    ("Release", &["Tag v1"][..]),
                ]
                .into_iter()
                .zip(0..)
                .map(|((name, tasks), p)| state::Project {
                    name: name.into(),
                    tasks: tasks
                        .iter()
                        .zip(0..)
                        .map(|(title, t)| state::Todo {
                            id: 100 + p * 10 + t,
                            title: (*title).into(),
                            done: false,
                        })
                        .collect(),
                })
                .collect(),
                next_id: 200,
                ..Default::default()
            },
            playgrounds: Demo::ALL
                .into_iter()
                .map(|demo| Playground {
                    demo,
                    text: demo.template().to_owned(),
                    view: demo.load().expect("gallery ECML demos are valid"),
                })
                .collect(),
        }
    }
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        locale: Locale,
        demo: Demo,
        surface: card::Style,
    ) {
        let Self { state, playgrounds } = self;
        let playground = playgrounds
            .iter_mut()
            .find(|p| p.demo == demo)
            .expect("every demo has a playground");
        ui.push_id(("ecml-playground", demo.file()), |ui| {
            card::render(
                ui,
                |ui| {
                    ui.set_min_width((ui.available_width() - 2.0).max(0.0));
                    ui.horizontal(|ui| {
                        label(ui, demo.file(), theme);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let edited = playground.text != demo.template();
                            if button::render(
                                ui,
                                locale.text("ecml:reset"),
                                button::Style::new_ghost_sm(theme).state(if edited {
                                    button::ButtonState::None
                                } else {
                                    button::ButtonState::Disable
                                }),
                            )
                            .clicked()
                            {
                                playground.text = demo.template().to_owned();
                                playground.view.replace(playground.text.clone());
                            }
                        });
                    });
                    ui.add_space(8.0);
                    let wide = ui.available_width() >= 720.0;
                    let editor = |ui: &mut egui::Ui, playground: &mut Playground| {
                        let rows = playground.text.lines().count().clamp(4, 18);
                        let response = text_edit::render_multiline(
                            ui,
                            &mut playground.text,
                            rows,
                            text_edit::Style::new_outline_sm(theme)
                                .text_size(13.0)
                                .max_rows(Some(18)),
                        );
                        if response.changed() {
                            playground.view.replace(playground.text.clone());
                        }
                    };
                    let preview = |ui: &mut egui::Ui,
                                   playground: &mut Playground,
                                   state: &mut state::State| {
                        label(ui, locale.text("ecml:preview"), theme);
                        ui.add_space(8.0);
                        playground.view.show(ui, theme, state);
                        if let Some(error) = playground.view.last_error() {
                            ui.add_space(12.0);
                            text::render(
                                ui,
                                &error.to_string(),
                                text::Style::new(theme)
                                    .size(13.0)
                                    .color(theme.colors.red.l6),
                            );
                            text::render(
                                ui,
                                locale.text("ecml:keptLastValid"),
                                text::Style::new(theme)
                                    .size(13.0)
                                    .color(theme.colors.gray.l6),
                            );
                        }
                    };
                    if wide {
                        ui.columns(2, |columns| {
                            editor(&mut columns[0], playground);
                            preview(&mut columns[1], playground, state);
                        });
                    } else {
                        editor(ui, playground);
                        ui.add_space(16.0);
                        preview(ui, playground, state);
                    }
                    ui.add_space(8.0);
                    text::render(
                        ui,
                        locale.text("ecml:editHint"),
                        text::Style::new(theme)
                            .size(13.0)
                            .color(theme.colors.gray.l6),
                    );
                },
                surface,
            );
        });
    }
}
fn label(ui: &mut egui::Ui, value: &str, theme: &Theme) {
    text::render(
        ui,
        value,
        text::Style::new_heading(theme)
            .size(11.0)
            .color(theme.colors.primary.l6),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_demo_template_loads_and_has_a_topic() {
        for demo in Demo::ALL {
            demo.load()
                .unwrap_or_else(|e| panic!("{} is invalid: {e}", demo.file()));
        }
        let shown: Vec<_> = EcmlTopic::ALL
            .iter()
            .filter_map(|topic| Demo::for_topic(*topic))
            .collect();
        assert_eq!(shown, Demo::ALL);
    }
}
