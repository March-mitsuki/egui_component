//! A TodoMVC-style application: the acceptance example for ECML lists (`e-for`).
use egui_component::{ecml, theme::UiTheme};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Filter {
    #[default]
    All,
    Active,
    Completed,
}
impl Filter {
    fn shows(self, todo: &Todo) -> bool {
        match self {
            Self::All => true,
            Self::Active => !todo.done,
            Self::Completed => todo.done,
        }
    }
}
struct Todo {
    id: u64,
    title: String,
    done: bool,
}
#[derive(Default)]
struct State {
    todos: Vec<Todo>,
    next_id: u64,
    draft: String,
    filter: Filter,
}
impl State {
    fn add(&mut self, title: &str) {
        self.next_id += 1;
        self.todos.push(Todo {
            id: self.next_id,
            title: title.into(),
            done: false,
        });
    }
}

fn builder() -> ecml::Builder<State> {
    ecml::View::<State>::builder()
        .model("draft", |s| &s.draft, |s, value| s.draft = value)
        .bind("can_add", |s| !s.draft.trim().is_empty())
        .bind("remaining", |s| {
            match s.todos.iter().filter(|t| !t.done).count() {
                1 => "1 item left".to_owned(),
                n => format!("{n} items left"),
            }
        })
        .bind("all_done", |s| s.todos.iter().all(|t| t.done))
        .bind("any_done", |s| s.todos.iter().any(|t| t.done))
        .bind("showing_all", |s| s.filter == Filter::All)
        .bind("showing_active", |s| s.filter == Filter::Active)
        .bind("showing_completed", |s| s.filter == Filter::Completed)
        .on("add", |s| {
            let title = std::mem::take(&mut s.draft);
            s.add(title.trim());
        })
        .on("complete_all", |s| {
            s.todos.iter_mut().for_each(|t| t.done = true)
        })
        .on("clear_completed", |s| s.todos.retain(|t| !t.done))
        .on("show_all", |s| s.filter = Filter::All)
        .on("show_active", |s| s.filter = Filter::Active)
        .on("show_completed", |s| s.filter = Filter::Completed)
        // Rust filters the rows; the template only sees `todo.done`, `todo.title`
        // and `todo.remove`. Rows are found by id, never by position.
        .list(
            "todos",
            |s| Box::new(s.todos.iter().filter(move |t| s.filter.shows(t))),
            |s, id| s.todos.iter_mut().find(|t| t.id == *id),
            |t| t.id,
            |row| {
                row.model("done", |t| &t.done, |t, value| t.done = value)
                    .model("title", |t| &t.title, |t, value| t.title = value)
                    .on("remove", |s, id| s.todos.retain(|t| t.id != *id))
            },
        )
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
            // ECML layouts do not scroll; the host decides where scrolling happens.
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.view.show(ui, &self.theme.dark, &mut self.state);
            });
        });
    }
}
fn main() -> eframe::Result {
    let view = builder()
        .load(ecml::source!("examples/ecml_todo_app/todos.ecml"))
        .expect("initial ECML template must be valid");
    let mut state = State::default();
    for title in ["Try ECML", "Edit todos.ecml while this runs", "Ship it"] {
        state.add(title);
    }
    state.todos[0].done = true;
    let app = App {
        state,
        view,
        theme: UiTheme::from_json_bytes(include_bytes!("../theme.json")).unwrap(),
    };
    eframe::run_native(
        "ECML todos — edit examples/ecml_todo_app/todos.ecml",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([520.0, 480.0]),
            ..Default::default()
        },
        Box::new(move |_| Ok(Box::new(app))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Event, PointerButton, Pos2, RawInput};

    fn frame(
        ctx: &egui::Context,
        view: &mut ecml::View<State>,
        state: &mut State,
        events: Vec<Event>,
    ) {
        let theme = UiTheme::from_json_bytes(include_bytes!("../theme.json")).unwrap();
        // A frame and its follow-up pass, as egui does after an interaction.
        for events in [events, vec![]] {
            ctx.run_ui(
                RawInput {
                    events,
                    screen_rect: Some(egui::Rect::from_min_size(
                        Pos2::ZERO,
                        egui::vec2(520.0, 480.0),
                    )),
                    ..Default::default()
                },
                |ui| view.show(ui, &theme.dark, state),
            )
            .drop_without_applying_deltas();
        }
    }
    fn click(ctx: &egui::Context, view: &mut ecml::View<State>, state: &mut State, id: &str) {
        let response = view.response(id).unwrap();
        assert!(
            ctx.content_rect().contains_rect(response.rect),
            "{id} is outside the window"
        );
        let pos = response.rect.center();
        for pressed in [true, false] {
            frame(
                ctx,
                view,
                state,
                vec![
                    Event::PointerMoved(pos),
                    Event::PointerButton {
                        pos,
                        button: PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ],
            );
        }
    }

    #[test]
    fn add_complete_filter_and_clear_todos() {
        let ctx = egui::Context::default();
        let mut view = builder()
            .load(ecml::Source::inline(
                "todos.ecml",
                include_str!("todos.ecml"),
            ))
            .unwrap();
        let mut state = State::default();
        frame(&ctx, &mut view, &mut state, vec![]);
        // An empty draft cannot be added.
        click(&ctx, &mut view, &mut state, "add");
        assert!(state.todos.is_empty());
        for title in ["milk", "eggs"] {
            view.response("draft").unwrap().request_focus();
            frame(&ctx, &mut view, &mut state, vec![Event::Text(title.into())]);
            click(&ctx, &mut view, &mut state, "add");
        }
        let titles: Vec<_> = state.todos.iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, ["milk", "eggs"]);
        assert!(state.draft.is_empty());

        state.todos[0].done = true;
        click(&ctx, &mut view, &mut state, "active");
        assert!(state.filter == Filter::Active);
        click(&ctx, &mut view, &mut state, "clear");
        let titles: Vec<_> = state.todos.iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, ["eggs"]);
        click(&ctx, &mut view, &mut state, "complete_all");
        assert!(state.todos.iter().all(|t| t.done));
        assert!(view.last_error().is_none());
    }
}
