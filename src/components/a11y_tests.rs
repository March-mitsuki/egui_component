//! Cross-component accessibility and focus checks (WAI-ARIA APG / WCAG).
use egui::accesskit::{Node, NodeId, Role, Toggled};
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

/// Like [`nodes`], keeping AccessKit node ids for relation checks.
fn tree(
    ctx: &egui::Context,
    events: Vec<Event>,
    run: impl FnMut(&mut egui::Ui),
) -> Vec<(NodeId, Node)> {
    let mut output = ctx.run_ui(
        RawInput {
            events,
            ..Default::default()
        },
        run,
    );
    output.textures_delta.clear();
    output
        .platform_output
        .accesskit_update
        .expect("accesskit enabled")
        .nodes
}

fn find(tree: &[(NodeId, Node)], f: impl Fn(&Node) -> bool) -> &(NodeId, Node) {
    tree.iter().find(|(_, n)| f(n)).expect("node exists")
}

fn text_node(tree: &[(NodeId, Node)], text: &str) -> NodeId {
    find(tree, |n| n.role() == Role::Label && n.value() == Some(text)).0
}

#[test]
fn field_label_names_its_control_and_form_errors_describe_it() {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let theme = theme();
    let mut email = String::new();
    let mut run = |ui: &mut egui::Ui| {
        crate::form::Form::new("signup").show_with_data(ui, &mut email, |form, ui, email| {
            form.field(
                ui,
                "email",
                email,
                |ui, value| {
                    super::field::render(
                        ui,
                        "Email",
                        |ui| {
                            super::text_edit::render(
                                ui,
                                value,
                                super::text_edit::Style::new_outline_md(&theme),
                            )
                        },
                        super::field::Style::new(&theme),
                    )
                    .inner
                },
                |value| {
                    if value.is_empty() {
                        Err("Email is required".into())
                    } else {
                        Ok(())
                    }
                },
            );
            form.request_submit();
        });
    };
    tree(&ctx, vec![], &mut run);
    let nodes = tree(&ctx, vec![], &mut run);
    let input = &find(&nodes, |n| n.role() == Role::TextInput).1;
    assert!(input.labelled_by().contains(&text_node(&nodes, "Email")));
    assert_eq!(input.invalid(), Some(egui::accesskit::Invalid::True));
    let error = input.described_by()[0];
    assert!(
        nodes
            .iter()
            .any(|(id, n)| *id == error && n.role() != Role::TextInput),
        "the error message is a separate node"
    );
}

#[test]
fn dialog_and_drawer_are_modal_dialogs_named_by_their_header() {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let theme = theme();
    let mut run = |ui: &mut egui::Ui| {
        super::dialog::render(
            ui.ctx(),
            |ui| super::dialog::render_header(ui, &theme, "Delete file", || {}, Default::default()),
            super::dialog::Style::new(&theme).title("delete"),
        );
    };
    tree(&ctx, vec![], &mut run);
    let nodes = tree(&ctx, vec![], &mut run);
    let dialog = &find(&nodes, |n| n.role() == Role::Dialog).1;
    assert!(dialog.is_modal());
    assert!(
        dialog
            .labelled_by()
            .contains(&text_node(&nodes, "Delete file"))
    );

    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let mut run = |ui: &mut egui::Ui| {
        super::drawer::render(
            ui.ctx(),
            "settings",
            true,
            |drawer| {
                drawer.header(|ui| {
                    super::drawer::render_header(ui, &theme, "Settings", Default::default());
                })
            },
            super::drawer::Style::new(&theme),
        );
    };
    tree(&ctx, vec![], &mut run);
    let nodes = tree(&ctx, vec![], &mut run);
    let drawer = &find(&nodes, |n| n.role() == Role::Dialog).1;
    assert!(drawer.is_modal());
    assert!(
        drawer
            .labelled_by()
            .contains(&text_node(&nodes, "Settings"))
    );
}

#[test]
fn tooltip_describes_its_target() {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let theme = theme();
    let mut run = |ui: &mut egui::Ui| {
        let target = button::render(ui, "Save", button::Style::new_solid_md(&theme));
        super::tooltip::render(
            &target,
            |ui| {
                ui.label("Saves the draft");
            },
            &super::tooltip::Style::new(&theme),
        );
    };
    tree(&ctx, vec![], &mut run);
    tree(&ctx, vec![key(Key::Tab)], &mut run);
    let nodes = tree(&ctx, vec![], &mut run);
    let target = &find(&nodes, |n| {
        n.role() == Role::Button && n.label() == Some("Save")
    })
    .1;
    let description = target.described_by();
    assert_eq!(description.len(), 1, "described by the open tooltip");
}

#[test]
fn disclosure_popup_and_value_states_are_exposed() {
    use super::{accordion, calendar, date_picker, slider};
    use egui::accesskit::{AriaCurrent, HasPopup};
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let theme = theme();
    let today = calendar::Date::from_calendar_date(2026, calendar::Month::November, 17).unwrap();
    let mut selection = calendar::Selection::Single(Some(today));
    let mut state = calendar::State::new(calendar::YearMonth::from_date(today));
    let mut picked = calendar::Selection::Single(None);
    let mut picker = date_picker::State::new(calendar::YearMonth::from_date(today));
    let mut value = 3.0;
    let nodes = nodes(&ctx, |ui| {
        accordion::render(
            ui,
            egui::Id::unique("faq"),
            "Shipping",
            None,
            None,
            |_| {},
            accordion::Style::new(&theme),
        );
        date_picker::render(
            ui,
            egui::Id::unique("due"),
            &mut picked,
            &mut picker,
            date_picker::Style::new(&theme),
            Default::default(),
        );
        calendar::render(
            ui,
            egui::Id::unique("cal"),
            &mut selection,
            &mut state,
            calendar::Style::new(&theme),
            calendar::Options {
                today: Some(today),
                ..Default::default()
            },
        );
        slider::render(
            ui,
            &mut value,
            0.0..=10.0,
            slider::Style::new(&theme).step(1.0),
        );
    });
    let accordion = nodes
        .iter()
        .find(|n| n.role() == Role::DisclosureTriangle)
        .unwrap();
    assert_eq!(accordion.is_expanded(), Some(false));
    let combo = nodes.iter().find(|n| n.role() == Role::ComboBox).unwrap();
    assert_eq!(combo.has_popup(), Some(HasPopup::Dialog));
    let day = nodes
        .iter()
        .find(|n| n.aria_current() == Some(AriaCurrent::Date))
        .expect("today is marked");
    assert_eq!(day.is_selected(), Some(true));
    let slider = nodes.iter().find(|n| n.role() == Role::Slider).unwrap();
    assert_eq!(
        (
            slider.min_numeric_value(),
            slider.max_numeric_value(),
            slider.numeric_value_step()
        ),
        (Some(0.0), Some(10.0), Some(1.0))
    );
}

#[test]
fn arrow_keys_on_a_slider_do_not_move_focus() {
    use super::slider;
    let ctx = egui::Context::default();
    let theme = theme();
    let (mut a, mut b) = (5.0, 5.0);
    let mut first = None;
    let mut run = |events: Vec<Event>, focus: bool| {
        let mut output = ctx.run_ui(
            RawInput {
                events,
                ..Default::default()
            },
            |ui| {
                ui.horizontal(|ui| {
                    let r = slider::render(
                        ui,
                        &mut a,
                        0.0..=10.0,
                        slider::Style::new(&theme).step(1.0),
                    );
                    slider::render(ui, &mut b, 0.0..=10.0, slider::Style::new(&theme).step(1.0));
                    if focus {
                        r.request_focus();
                    }
                    first = Some(r.id);
                });
            },
        );
        output.textures_delta.clear();
    };
    run(vec![], true);
    run(vec![], false);
    run(vec![key(Key::ArrowRight)], false);
    run(vec![key(Key::ArrowRight)], false);
    run(vec![], false);
    assert_eq!((a, b), (7.0, 5.0));
    assert_eq!(ctx.memory(|m| m.focused()), first);
}

#[test]
fn icon_aria_label_picks_button_image_or_decorative() {
    use super::icon;
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let theme = theme();
    let nodes = nodes(&ctx, |ui| {
        let style = || icon::Style::new(&theme);
        let named = |label| icon::Options {
            aria_label: Some(label),
        };
        icon::render(ui, ICONS.close.clone(), style(), named("Status"));
        let handle = style().sense(egui::Sense::drag());
        icon::render(ui, ICONS.close.clone(), handle, named("Reorder"));
        icon::render(ui, ICONS.close.clone(), style(), Default::default());
    });
    let role_of = |label| {
        nodes
            .iter()
            .find(|n| n.label() == Some(label))
            .map(|n| n.role())
    };
    assert_eq!(role_of("Status"), Some(Role::Image));
    assert_eq!(role_of("Reorder"), Some(Role::Button));
    assert!(
        !nodes
            .iter()
            .any(|n| n.role() == Role::Image && n.label().is_none()),
        "unlabeled hover-only icons stay out of the tree"
    );
}
