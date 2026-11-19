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
    /// While editing, the title being typed; `None` when not editing.
    editing: Option<String>,
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
            editing: None,
        });
    }
    /// Starts editing one todo; any other edit in progress is saved first.
    fn edit(&mut self, id: u64) {
        let others: Vec<_> = self
            .todos
            .iter()
            .filter(|t| t.id != id && t.editing.is_some())
            .map(|t| t.id)
            .collect();
        others.into_iter().for_each(|other| self.save(other));
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.editing = Some(todo.title.clone());
        }
    }
    /// Saves the trimmed title; an empty title deletes the todo, as in TodoMVC.
    fn save(&mut self, id: u64) {
        let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) else {
            return;
        };
        let Some(title) = todo.editing.take() else {
            return;
        };
        match title.trim() {
            "" => self.todos.retain(|t| t.id != id),
            title => todo.title = title.to_owned(),
        }
    }
}

fn builder() -> ecml::Builder<State> {
    ecml::View::<State>::builder()
        .model("draft", |s| &s.draft, |s, value| s.draft = value)
        .bind("can_add", |s| !s.draft.trim().is_empty())
        .bind("has_todos", |s| !s.todos.is_empty())
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
        // Enter in the entry field and the Add button both add the draft.
        .on("add", |s| {
            let title = std::mem::take(&mut s.draft);
            if !title.trim().is_empty() {
                s.add(title.trim());
            }
        })
        .on("complete_all", |s| {
            s.todos.iter_mut().for_each(|t| t.done = true)
        })
        .on("clear_completed", |s| s.todos.retain(|t| !t.done))
        .on("show_all", |s| s.filter = Filter::All)
        .on("show_active", |s| s.filter = Filter::Active)
        .on("show_completed", |s| s.filter = Filter::Completed)
        // Rust filters the rows; the template sees only these row registrations.
        // Rows are found by id, never by position.
        .list(
            "todos",
            |s| Box::new(s.todos.iter().filter(move |t| s.filter.shows(t))),
            |s, id| s.todos.iter_mut().find(|t| t.id == *id),
            |t| t.id,
            |row| {
                row.model("done", |t| &t.done, |t, value| t.done = value)
                    .bind("title", |_, t| t.title.clone())
                    .bind("editing", |_, t| t.editing.is_some())
                    .model(
                        "draft",
                        |t| t.editing.as_ref().unwrap_or(&t.title),
                        |t, value| t.editing = Some(value),
                    )
                    .on("edit", |s, id| s.edit(*id))
                    .on("save", |s, id| s.save(*id))
                    .on("cancel", |s, id| {
                        if let Some(t) = s.todos.iter_mut().find(|t| t.id == *id) {
                            t.editing = None;
                        }
                    })
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
        // With no todos the list and footer are hidden.
        assert!(view.response("all").is_none());
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
        assert!(view.response("all").is_some());
        // "Clear completed" appears only once something is done.
        assert!(view.response("clear").is_none());

        state.todos[0].done = true;
        click(&ctx, &mut view, &mut state, "active");
        assert!(state.filter == Filter::Active);
        click(&ctx, &mut view, &mut state, "clear");
        let titles: Vec<_> = state.todos.iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, ["eggs"]);
        click(&ctx, &mut view, &mut state, "complete_all");
        assert!(state.todos.iter().all(|t| t.done));
        click(&ctx, &mut view, &mut state, "clear");
        assert!(state.todos.is_empty());
        assert!(view.response("all").is_none());
        assert!(view.last_error().is_none());
    }

    fn key(key: egui::Key) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }
    }
    fn titles(state: &State) -> Vec<&str> {
        state.todos.iter().map(|t| t.title.as_str()).collect()
    }

    #[test]
    fn enter_adds_and_edits_save_on_enter_or_blur_and_cancel_on_escape() {
        let ctx = egui::Context::default();
        let mut view = builder()
            .load(ecml::Source::inline(
                "todos.ecml",
                include_str!("todos.ecml"),
            ))
            .unwrap();
        let mut state = State::default();
        frame(&ctx, &mut view, &mut state, vec![]);
        // The entry field is focused on start and stays focused after Enter.
        assert!(view.response("draft").unwrap().has_focus());
        for title in ["milk", "eggs"] {
            let events = vec![Event::Text(title.into()), key(egui::Key::Enter)];
            frame(&ctx, &mut view, &mut state, events);
            assert!(view.response("draft").unwrap().has_focus());
        }
        assert_eq!(titles(&state), ["milk", "eggs"]);
        // Edit (what the Edit button runs): the edit field takes focus as it appears.
        state.edit(1);
        frame(&ctx, &mut view, &mut state, vec![]);
        assert!(!view.response("draft").unwrap().has_focus());
        let events = vec![Event::Text("!".into()), key(egui::Key::Enter)];
        frame(&ctx, &mut view, &mut state, events);
        assert_eq!(titles(&state), ["milk!", "eggs"]);
        assert!(state.todos[0].editing.is_none());
        // Escape cancels.
        state.edit(2);
        frame(&ctx, &mut view, &mut state, vec![]);
        frame(&ctx, &mut view, &mut state, vec![Event::Text("x".into())]);
        frame(&ctx, &mut view, &mut state, vec![key(egui::Key::Escape)]);
        assert_eq!(titles(&state), ["milk!", "eggs"]);
        assert!(state.todos[1].editing.is_none());
        // Leaving the field saves.
        state.edit(1);
        frame(&ctx, &mut view, &mut state, vec![]);
        frame(&ctx, &mut view, &mut state, vec![Event::Text("?".into())]);
        click(&ctx, &mut view, &mut state, "complete_all");
        assert_eq!(titles(&state), ["milk!?", "eggs"]);
        assert!(state.todos.iter().all(|t| t.done && t.editing.is_none()));
        // Saving an empty title deletes the todo.
        state.edit(2);
        state.todos[1].editing = Some("  ".into());
        frame(&ctx, &mut view, &mut state, vec![]);
        frame(&ctx, &mut view, &mut state, vec![key(egui::Key::Enter)]);
        assert_eq!(titles(&state), ["milk!?"]);
        assert!(view.last_error().is_none());
    }
}
