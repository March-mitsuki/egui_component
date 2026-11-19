use egui_component::ecml;

/// Application state stays in Rust; templates only see what is registered.
#[derive(Default)]
pub struct State {
    pub name: String,
    pub count: i32,
    pub enabled: bool,
    pub todos: Vec<Todo>,
    pub next_id: u32,
    pub new_title: String,
    pub hide_done: bool,
    pub projects: Vec<Project>,
    pub last_event: String,
    pub show_focus: bool,
}
pub struct Project {
    pub name: String,
    pub tasks: Vec<Todo>,
}
pub struct Todo {
    pub id: u32,
    pub title: String,
    pub done: bool,
}

pub fn builder() -> ecml::Builder<State> {
    ecml::View::<State>::builder()
        // Readable and writable: e-model, interpolation, :disabled.
        .model("name", |s| &s.name, |s, value| s.name = value)
        .model("enabled", |s| &s.enabled, |s, value| s.enabled = value)
        .model(
            "new_title",
            |s| &s.new_title,
            |s, value| s.new_title = value,
        )
        .model(
            "hide_done",
            |s| &s.hide_done,
            |s, value| s.hide_done = value,
        )
        .model(
            "show_focus",
            |s| &s.show_focus,
            |s, value| s.show_focus = value,
        )
        // Read-only values and calculations.
        .bind("count", |s| s.count)
        .bind("can_increment", |s| s.enabled && s.count < 10)
        .bind("has_count", |s| s.count > 0)
        .bind("at_limit", |s| s.count >= 10)
        .bind("last_event", |s| s.last_event.clone())
        .bind("can_add", |s| !s.new_title.trim().is_empty())
        .bind("remaining", |s| {
            s.todos.iter().filter(|t| !t.done).count() as i64
        })
        // Handlers for @click.
        .on("increment", |s| s.count += 1)
        .on("reset", |s| s.count = 0)
        // Text input events: one fires per focus loss.
        .on("submitted", |s| {
            s.last_event = format!("Submitted \"{}\"", s.name)
        })
        .on("cancelled", |s| {
            s.last_event = "Cancelled with Escape".into()
        })
        .on("blurred", |s| s.last_event = "Left the field".into())
        .on("add", |s| {
            let title = std::mem::take(&mut s.new_title).trim().to_owned();
            s.next_id += 1;
            s.todos.push(Todo {
                id: s.next_id,
                title,
                done: false,
            });
        })
        // A list for e-for: rows to show, lookup by key, the key, and row registrations.
        .list(
            "todos",
            |s| Box::new(s.todos.iter().filter(move |t| !(s.hide_done && t.done))),
            |s, id| s.todos.iter_mut().find(|t| t.id == *id),
            |t| t.id,
            |row| {
                row.model("done", |t| &t.done, |t, value| t.done = value)
                    .model("title", |t| &t.title, |t, value| t.title = value)
                    .on("remove", |s, id| s.todos.retain(|t| t.id != *id))
            },
        )
        // Nested lists: each project row registers a child list of its tasks.
        .list(
            "projects",
            |s| Box::new(s.projects.iter()),
            |s, name| s.projects.iter_mut().find(|p| p.name == *name),
            |p| p.name.clone(),
            |project| {
                project
                    .bind("name", |_, p| p.name.clone())
                    .bind("open", |_, p| {
                        p.tasks.iter().filter(|t| !t.done).count() as i64
                    })
                    .on("add", |s, name| {
                        s.next_id += 1;
                        let id = s.next_id;
                        if let Some(p) = s.projects.iter_mut().find(|p| p.name == *name) {
                            p.tasks.push(Todo {
                                id,
                                title: "New task".into(),
                                done: false,
                            });
                        }
                    })
                    .list(
                        "tasks",
                        |_, p| Box::new(p.tasks.iter()),
                        |p, id| p.tasks.iter_mut().find(|t| t.id == *id),
                        |t| t.id,
                        |task| {
                            task.model("done", |t| &t.done, |t, value| t.done = value)
                                .model("title", |t| &t.title, |t, value| t.title = value)
                                // Child rows receive the key path: (project name, task id).
                                .on("remove", |s, (name, id)| {
                                    if let Some(p) = s.projects.iter_mut().find(|p| p.name == *name)
                                    {
                                        p.tasks.retain(|t| t.id != *id);
                                    }
                                })
                        },
                    )
            },
        )
}
