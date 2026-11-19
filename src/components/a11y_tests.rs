//! Cross-component accessibility and focus checks (WAI-ARIA APG / WCAG).
use egui::accesskit::{Node, Role, Toggled};
use egui::{Event, Key, Modifiers, RawInput};

use super::{button, icon_button, menu_button, navigation, select, switch};
use crate::{icon::ICONS, theme::UiTheme};

fn theme() -> crate::theme::Theme {
    UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
        .unwrap()
        .light
}

fn key(key: Key) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

fn nodes(ctx: &egui::Context, run: impl FnMut(&mut egui::Ui)) -> Vec<Node> {
    let mut output = ctx.run_ui(RawInput::default(), run);
    output.textures_delta.clear();
    let update = output
        .platform_output
        .accesskit_update
        .expect("accesskit enabled");
    update.nodes.into_iter().map(|(_, node)| node).collect()
}

#[test]
fn controls_expose_roles_names_and_states() {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let theme = theme();
    let mut on = true;
    let nodes = nodes(&ctx, |ui| {
        button::render(ui, "Save", button::Style::new_solid_md(&theme));
        icon_button::render(
            ui,
            ICONS.close.clone(),
            "Close",
            icon_button::Style::new_ghost_sm(&theme),
        );
        switch::render(ui, &mut on, switch::Style::new(&theme));
        select::render(
            ui,
            egui::Id::unique("fruit"),
            "Apple",
            select::Style::new(&theme, None, select::SelectSize::Md),
            |_| {},
        );
    });
    let find = |role: Role| nodes.iter().find(|n| n.role() == role);
    let labeled = |label: &str| {
        nodes
            .iter()
            .find(|n| n.role() == Role::Button && n.label() == Some(label))
            .is_some()
    };
    assert!(labeled("Save"), "button text is its name");
    assert!(labeled("Close"), "icon-only button has an accessible name");
    assert_eq!(
        find(Role::Switch).and_then(Node::toggled),
        Some(Toggled::True)
    );
    let combo = find(Role::ComboBox).expect("select trigger is a combobox");
    assert_eq!(combo.value(), Some("Apple"));
    assert_eq!(combo.is_expanded(), Some(false));
}

#[test]
fn navigation_items_are_links_marked_current() {
    use egui::accesskit::AriaCurrent;
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let theme = theme();
    let nodes = nodes(&ctx, |ui| {
        navigation::group(ui, "Sections", |ui| {
            let style = navigation::Style::new(&theme).current_kind(navigation::Current::Location);
            navigation::render(ui, "Intro", true, style.clone());
            navigation::render(ui, "Usage", false, style);
        });
    });
    let link = |label: &str| {
        nodes
            .iter()
            .find(|n| n.role() == Role::Link && n.label() == Some(label))
            .expect("navigation item is a link")
    };
    assert_eq!(link("Intro").aria_current(), Some(AriaCurrent::Location));
    assert_eq!(link("Usage").aria_current(), None);
    assert_eq!(link("Intro").toggled(), None, "not a toggle button");
    assert!(
        nodes
            .iter()
            .any(|n| n.role() == Role::Navigation && n.label() == Some("Sections")),
        "group is a named navigation landmark"
    );
}

#[test]
fn loading_button_keeps_focus_and_ignores_activation() {
    let ctx = egui::Context::default();
    let theme = theme();
    let run = |events: Vec<Event>, state: button::ButtonState, focus: bool| {
        let mut out = None;
        ctx.run_ui(
            RawInput {
                events,
                ..Default::default()
            },
            |ui| {
                let r = button::render(
                    ui,
                    "Save",
                    button::Style::new_solid_md(&theme).state(state.clone()),
                );
                if focus {
                    r.request_focus();
                }
                out = Some((r.clicked(), r.has_focus()));
            },
        )
        .textures_delta
        .clear();
        out.unwrap()
    };
    run(vec![], button::ButtonState::None, true);
    assert_eq!(
        run(vec![], button::ButtonState::Loading, false),
        (false, true)
    );
    assert_eq!(
        run(vec![key(Key::Enter)], button::ButtonState::Loading, false),
        (false, true),
        "Enter on a loading button does nothing"
    );
    assert!(
        !run(vec![], button::ButtonState::Disable, false).1,
        "disabled buttons are not focusable"
    );
}

#[test]
fn select_popup_is_keyboard_operable() {
    let ctx = egui::Context::default();
    let theme = theme();
    let popup = egui::Id::unique("keyboard_select");
    let options = ["A", "B", "C"];
    let mut value = "B";
    // Returns (open, focused: "trigger" | option | "none").
    let run = |events: Vec<Event>, focus_trigger: bool, value: &mut &'static str| {
        let mut names = std::collections::HashMap::new();
        ctx.run_ui(
            RawInput {
                events,
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 600.0),
                )),
                ..Default::default()
            },
            |ui| {
                let style = select::Style::new(&theme, None, select::SelectSize::Md);
                let current = *value;
                let trigger = select::render(ui, popup, current, style.clone(), |ui| {
                    for option in options {
                        let r = select::render_item(ui, option, option == current, &style, &theme);
                        if r.clicked() {
                            *value = option;
                        }
                        names.insert(r.id, option);
                    }
                });
                if focus_trigger {
                    trigger.request_focus();
                }
                names.insert(trigger.id, "trigger");
            },
        )
        .textures_delta
        .clear();
        let focused = ctx.memory(|m| m.focused());
        (
            egui::Popup::is_id_open(&ctx, popup),
            focused.map_or("none", |id| names.get(&id).copied().unwrap_or("other")),
        )
    };
    let press = |k: Key, value: &mut &'static str| {
        run(vec![key(k)], false, value);
        run(vec![], false, value);
        run(vec![], false, value)
    };
    run(vec![], true, &mut value);
    run(vec![], false, &mut value);
    assert_eq!(
        press(Key::Enter, &mut value),
        (true, "B"),
        "opens on the selected option"
    );
    assert_eq!(press(Key::ArrowDown, &mut value), (true, "C"));
    assert_eq!(press(Key::ArrowDown, &mut value), (true, "A"), "wraps");
    assert_eq!(press(Key::End, &mut value), (true, "C"));
    assert_eq!(press(Key::Home, &mut value), (true, "A"));
    assert_eq!(
        press(Key::Enter, &mut value),
        (false, "trigger"),
        "selects and closes"
    );
    assert_eq!(value, "A");
    assert_eq!(
        press(Key::ArrowDown, &mut value),
        (true, "A"),
        "ArrowDown opens"
    );
    assert_eq!(press(Key::Escape, &mut value), (false, "trigger"));
    press(Key::Space, &mut value);
    assert_eq!(
        press(Key::Tab, &mut value),
        (false, "trigger"),
        "Tab closes to the trigger"
    );
}

#[test]
fn menu_button_popup_is_keyboard_operable() {
    let ctx = egui::Context::default();
    let theme = theme();
    let popup = egui::Id::unique("keyboard_menu");
    let mut activated = vec![];
    let run = |events: Vec<Event>, focus_trigger: bool, activated: &mut Vec<&'static str>| {
        let mut names = std::collections::HashMap::new();
        ctx.run_ui(
            RawInput {
                events,
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 600.0),
                )),
                ..Default::default()
            },
            |ui| {
                let mut trigger_id = egui::Id::NULL;
                menu_button::render(
                    ui,
                    popup,
                    |ui| {
                        let r =
                            button::render(ui, "Actions", button::Style::new_outline_md(&theme));
                        if focus_trigger {
                            r.request_focus();
                        }
                        trigger_id = r.id;
                        r
                    },
                    |ui| {
                        for item in ["New", "Archive"] {
                            let r = button::render(ui, item, button::Style::new_ghost_sm(&theme));
                            if r.clicked() {
                                activated.push(item);
                            }
                            names.insert(r.id, item);
                        }
                    },
                    menu_button::Style::new(&theme),
                );
                names.insert(trigger_id, "trigger");
            },
        )
        .textures_delta
        .clear();
        let focused = ctx.memory(|m| m.focused());
        (
            egui::Popup::is_id_open(&ctx, popup),
            focused.map_or("none", |id| names.get(&id).copied().unwrap_or("other")),
        )
    };
    let press = |k: Key, activated: &mut Vec<&'static str>| {
        run(vec![key(k)], false, activated);
        run(vec![], false, activated);
        run(vec![], false, activated)
    };
    run(vec![], true, &mut activated);
    run(vec![], false, &mut activated);
    assert_eq!(
        press(Key::Enter, &mut activated),
        (true, "New"),
        "focuses the first item"
    );
    assert_eq!(press(Key::ArrowDown, &mut activated), (true, "Archive"));
    assert_eq!(
        press(Key::Enter, &mut activated),
        (false, "trigger"),
        "activates and closes"
    );
    assert_eq!(activated, ["Archive"]);
}
