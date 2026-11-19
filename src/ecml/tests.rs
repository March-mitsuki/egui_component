use super::*;
use crate::theme::{Theme, UiTheme};
use egui::{Event, PointerButton, Pos2, RawInput};

#[derive(Default)]
struct State {
    name: String,
    enabled: bool,
    count: i32,
    writes: usize,
    saved: String,
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
fn frame(
    ctx: &egui::Context,
    view: &mut View<State>,
    state: &mut State,
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
