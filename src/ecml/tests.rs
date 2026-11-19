use super::*;
use crate::theme::{Theme, UiTheme};
use egui::{Event, PointerButton, Pos2, RawInput};
use std::collections::HashMap;

#[derive(Default)]
struct State {
    name: String,
    enabled: bool,
    count: i32,
    writes: usize,
    saved: String,
    blurs: usize,
}
fn builder() -> Builder<State> {
    View::<State>::builder()
        .model(
            "name",
            |s| &s.name,
            |s, v| {
                s.name = v;
                s.writes += 1;
            },
        )
        .model(
            "enabled",
            |s| &s.enabled,
            |s, v| {
                s.enabled = v;
                s.writes += 1;
            },
        )
        .bind("count", |s| s.count)
        .bind("disabled", |s| !s.enabled)
        .on("increment", |s| s.count += 1)
        .on("save", |s| s.saved = s.name.clone())
        .on("blur", |s| s.blurs += 1)
}
fn view(template: &str) -> View<State> {
    builder()
        .load(Source::inline("test.ecml", template))
        .unwrap()
}
fn theme() -> Theme {
    UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
        .unwrap()
        .light
}
fn frame<S: 'static>(
    ctx: &egui::Context,
    view: &mut View<S>,
    state: &mut S,
    events: Vec<Event>,
    multi: bool,
) {
    let theme = theme();
    let output = ctx.run_ui(
        RawInput {
            events,
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(600.0, 500.0),
            )),
            ..Default::default()
        },
        |ui| {
            view.show(ui, &theme, state);
            if multi && ui.ctx().current_pass_index() == 0 {
                ui.ctx().request_discard("ECML multipass test");
            }
        },
    );
    if multi {
        assert_eq!(output.platform_output.num_completed_passes, 2);
    }
    output.drop_without_applying_deltas();
}
fn click(ctx: &egui::Context, view: &mut View<State>, state: &mut State, id: &str) {
    let pos = view.response(id).unwrap().rect.center();
    click_at(ctx, view, state, pos);
}
fn click_at<S: 'static>(ctx: &egui::Context, view: &mut View<S>, state: &mut S, pos: Pos2) {
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
            true,
        );
    }
}

#[test]
fn strict_schema_and_types_have_source_locations() {
    for (source, message) in [
        ("<input />", "unknown component 'input'"),
        ("<text_edit e-model=\"enabled\" />", "expected String"),
        ("<switch e-model=\"disabled\" />", "read-only"),
        ("<switch />", "unknown binding"),
        ("<text nope=\"1\" />", "unknown attribute"),
        ("<layout item_spacing.x=\"NaN\" />", "finite nonnegative"),
        ("<text size=\"giant\" />", "must be one of"),
        ("<button @click=\"missing\" />", "unknown handler"),
        ("<button :disabled=\"count\" />", "expected Bool"),
        ("<text>{{missing}}</text>", "unknown binding"),
        (
            "<layout><text id=\"a\"/><text id=\"a\"/></layout>",
            "unique",
        ),
        ("<text><button /></text>", "only layout"),
        ("<layout>lost text</layout>", "text content"),
        ("<text id=\"a\" id=\"b\" />", "duplicate attribute"),
        ("<text>{{ name </text>", "unclosed"),
        ("<text></button>", "expected </text>"),
        ("<text/><text/>", "one root"),
    ] {
        let error = builder()
            .load(Source::inline("bad.ecml", format!("\n{source}")))
            .err()
            .expect(source);
        assert!(error.message.contains(message), "{source}: {error}");
        assert_eq!(error.file, "bad.ecml");
        assert_eq!(error.line, 2);
    }
    assert!(
        builder()
            .bind("name", |_| String::new())
            .load(Source::inline("x", "<text/>"))
            .err()
            .unwrap()
            .message
            .contains("duplicate binding")
    );
}

#[test]
fn clicks_and_model_edits_submit_once_across_passes() {
    let ctx = egui::Context::default();
    let mut view = view(
        "<layout><text_edit id=\"name\" e-model=\"name\" width=\"200\"/><switch id=\"toggle\" e-model=\"enabled\"/><button id=\"click\" @click=\"increment\">Add</button><button id=\"save\" @click=\"save\">Save</button></layout>",
    );
    let mut state = State::default();
    for _ in 0..2 {
        frame(&ctx, &mut view, &mut state, vec![], true);
    }
    click(&ctx, &mut view, &mut state, "click");
    assert_eq!(state.count, 1);
    click(&ctx, &mut view, &mut state, "toggle");
    assert!(state.enabled);
    assert_eq!(state.writes, 1);
    view.response("name").unwrap().request_focus();
    frame(
        &ctx,
        &mut view,
        &mut state,
        vec![Event::Text("abc".into())],
        true,
    );
    assert_eq!(state.name, "abc");
    assert_eq!(state.writes, 2);
    click(&ctx, &mut view, &mut state, "save");
    assert_eq!(state.saved, "abc");
}

#[test]
fn explicit_identity_keeps_focus_cursor_and_instance_isolation() {
    let ctx = egui::Context::default();
    let initial = "<layout><text_edit id=\"name\" e-model=\"name\" width=\"200\"/></layout>";
    let mut a = view(initial);
    let mut state = State::default();
    frame(&ctx, &mut a, &mut state, vec![], false);
    let id = a.response("name").unwrap().id;
    a.response("name").unwrap().request_focus();
    frame(
        &ctx,
        &mut a,
        &mut state,
        vec![Event::Text("abc".into())],
        true,
    );
    let cursor = egui::text_edit::TextEditState::load(&ctx, id)
        .unwrap()
        .cursor
        .char_range();
    a.replace("<layout><text>Inserted before</text><layout direction=\"left_to_right\"><text_edit id=\"name\" e-model=\"name\" width=\"200\"/></layout></layout>");
    frame(&ctx, &mut a, &mut state, vec![], true);
    assert_eq!(a.response("name").unwrap().id, id);
    assert!(a.response("name").unwrap().has_focus());
    assert_eq!(
        egui::text_edit::TextEditState::load(&ctx, id)
            .unwrap()
            .cursor
            .char_range(),
        cursor
    );
    frame(
        &ctx,
        &mut a,
        &mut state,
        vec![Event::Text("d".into())],
        true,
    );
    assert_eq!(state.name, "abcd");
    let mut b = view(initial);
    let mut other = State::default();
    ctx.run_ui(Default::default(), |ui| {
        a.show(ui, &theme(), &mut state);
        b.show(ui, &theme(), &mut other);
    })
    .drop_without_applying_deltas();
    assert_ne!(
        a.response("name").unwrap().id,
        b.response("name").unwrap().id
    );
    assert!(!b.response("name").unwrap().has_focus());
    assert_eq!(other.name, "");
}

#[test]
fn replacement_is_atomic_at_frame_boundary_and_errors_recover() {
    let ctx = egui::Context::default();
    let mut v = view("<button id=\"old\" @click=\"increment\">Old</button>");
    let mut state = State {
        count: 7,
        ..Default::default()
    };
    let theme = theme();
    ctx.run_ui(Default::default(), |ui| {
        v.show(ui, &theme, &mut state);
        assert!(v.response("old").is_some());
        if ui.ctx().current_pass_index() == 0 {
            v.replace("<button id=\"new\" @click=\"increment\">New</button>");
            ui.ctx().request_discard("replacement test");
        }
    })
    .drop_without_applying_deltas();
    frame(&ctx, &mut v, &mut state, vec![], false);
    assert!(v.response("new").is_some());
    v.replace("<input />");
    frame(&ctx, &mut v, &mut state, vec![], false);
    assert!(v.last_error().is_some());
    assert!(v.response("new").is_some());
    assert_eq!(state.count, 7);
    v.replace("<button id=\"fixed\" @click=\"increment\">Fixed</button>");
    frame(&ctx, &mut v, &mut state, vec![], false);
    assert!(v.last_error().is_none());
    assert!(v.response("fixed").is_some());
    assert_eq!(state.count, 7);
}

#[test]
fn boolean_disable_and_entities_work() {
    let ctx = egui::Context::default();
    let mut view = view(
        "<button id=\"click\" :disabled=\"!enabled\" @click=\"increment\">&lt;{{count}} &amp; test&gt;</button>",
    );
    let mut state = State::default();
    frame(&ctx, &mut view, &mut state, vec![], false);
    click(&ctx, &mut view, &mut state, "click");
    assert_eq!(state.count, 0);
    state.enabled = true;
    frame(&ctx, &mut view, &mut state, vec![], false);
    click(&ctx, &mut view, &mut state, "click");
    assert_eq!(state.count, 1);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn watcher_wakes_idle_ui_handles_atomic_save_and_keeps_latest() {
    use std::{
        fs,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::{Duration, Instant},
    };
    let dir = std::env::temp_dir().join(format!(
        "ecml-test-{}-{}",
        std::process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("view.ecml");
    fs::write(&path, "<text>old</text>").unwrap();
    let mut view = builder().load(Source::file(&path)).unwrap();
    let ctx = egui::Context::default();
    let mut state = State::default();
    let wakes = Arc::new(AtomicUsize::new(0));
    let counter = wakes.clone();
    ctx.set_request_repaint_callback(move |_| {
        counter.fetch_add(1, Ordering::Relaxed);
    });
    for _ in 0..4 {
        frame(&ctx, &mut view, &mut state, vec![], false);
    }
    let before = wakes.load(Ordering::Relaxed);
    fs::write(
        dir.join("save.tmp"),
        "<text_edit id=\"latest\" e-model=\"name\" />",
    )
    .unwrap();
    fs::rename(dir.join("save.tmp"), &path).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while wakes.load(Ordering::Relaxed) == before && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(
        wakes.load(Ordering::Relaxed) > before,
        "watcher must wake without show polling"
    );
    frame(&ctx, &mut view, &mut state, vec![], false);
    assert!(view.response("latest").is_some());
    fs::write(&path, "<input/>").unwrap();
    std::thread::sleep(Duration::from_millis(350));
    frame(&ctx, &mut view, &mut state, vec![], false);
    assert!(view.last_error().is_some());
    assert!(view.response("latest").is_some());
    for n in 0..8 {
        fs::write(&path, format!("<button id=\"v{n}\">new</button>")).unwrap();
    }
    std::thread::sleep(Duration::from_millis(350));
    frame(&ctx, &mut view, &mut state, vec![], false);
    assert!(view.last_error().is_none());
    assert!(view.response("v7").is_some());
    drop(view);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn counter_example_buttons_fit_the_demo_window() {
    let ctx = egui::Context::default();
    let mut view = builder()
        .bind("can_increment", |s| s.enabled && s.count < 10)
        .on("reset", |s| s.count = 0)
        .load(Source::inline(
            "counter.ecml",
            include_str!("../../examples/ecml_hello_world/counter.ecml"),
        ))
        .unwrap();
    let mut state = State {
        enabled: true,
        ..Default::default()
    };
    let theme = theme();
    for _ in 0..3 {
        ctx.run_ui(
            RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    egui::vec2(480.0, 360.0),
                )),
                ..Default::default()
            },
            |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    view.show(ui, &theme, &mut state);
                    let button = view.response("increment").unwrap();
                    assert!(
                        ui.clip_rect().contains_rect(button.rect),
                        "button {:?} outside viewport {:?}",
                        button.rect,
                        ui.clip_rect()
                    );
                    let toggle = view.response("enabled").unwrap();
                    let input = view.response("name").unwrap();
                    assert!(
                        toggle.rect.top() - input.rect.bottom() < 120.0,
                        "row consumed remaining window height"
                    );
                });
            },
        )
        .drop_without_applying_deltas();
    }
}

#[test]
fn dotted_attributes_are_strict_and_aliases_are_rejected() {
    let parsed = parser::parse(
        "<layout\n item_spacing.x = '8'\titem_spacing.y=\"12\"/>",
        "spacing.ecml",
    )
    .unwrap();
    assert_eq!(parsed.attrs["item_spacing.x"].value, "8");
    assert_eq!(parsed.attrs["item_spacing.y"].value, "12");
    for name in [".x", "item_spacing.", "item_spacing..x", "item_spacing.1"] {
        assert!(
            parser::parse(&format!("<layout {name}=\"8\"/>"), "bad.ecml").is_err(),
            "{name}"
        );
    }
    for attribute in [
        "gap=\"8\"",
        "gap_x=\"8\"",
        "item_spacing_x=\"8\"",
        "item_spacing.z=\"8\"",
        "align=\"min\"",
        "wrap=\"true\"",
        "main_justify=\"true\"",
        "width=\"100\"",
    ] {
        let error = builder()
            .load(Source::inline(
                "bad.ecml",
                format!("<layout\n  {attribute}/>"),
            ))
            .err()
            .unwrap();
        assert!(error.message.contains("unknown attribute"), "{error}");
        assert_eq!((error.line, error.column), (2, 3));
    }
    for tag in ["row", "column"] {
        assert!(
            builder()
                .load(Source::inline("bad.ecml", format!("<{tag}/>")))
                .is_err()
        );
    }
    for direction in ["horizontal", "vertical", "right_to_left", "bottom_up"] {
        assert!(
            builder()
                .load(Source::inline(
                    "bad.ecml",
                    format!("<layout direction=\"{direction}\"/>")
                ))
                .is_err()
        );
    }
    for value in ["-1", "NaN", "inf"] {
        assert!(
            builder()
                .load(Source::inline(
                    "bad.ecml",
                    format!("<layout item_spacing.y=\"{value}\"/>")
                ))
                .is_err()
        );
    }
    assert!(
        parser::parse(
            "<layout item_spacing.x='1' item_spacing.x='2'/>",
            "bad.ecml"
        )
        .is_err()
    );
    assert!(
        builder()
            .load(Source::inline("bad.ecml", "<button item_spacing.x='8'/>"))
            .is_err()
    );
}

#[test]
fn layout_spacing_inherits_per_axis_and_does_not_leak() {
    let ctx = egui::Context::default();
    let mut v = view(
        r#"<layout item_spacing.x="31" item_spacing.y="19">
        <layout direction="left_to_right" item_spacing.x="7">
            <button id="a">A</button><button id="b">B</button>
        </layout>
        <layout direction="left_to_right">
            <button id="c">C</button><button id="d">D</button>
        </layout>
    </layout>"#,
    );
    let mut defaults =
        view(r#"<layout><button id="a">A</button><button id="b">B</button></layout>"#);
    let mut horizontal = view(
        r#"<layout direction="left_to_right"><button id="a">A</button><button id="b">B</button></layout>"#,
    );
    let mut state = State::default();
    let theme = theme();
    ctx.run_ui(Default::default(), |ui| {
        ui.spacing_mut().item_spacing = egui::vec2(13.0, 17.0);
        v.show(ui, &theme, &mut state);
        let rect = |id| v.response(id).unwrap().rect;
        assert!((rect("b").left() - rect("a").right() - 7.0).abs() < 0.1);
        assert!((rect("d").left() - rect("c").right() - 31.0).abs() < 0.1);
        assert!((rect("c").top() - rect("a").bottom() - 19.0).abs() < 0.1);
        assert_eq!(ui.spacing().item_spacing, egui::vec2(13.0, 17.0));
        defaults.show(ui, &theme, &mut state);
        assert!(
            (defaults.response("b").unwrap().rect.top()
                - defaults.response("a").unwrap().rect.bottom()
                - 17.0)
                .abs()
                < 0.1
        );
        horizontal.show(ui, &theme, &mut state);
        assert!(
            (horizontal.response("b").unwrap().rect.left()
                - horizontal.response("a").unwrap().rect.right()
                - 13.0)
                .abs()
                < 0.1
        );
        assert_eq!(ui.spacing().item_spacing, egui::vec2(13.0, 17.0));
    })
    .drop_without_applying_deltas();
}

#[test]
fn error_window_can_be_disabled_while_last_error_remains() {
    let windows = |show_window: bool| {
        let ctx = egui::Context::default();
        let mut v = builder()
            .error_window(show_window)
            .load(Source::inline("test.ecml", "<text>Ok</text>"))
            .unwrap();
        let mut state = State::default();
        v.replace("<input />");
        frame(&ctx, &mut v, &mut state, vec![], false);
        assert!(v.last_error().is_some());
        ctx.memory(|m| {
            m.layer_ids()
                .filter(|layer| layer.order == egui::Order::Middle)
                .count()
        })
    };
    assert_eq!(windows(true), 1);
    assert_eq!(windows(false), 0);
}

// ---- e-for ----

#[derive(Clone)]
struct Todo {
    id: u32,
    title: String,
    done: bool,
}
#[derive(Default)]
struct Todos {
    todos: Vec<Todo>,
    hide_done: bool,
    selected: Option<u32>,
    draft: String,
    removed: Vec<u32>,
    /// `done` values seen by a row getter drawn after the row's checkbox.
    seen: std::cell::RefCell<Vec<bool>>,
}
fn todo(id: u32, title: &str) -> Todo {
    Todo {
        id,
        title: title.into(),
        done: false,
    }
}
fn todos() -> Builder<Todos> {
    View::<Todos>::builder()
        .model("draft", |s| &s.draft, |s, v| s.draft = v)
        .bind("count", |s| s.todos.len() as i64)
        .list(
            "todos",
            |s| Box::new(s.todos.iter().filter(move |t| !(s.hide_done && t.done))),
            |s, id| s.todos.iter_mut().find(|t| t.id == *id),
            |t| t.id,
            |row| {
                row.model("done", |t| &t.done, |t, v| t.done = v)
                    .model("title", |t| &t.title, |t, v| t.title = v)
                    .bind("label", |_, t| format!("#{} {}", t.id, t.title))
                    .bind("selected", |s, t| s.selected == Some(t.id))
                    .bind("probe", |s, t| {
                        s.seen.borrow_mut().push(t.done);
                        String::new()
                    })
                    .on("remove", |s, id| {
                        s.removed.push(*id);
                        s.todos.retain(|t| t.id != *id);
                    })
                    .on("select", |s, id| s.selected = Some(*id))
            },
        )
}
const ROWS: &str = r#"<layout>
    <layout e-for="todo in todos" id="row" direction="left_to_right">
        <checkbox id="done" e-model="todo.done" />
        <text>{{ todo.probe }}</text>
        <text_edit id="title" e-model="todo.title" width="120" />
        <button id="remove" :disabled="todo.selected" @click="todo.remove">Remove</button>
        <button id="select" @click="todo.select">{{ todo.label }}</button>
    </layout>
    <text_edit id="draft" e-model="draft" width="120" />
</layout>"#;
fn todo_view(template: &str) -> View<Todos> {
    todos()
        .load(Source::inline("todos.ecml", template))
        .unwrap()
}
fn row_click(ctx: &egui::Context, view: &mut View<Todos>, state: &mut Todos, id: &str, key: u32) {
    let pos = view.row_response(id, &key).unwrap().rect.center();
    click_at(ctx, view, state, pos);
}

#[test]
fn rows_follow_the_iterator_and_row_handlers_get_their_key_once() {
    let ctx = egui::Context::default();
    let mut view = todo_view(ROWS);
    let mut state = Todos {
        todos: vec![todo(1, "a"), todo(2, "b"), todo(3, "c")],
        ..Default::default()
    };
    frame(&ctx, &mut view, &mut state, vec![], false);
    for key in [1, 2, 3] {
        assert!(view.row_response("remove", &key).is_some());
    }
    // Loop controls are not exposed through the root lookup.
    assert!(view.response("remove").is_none());
    assert!(view.response("draft").is_some());
    row_click(&ctx, &mut view, &mut state, "remove", 2);
    assert_eq!(state.removed, [2]);
    assert_eq!(state.todos.iter().map(|t| t.id).collect::<Vec<_>>(), [1, 3]);
    frame(&ctx, &mut view, &mut state, vec![], false);
    assert!(view.row_response("remove", &2).is_none());
    // Row bindings may read the whole state.
    row_click(&ctx, &mut view, &mut state, "select", 3);
    assert_eq!(state.selected, Some(3));
    // `selected` disables removing row 3 only.
    frame(&ctx, &mut view, &mut state, vec![], false);
    row_click(&ctx, &mut view, &mut state, "remove", 3);
    assert_eq!(state.removed, [2]);
    // Rust filters the rows.
    state.todos[0].done = true;
    state.hide_done = true;
    frame(&ctx, &mut view, &mut state, vec![], false);
    assert!(view.row_response("remove", &1).is_none());
    assert!(view.row_response("remove", &3).is_some());
}

#[test]
fn loop_writes_commit_after_the_loop_and_rows_read_old_values_until_then() {
    let ctx = egui::Context::default();
    let mut view = todo_view(ROWS);
    let mut state = Todos {
        todos: vec![todo(1, "a"), todo(2, "b")],
        ..Default::default()
    };
    frame(&ctx, &mut view, &mut state, vec![], false);
    state.seen.borrow_mut().clear();
    row_click(&ctx, &mut view, &mut state, "done", 2);
    assert!(state.todos[1].done);
    assert!(!state.todos[0].done);
    // The press frame draws nothing new; on the release frame the first pass draws
    // row 2's probe after its checkbox but before the commit, so it still sees false.
    // The second pass then shows the committed value.
    let seen = state.seen.borrow().clone();
    assert_eq!(&seen[seen.len() - 4..], [false, false, false, true]);
    // A row text edit writes through the key lookup.
    view.row_response("title", &1).unwrap().request_focus();
    frame(
        &ctx,
        &mut view,
        &mut state,
        vec![Event::Text("!".into())],
        true,
    );
    assert_eq!(state.todos[0].title, "a!");
    assert_eq!(state.todos[1].title, "b");
}

#[test]
fn row_identity_follows_the_key_when_rows_move() {
    let ctx = egui::Context::default();
    let mut view = todo_view(ROWS);
    let mut state = Todos {
        todos: vec![todo(1, "a"), todo(2, "b")],
        ..Default::default()
    };
    frame(&ctx, &mut view, &mut state, vec![], false);
    let id = view.row_response("title", &2).unwrap().id;
    assert_ne!(id, view.row_response("title", &1).unwrap().id);
    view.row_response("title", &2).unwrap().request_focus();
    frame(
        &ctx,
        &mut view,
        &mut state,
        vec![Event::Text("x".into())],
        true,
    );
    // Insert before and reorder: row 2 keeps its id and focus.
    state.todos.insert(0, todo(9, "new"));
    state.todos.swap(1, 2);
    frame(&ctx, &mut view, &mut state, vec![], true);
    let moved = view.row_response("title", &2).unwrap();
    assert_eq!(moved.id, id);
    assert!(moved.has_focus());
    frame(
        &ctx,
        &mut view,
        &mut state,
        vec![Event::Text("y".into())],
        true,
    );
    assert_eq!(state.todos[1].title, "bxy");
    assert_eq!(state.todos[0].title, "new");
}

#[test]
fn duplicate_keys_render_the_first_row_and_report_until_fixed() {
    let ctx = egui::Context::default();
    let mut view = todos()
        .error_window(false)
        .load(Source::inline("todos.ecml", ROWS))
        .unwrap();
    let mut state = Todos {
        todos: vec![todo(1, "first"), todo(1, "second"), todo(2, "b")],
        ..Default::default()
    };
    frame(&ctx, &mut view, &mut state, vec![], false);
    let error = view.last_error().unwrap();
    assert!(
        error.message.contains("list 'todos' has duplicate key 1"),
        "{error}"
    );
    assert_eq!((error.file.as_str(), error.line), ("todos.ecml", 2));
    assert!(view.row_response("remove", &2).is_some());
    state.seen.borrow_mut().clear();
    frame(&ctx, &mut view, &mut state, vec![], false);
    assert_eq!(state.seen.borrow().len(), 2, "the duplicate row is skipped");
    state.todos.remove(1);
    frame(&ctx, &mut view, &mut state, vec![], false);
    assert!(view.last_error().is_none());
}

#[test]
fn root_writes_inside_a_loop_are_buffered_too() {
    let ctx = egui::Context::default();
    let mut view = todo_view(
        r#"<layout><layout e-for="t in todos"><text_edit id="d" e-model="draft" width="120" /><text>{{ draft }}</text></layout></layout>"#,
    );
    let mut state = Todos {
        todos: vec![todo(1, "a")],
        ..Default::default()
    };
    frame(&ctx, &mut view, &mut state, vec![], false);
    view.row_response("d", &1).unwrap().request_focus();
    frame(
        &ctx,
        &mut view,
        &mut state,
        vec![Event::Text("hi".into())],
        true,
    );
    assert_eq!(state.draft, "hi");
}

#[test]
fn checkbox_edits_a_root_bool_model() {
    let ctx = egui::Context::default();
    let mut v = view(r#"<checkbox id="c" e-model="enabled">Enabled {{ count }}</checkbox>"#);
    let mut state = State::default();
    frame(&ctx, &mut v, &mut state, vec![], false);
    click(&ctx, &mut v, &mut state, "c");
    assert!(state.enabled);
    assert_eq!(state.writes, 1);
}

#[test]
fn loop_errors_have_source_locations() {
    for (source, message) in [
        (r#"<text e-for="t in todos" />"#, "root element"),
        (
            r#"<layout><text e-for="t in missing" /></layout>"#,
            "unknown list 'missing'",
        ),
        (
            r#"<layout><text e-for="t of todos" /></layout>"#,
            "<alias> in <list>",
        ),
        (
            r#"<layout><text e-for="t in" /></layout>"#,
            "<alias> in <list>",
        ),
        (
            r#"<layout><text e-for="t.x in todos" /></layout>"#,
            "plain names",
        ),
        (
            r#"<layout><text e-for="draft in todos" /></layout>"#,
            "collides",
        ),
        (
            r#"<layout><text e-for="todos in todos" /></layout>"#,
            "collides",
        ),
        (
            r#"<layout><text>{{ t.label }}</text></layout>"#,
            "not a loop alias",
        ),
        (
            r#"<layout><text e-for="t in todos" /><text>{{ t.label }}</text></layout>"#,
            "not a loop alias",
        ),
        (
            r#"<layout><text e-for="t in todos">{{ t.nope }}</text></layout>"#,
            "unknown row binding 'nope'",
        ),
        (
            r#"<layout><button e-for="t in todos" @click="t.nope" /></layout>"#,
            "unknown handler 't.nope'",
        ),
        (
            r#"<layout><checkbox e-for="t in todos" e-model="t.selected" /></layout>"#,
            "read-only",
        ),
        (
            r#"<layout><checkbox e-for="t in todos" e-model="t.title" /></layout>"#,
            "expected Bool",
        ),
        (
            r#"<layout><text e-for="t in todos">{{ todos }}</text></layout>"#,
            "unknown binding 'todos'",
        ),
        (
            r#"<layout><layout e-for="t in todos"><text e-for="t in todos" /></layout></layout>"#,
            "outer alias",
        ),
        (
            r#"<layout><layout e-for="t in todos"><text e-for="u in t.missing" /></layout></layout>"#,
            "unknown row list 'missing' for 't'",
        ),
        (
            r#"<layout><text e-for="u in t.items" /></layout>"#,
            "not a loop alias",
        ),
        (
            r#"<layout><layout e-for="t in todos"><text id="a" /><text id="a" /></layout></layout>"#,
            "unique",
        ),
    ] {
        let error = todos()
            .load(Source::inline("bad.ecml", format!("\n{source}")))
            .err()
            .expect(source);
        assert!(error.message.contains(message), "{source}: {error}");
        assert_eq!(error.line, 2, "{source}");
    }
    for (builder, message) in [
        (todos().bind("todos", |_| 1), "collides"),
        (
            todos().list(
                "more",
                |s| Box::new(s.todos.iter()),
                |s, id| s.todos.iter_mut().find(|t| t.id == *id),
                |t| t.id,
                |row| row.bind("x", |_, _| 1).bind("x", |_, _| 2),
            ),
            "duplicate row binding 'x'",
        ),
    ] {
        let error = builder.load(Source::inline("x", "<text/>")).err().unwrap();
        assert!(error.message.contains(message), "{error}");
    }
}

// ---- nested e-for ----

#[derive(Clone)]
struct Item {
    id: u32,
    label: String,
    done: bool,
}
#[derive(Default)]
struct Groups {
    groups: Vec<(String, Vec<Item>)>,
    removed: Vec<(String, u32)>,
    /// `(group, done)` seen by an item getter drawn after the item's checkbox.
    seen: std::cell::RefCell<Vec<(String, bool)>>,
}
fn item(id: u32, label: &str) -> Item {
    Item {
        id,
        label: label.into(),
        done: false,
    }
}
fn groups() -> Builder<Groups> {
    View::<Groups>::builder().list(
        "groups",
        |s| Box::new(s.groups.iter()),
        |s, name| s.groups.iter_mut().find(|g| g.0 == *name),
        |g| g.0.clone(),
        |group| {
            group
                .bind("name", |_, g| g.0.clone())
                .bind("size", |_, g| g.1.len() as i64)
                .list(
                    "items",
                    |_, g| Box::new(g.1.iter()),
                    |g, id| g.1.iter_mut().find(|i| i.id == *id),
                    |i| i.id,
                    |item| {
                        item.model("done", |i| &i.done, |i, v| i.done = v)
                            .bind("label", |_, i| i.label.clone())
                            .bind("probe", |s, i| {
                                let group = s
                                    .groups
                                    .iter()
                                    .find(|g| g.1.iter().any(|j| j.id == i.id))
                                    .map_or(String::new(), |g| g.0.clone());
                                s.seen.borrow_mut().push((group, i.done));
                                String::new()
                            })
                            .on("remove", |s, (group, id)| {
                                s.removed.push((group.clone(), *id));
                                if let Some(g) = s.groups.iter_mut().find(|g| g.0 == *group) {
                                    g.1.retain(|i| i.id != *id);
                                }
                            })
                    },
                )
        },
    )
}
const GROUPS: &str = r#"<layout>
    <layout e-for="group in groups" id="group">
        <text>{{ group.name }} ({{ group.size }})</text>
        <layout e-for="item in group.items" id="item" direction="left_to_right">
            <checkbox id="done" e-model="item.done">{{ item.label }} in {{ group.name }}</checkbox>
            <text>{{ item.probe }}</text>
            <button id="remove" @click="item.remove">Remove</button>
        </layout>
    </layout>
</layout>"#;
fn group_view() -> View<Groups> {
    groups()
        .error_window(false)
        .load(Source::inline("groups.ecml", GROUPS))
        .unwrap()
}
fn group_click(
    ctx: &egui::Context,
    view: &mut View<Groups>,
    state: &mut Groups,
    id: &str,
    key: u32,
) {
    let pos = view.row_response(id, &key).unwrap().rect.center();
    click_at(ctx, view, state, pos);
}

#[test]
fn nested_loops_write_through_parents_and_handlers_get_the_key_path() {
    let ctx = egui::Context::default();
    let mut view = group_view();
    let mut state = Groups {
        groups: vec![
            ("fruit".into(), vec![item(1, "apple"), item(2, "pear")]),
            ("veg".into(), vec![item(3, "leek")]),
        ],
        ..Default::default()
    };
    frame(&ctx, &mut view, &mut state, vec![], false);
    for key in [1, 2, 3] {
        assert!(view.row_response("done", &key).is_some());
    }
    // Inner rows reach their own registrations and the outer alias.
    state.seen.borrow_mut().clear();
    group_click(&ctx, &mut view, &mut state, "done", 3);
    assert!(state.groups[1].1[0].done);
    assert!(!state.groups[0].1.iter().any(|i| i.done));
    // Buffered until the outermost loop ends: the release frame's first pass still
    // sees false for item 3, and its second pass sees the committed value.
    let seen = state.seen.borrow().clone();
    let leek: Vec<_> = seen
        .iter()
        .filter(|(g, _)| g == "veg")
        .map(|s| s.1)
        .collect();
    assert_eq!(&leek[leek.len() - 2..], [false, true]);
    group_click(&ctx, &mut view, &mut state, "remove", 2);
    assert_eq!(state.removed, [("fruit".to_owned(), 2)]);
    assert_eq!(state.groups[0].1.len(), 1);
    assert!(view.last_error().is_none());
}

#[test]
fn nested_identity_and_duplicates_are_per_parent() {
    let ctx = egui::Context::default();
    let mut view = group_view();
    let mut state = Groups {
        groups: vec![
            ("a".into(), vec![item(1, "x"), item(1, "dup")]),
            ("b".into(), vec![item(2, "y")]),
        ],
        ..Default::default()
    };
    frame(&ctx, &mut view, &mut state, vec![], false);
    let error = view.last_error().unwrap();
    assert!(
        error.message.contains("list 'items' has duplicate key 1"),
        "{error}"
    );
    assert_eq!(error.line, 4);
    // The same child key under another parent is not a duplicate.
    state.groups[0].1.pop();
    state.groups[1].1.push(item(1, "also x"));
    frame(&ctx, &mut view, &mut state, vec![], false);
    assert!(view.last_error().is_none());
    // Moving a group keeps its rows' identity.
    let before = view.row_response("remove", &2).unwrap().id;
    state.groups.swap(0, 1);
    frame(&ctx, &mut view, &mut state, vec![], false);
    assert_eq!(view.row_response("remove", &2).unwrap().id, before);
}

#[test]
fn three_levels_pass_the_full_key_path() {
    type Tree = (Vec<(u32, Vec<(u32, Vec<u32>)>)>, Vec<((u32, u32), u32)>);
    let mut view = View::<Tree>::builder()
        .list(
            "a",
            |s| Box::new(s.0.iter()),
            |s, k| s.0.iter_mut().find(|x| x.0 == *k),
            |x| x.0,
            |a| {
                a.list(
                    "b",
                    |_, x| Box::new(x.1.iter()),
                    |x, k| x.1.iter_mut().find(|y| y.0 == *k),
                    |y| y.0,
                    |b| {
                        b.list(
                            "c",
                            |_, y| Box::new(y.1.iter()),
                            |y, k| y.1.iter_mut().find(|z| **z == *k),
                            |z| *z,
                            |c| {
                                c.bind("value", |_, z| *z as i64)
                                    .on("pick", |s, path| s.1.push(*path))
                            },
                        )
                    },
                )
            },
        )
        .load(Source::inline(
            "tree.ecml",
            r#"<layout><layout e-for="x in a"><layout e-for="y in x.b"><button e-for="z in y.c" id="pick" @click="z.pick">{{ z.value }}</button></layout></layout></layout>"#,
        ))
        .unwrap();
    let ctx = egui::Context::default();
    let mut state: Tree = (
        vec![(1, vec![(10, vec![100, 101])]), (2, vec![(20, vec![200])])],
        vec![],
    );
    frame(&ctx, &mut view, &mut state, vec![], false);
    let pos = view.row_response("pick", &200u32).unwrap().rect.center();
    click_at(&ctx, &mut view, &mut state, pos);
    assert_eq!(state.1, [((2, 20), 200)]);
}

// ---- e-if and fragment ----

#[test]
fn conditional_chains_draw_the_first_true_branch() {
    let ctx = egui::Context::default();
    let mut v = view(
        r#"<layout>
    <switch id="toggle" e-model="enabled" />
    <!-- comments may sit inside a chain -->
    <button e-if="enabled" id="on">On</button>
    <!-- between branches -->
    <button e-else-if="!enabled" id="off">Off</button>
    <button e-else id="never">Never</button>
    <text id="after">after</text>
</layout>"#,
    );
    let mut state = State::default();
    frame(&ctx, &mut v, &mut state, vec![], false);
    assert!(v.response("on").is_none());
    assert!(v.response("off").is_some());
    assert!(v.response("never").is_none());
    click(&ctx, &mut v, &mut state, "toggle");
    assert!(state.enabled);
    assert!(v.response("on").is_some());
    assert!(v.response("off").is_none());
}

#[test]
fn hidden_branches_allocate_no_space() {
    let ctx = egui::Context::default();
    let mut v = view(
        r#"<layout><switch id="toggle" e-model="enabled" /><button e-if="enabled" id="b">B</button><text id="after">after</text></layout>"#,
    );
    let mut state = State::default();
    frame(&ctx, &mut v, &mut state, vec![], false);
    let gap = v.response("after").unwrap().rect.min.y - v.response("toggle").unwrap().rect.max.y;
    assert!(gap <= 10.0, "hidden button left a gap of {gap}");
    state.enabled = true;
    frame(&ctx, &mut v, &mut state, vec![], false);
    assert!(v.response("after").unwrap().rect.min.y >= v.response("b").unwrap().rect.max.y);
}

#[test]
fn a_false_root_condition_draws_nothing() {
    let ctx = egui::Context::default();
    let mut v = view(r#"<text e-if="enabled" id="t">shown</text>"#);
    let mut state = State::default();
    frame(&ctx, &mut v, &mut state, vec![], false);
    assert!(v.response("t").is_none());
    state.enabled = true;
    frame(&ctx, &mut v, &mut state, vec![], false);
    assert!(v.response("t").is_some());
}

#[test]
fn fragments_draw_children_into_the_parent_layout() {
    let ctx = egui::Context::default();
    let mut v = view(
        r#"<layout direction="left_to_right">
    <button id="a">A</button>
    <fragment e-if="enabled"><button id="b">B</button><button id="c">C</button></fragment>
</layout>"#,
    );
    let mut state = State {
        enabled: true,
        ..Default::default()
    };
    frame(&ctx, &mut v, &mut state, vec![], false);
    let [a, b, c] = ["a", "b", "c"].map(|id| v.response(id).unwrap().rect);
    assert_eq!((a.min.y, b.min.y, c.min.y), (a.min.y, a.min.y, a.min.y));
    assert!(a.max.x < b.min.x && b.max.x < c.min.x);
    // A root fragment draws several top-level elements into the host Ui.
    let mut v = view(r#"<fragment><text id="x">1</text><text id="y">2</text></fragment>"#);
    frame(&ctx, &mut v, &mut state, vec![], false);
    assert!(v.response("x").unwrap().rect.max.y <= v.response("y").unwrap().rect.min.y);
}

#[test]
fn conditions_and_fragments_work_inside_loops() {
    let ctx = egui::Context::default();
    let mut view = todo_view(
        r#"<layout direction="left_to_right">
    <fragment e-for="t in todos" id="rows">
        <button e-if="t.selected" id="sel">{{ t.label }}</button>
        <button e-else id="pick" @click="t.select">{{ t.label }}</button>
    </fragment>
</layout>"#,
    );
    let mut state = Todos {
        todos: vec![todo(1, "a"), todo(2, "b")],
        selected: Some(2),
        ..Default::default()
    };
    frame(&ctx, &mut view, &mut state, vec![], false);
    assert!(view.row_response("sel", &2).is_some());
    assert!(view.row_response("sel", &1).is_none());
    let pick = view.row_response("pick", &1).unwrap().rect;
    // Fragment rows stay in the parent's row.
    assert_eq!(pick.min.y, view.row_response("sel", &2).unwrap().rect.min.y);
    row_click(&ctx, &mut view, &mut state, "pick", 1);
    assert_eq!(state.selected, Some(1));
    assert!(view.row_response("sel", &1).is_some());
    assert!(view.row_response("pick", &2).is_some());
}

#[test]
fn conditional_errors_have_source_locations() {
    for (source, message) in [
        ("<text e-else />", "the root has none"),
        ("<text e-else-if=\"enabled\" />", "the root has none"),
        ("<layout><text e-else /></layout>", "must directly follow"),
        (
            "<layout><text e-if=\"enabled\" /><text /><text e-else /></layout>",
            "must directly follow",
        ),
        (
            "<layout><text e-if=\"enabled\" /><text e-else /><text e-else /></layout>",
            "must directly follow",
        ),
        (
            "<layout><text e-if=\"\" /></layout>",
            "needs a registered bool name",
        ),
        (
            "<layout><text e-if=\"!\" /></layout>",
            "needs a registered bool name",
        ),
        ("<layout><text e-if=\"count\" /></layout>", "expected Bool"),
        (
            "<layout><text e-if=\"nope\" /></layout>",
            "unknown binding 'nope'",
        ),
        (
            "<layout><text e-if=\"enabled\" e-else /></layout>",
            "only one of",
        ),
        (
            "<layout><text e-else=\"x\" /></layout>",
            "e-else takes no value",
        ),
        ("<layout><text disabled /></layout>", "expected '='"),
        (
            "<layout><text e-if=\"enabled\" e-for=\"t in todos\" /></layout>",
            "cannot be combined with e-for",
        ),
        ("<fragment size=\"sm\" />", "unknown attribute 'size'"),
        ("<fragment>loose text</fragment>", "text content"),
        (
            "<layout><text e-if=\"enabled\" id=\"a\" /><text e-else id=\"a\" /></layout>",
            "unique",
        ),
    ] {
        let error = builder()
            .load(Source::inline("bad.ecml", format!("\n{source}")))
            .err()
            .expect(source);
        assert!(error.message.contains(message), "{source}: {error}");
        assert_eq!(error.line, 2, "{source}");
    }
}

// ---- text_edit events and focus ----

fn key(key: egui::Key) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Default::default(),
    }
}
fn focused(v: &View<State>, id: &str) -> bool {
    v.response(id).is_some_and(|r| r.has_focus())
}

#[test]
fn each_focus_loss_fires_one_text_edit_handler() {
    let ctx = egui::Context::default();
    let mut v = view(
        r#"<layout><text_edit id="t" e-model="name" width="200" @submit="save" @cancel="increment" @blur="blur" /><button id="other">Other</button></layout>"#,
    );
    let mut state = State::default();
    frame(&ctx, &mut v, &mut state, vec![], false);
    v.response("t").unwrap().request_focus();
    // Text and Enter in one frame: the edit is written before the handler runs.
    frame(
        &ctx,
        &mut v,
        &mut state,
        vec![Event::Text("abc".into()), key(egui::Key::Enter)],
        true,
    );
    assert_eq!(state.saved, "abc");
    assert_eq!((state.count, state.blurs), (0, 0));
    // @submit keeps focus so the next entry can be typed straight away.
    frame(&ctx, &mut v, &mut state, vec![], false);
    assert!(focused(&v, "t"));
    frame(&ctx, &mut v, &mut state, vec![key(egui::Key::Escape)], true);
    assert_eq!((state.count, state.blurs), (1, 0));
    assert!(!focused(&v, "t"));
    v.response("t").unwrap().request_focus();
    frame(&ctx, &mut v, &mut state, vec![], false);
    click(&ctx, &mut v, &mut state, "other");
    assert_eq!((state.count, state.blurs), (1, 1));
    assert_eq!(state.saved, "abc", "submit ran once");
}

#[test]
fn focus_loss_without_a_specific_handler_fires_blur() {
    let ctx = egui::Context::default();
    let mut v = view(r#"<text_edit id="t" e-model="name" width="200" @blur="blur" />"#);
    let mut state = State::default();
    for k in [egui::Key::Enter, egui::Key::Escape] {
        frame(&ctx, &mut v, &mut state, vec![], false);
        v.response("t").unwrap().request_focus();
        frame(&ctx, &mut v, &mut state, vec![], false);
        frame(&ctx, &mut v, &mut state, vec![key(k)], true);
    }
    assert_eq!(state.blurs, 2);
    // Without @submit, Enter keeps egui's behavior and ends editing.
    frame(&ctx, &mut v, &mut state, vec![], false);
    assert!(!focused(&v, "t"));
}

#[test]
fn autofocus_focuses_a_control_when_it_appears() {
    let ctx = egui::Context::default();
    let mut v = view(
        r#"<layout><switch id="toggle" e-model="enabled" /><text_edit e-if="enabled" id="t" e-model="name" width="200" autofocus="true" /></layout>"#,
    );
    let mut state = State::default();
    frame(&ctx, &mut v, &mut state, vec![], false);
    click(&ctx, &mut v, &mut state, "toggle");
    frame(&ctx, &mut v, &mut state, vec![], false);
    assert!(focused(&v, "t"), "focused when its branch appears");
    // It is not focused again on every frame once the user leaves it.
    frame(&ctx, &mut v, &mut state, vec![key(egui::Key::Escape)], true);
    for _ in 0..2 {
        frame(&ctx, &mut v, &mut state, vec![], false);
    }
    assert!(!focused(&v, "t"));
    // Hiding and showing it again focuses it again.
    state.enabled = false;
    frame(&ctx, &mut v, &mut state, vec![], false);
    state.enabled = true;
    for _ in 0..2 {
        frame(&ctx, &mut v, &mut state, vec![], false);
    }
    assert!(focused(&v, "t"));
    // On the View's first show too.
    let mut v = view(r#"<text_edit id="t" e-model="name" autofocus="true" />"#);
    for _ in 0..2 {
        frame(&ctx, &mut v, &mut state, vec![], false);
    }
    assert!(focused(&v, "t"));
}

#[test]
fn row_text_edit_events_reach_row_handlers() {
    let ctx = egui::Context::default();
    let mut view = todo_view(
        r#"<layout><layout e-for="todo in todos" id="row"><text_edit id="title" e-model="todo.title" width="120" @submit="todo.select" @blur="todo.remove" /></layout></layout>"#,
    );
    let mut state = Todos {
        todos: vec![todo(1, "a"), todo(2, "b")],
        ..Default::default()
    };
    frame(&ctx, &mut view, &mut state, vec![], false);
    view.row_response("title", &2).unwrap().request_focus();
    frame(
        &ctx,
        &mut view,
        &mut state,
        vec![Event::Text("!".into()), key(egui::Key::Enter)],
        true,
    );
    assert_eq!(state.selected, Some(2));
    assert_eq!(state.todos[1].title, "b!");
    assert!(state.removed.is_empty());
}

#[test]
fn text_edit_event_errors_have_source_locations() {
    for (source, message) in [
        (
            r#"<text_edit e-model="name" @submit="missing" />"#,
            "unknown handler 'missing'",
        ),
        (
            r#"<text_edit e-model="name" @blur="t.save" />"#,
            "not a loop alias",
        ),
        (
            r#"<text_edit e-model="name" autofocus="yes" />"#,
            "must be one of",
        ),
        (
            r#"<switch e-model="enabled" @blur="save" />"#,
            "unknown attribute '@blur'",
        ),
        (
            r#"<button @submit="save">x</button>"#,
            "unknown attribute '@submit'",
        ),
    ] {
        let error = builder()
            .load(Source::inline("bad.ecml", format!("\n{source}")))
            .err()
            .expect(source);
        assert!(error.message.contains(message), "{source}: {error}");
        assert_eq!(error.line, 2, "{source}");
    }
}

#[test]
fn a_focus_loss_fires_once_with_single_pass_frames() {
    // egui's lost_focus() stays true for two frames; with one pass per frame (as on
    // the web) that must still fire a single handler.
    let ctx = egui::Context::default();
    let mut v = view(
        r#"<layout><button id="other">Other</button><text_edit id="t" e-model="name" width="200" @cancel="increment" @blur="blur" /></layout>"#,
    );
    let mut state = State::default();
    let focus = |v: &mut View<State>, state: &mut State| {
        v.response("t").unwrap().request_focus();
        frame(&ctx, v, state, vec![], false);
        frame(&ctx, v, state, vec![], false);
    };
    frame(&ctx, &mut v, &mut state, vec![], false);
    focus(&mut v, &mut state);
    frame(
        &ctx,
        &mut v,
        &mut state,
        vec![key(egui::Key::Escape)],
        false,
    );
    for _ in 0..3 {
        frame(&ctx, &mut v, &mut state, vec![], false);
    }
    assert_eq!((state.count, state.blurs), (1, 0));
    // Clicking a control drawn before the field: focus moves before the field draws.
    focus(&mut v, &mut state);
    let pos = v.response("other").unwrap().rect.center();
    for pressed in [true, false] {
        let events = vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed,
                modifiers: Default::default(),
            },
        ];
        frame(&ctx, &mut v, &mut state, events, false);
    }
    for _ in 0..3 {
        frame(&ctx, &mut v, &mut state, vec![], false);
    }
    assert_eq!((state.count, state.blurs), (1, 1));
}

/// What one drawn text looks like: its color and its underline and strikethrough.
#[derive(Debug, PartialEq)]
struct Drawn {
    color: egui::Color32,
    underline: egui::Stroke,
    strikethrough: egui::Stroke,
}
/// Draws one frame and returns the drawn text by content.
fn drawn(view: &mut View<State>, state: &mut State) -> HashMap<String, Drawn> {
    let ctx = egui::Context::default();
    let theme = theme();
    let mut output = ctx.run_ui(
        RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(600.0, 500.0),
            )),
            ..Default::default()
        },
        |ui| view.show(ui, &theme, state),
    );
    output.textures_delta.clear();
    let mut found = HashMap::new();
    for clipped in output.shapes {
        if let egui::Shape::Text(shape) = clipped.shape {
            let format = &shape.galley.job.sections[0].format;
            let color = if format.color == egui::Color32::PLACEHOLDER {
                shape.fallback_color
            } else {
                format.color
            };
            found.insert(
                shape.galley.text().to_owned(),
                Drawn {
                    color,
                    underline: format.underline,
                    strikethrough: format.strikethrough,
                },
            );
        }
    }
    found
}

#[test]
fn styling_attributes_take_constants_and_conditional_expressions() {
    let mut view = view(
        r#"<layout>
            <text :color="enabled ? $theme.colors.red.l6 : $theme.colors.gray.l6"
                  :strikethrough="enabled" underline="true">Task</text>
            <text color="$theme.colors.fg.base">Plain</text>
            <button :variant="!enabled ? 'ghost' : 'solid'" color="$theme.colors.red">Go</button>
            <button id="sized" :size="enabled ? 'lg' : disabled ? 'xs' : 'md'">Size</button>
        </layout>"#,
    );
    let t = theme();
    let line = |color| egui::Stroke::new(1.0, color);
    let mut state = State::default();
    let off = drawn(&mut view, &mut state);
    assert_eq!(
        off["Task"],
        Drawn {
            color: t.colors.gray.l6,
            underline: line(t.colors.gray.l6),
            strikethrough: egui::Stroke::NONE,
        }
    );
    assert_eq!(off["Plain"].color, t.colors.fg.base);
    // A ghost button's text uses the scale's normal color.
    assert_eq!(off["Go"].color, t.colors.red.l6);
    assert_eq!(
        view.response("sized").unwrap().rect.height(),
        t.button.xs.size
    );

    state.enabled = true;
    let on = drawn(&mut view, &mut state);
    assert_eq!(
        on["Task"],
        Drawn {
            color: t.colors.red.l6,
            underline: line(t.colors.red.l6),
            strikethrough: line(t.colors.red.l6),
        }
    );
    // A solid button's text uses the scale's foreground.
    assert_eq!(on["Go"].color, t.colors.red.foreground);
    assert_eq!(
        view.response("sized").unwrap().rect.height(),
        t.button.lg.size
    );
    assert!(view.last_error().is_none());
}

#[test]
fn styling_expression_errors_have_source_locations() {
    for (source, message) in [
        ("<text :color=\"gray\" />", "expected a quoted value"),
        ("<text color=\"gray.l6\" />", "is not a color"),
        (
            "<text color=\"$theme.colors.gray.l11\" />",
            "unknown theme color",
        ),
        (
            "<text color=\"$theme.colors.gray\" />",
            "unknown theme color",
        ),
        (
            "<text color=\"$theme.text_size.md\" />",
            "only $theme.colors",
        ),
        (
            "<text color=\"$theme.colors.fg.base\" :color=\"$theme.colors.fg.base\" />",
            "cannot combine",
        ),
        (
            "<button color=\"$theme.colors.gray.l6\" />",
            "unknown color scale",
        ),
        ("<button variant=\"$theme.colors.red\" />", "must be one of"),
        (
            "<button :variant=\"enabled ? 'big' : 'solid'\" />",
            "must be one of",
        ),
        (
            "<button :variant=\"count ? 'ghost' : 'solid'\" />",
            "expected Bool",
        ),
        (
            "<button :variant=\"nope ? 'ghost' : 'solid'\" />",
            "unknown binding 'nope'",
        ),
        ("<button :variant=\"enabled ? 'ghost'\" />", "expected ':'"),
        ("<button :variant=\"'ghost' : 'solid'\" />", "unexpected"),
        (
            "<button :variant=\"enabled ? ghost : 'solid'\" />",
            "expected a quoted value",
        ),
        (
            "<button :variant=\"? 'ghost' : 'solid'\" />",
            "needs a registered bool name",
        ),
        ("<text :strikethrough=\"count\" />", "expected Bool"),
        ("<text underline=\"yes\" />", "must be one of"),
    ] {
        let error = builder()
            .load(Source::inline("bad.ecml", format!("\n{source}")))
            .err()
            .expect(source);
        assert!(error.message.contains(message), "{source}: {error}");
        assert_eq!(error.line, 2, "{source}");
    }
}
