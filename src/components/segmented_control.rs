//! A compact choice group composed from themed buttons.
use crate::{components::button, theme::Theme};

#[derive(Clone)]
pub struct Style {
    pub selected: button::Style,
    pub unselected: button::Style,
    pub disabled: bool,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            selected: button::Style::new_surface_sm_color(theme, theme.colors.primary),
            unselected: button::Style::new_ghost_sm(theme),
            disabled: false,
        }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
/// A single-select group, like a WAI-ARIA radio group: the group is one Tab
/// stop (the selected choice, or the first if none is selected), and the arrow
/// keys move the selection, wrapping at the ends. Returns a changed response
/// when a different choice is activated.
pub fn render<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    value: &mut T,
    choices: &[(T, &str)],
    style: Style,
) -> egui::Response {
    let mut changed = false;
    let tab_index = choices
        .iter()
        .position(|(choice, _)| *choice == *value)
        .unwrap_or(0);
    let group = ui
        .add_enabled_ui(!style.disabled, |ui| {
            ui.horizontal_wrapped(|ui| {
                // Keyed by the choice's own widget ID, so sibling groups cannot
                // take each other's request.
                let pending_key = egui::Id::unique("egui_component_segmented_pending_focus");
                let pending = ui.data(|d| d.get_temp::<egui::Id>(pending_key));
                let mut target = None;
                let mut focus_next = None;
                let mut ids = Vec::with_capacity(choices.len());
                for (index, (choice, label)) in choices.iter().enumerate() {
                    let selected = *value == *choice;
                    let mut button_style = if selected {
                        style.selected.clone()
                    } else {
                        style.unselected.clone()
                    };
                    if !ui.is_enabled() {
                        button_style.state = button::ButtonState::Disable;
                    }
                    let response =
                        button::render_inner(ui, label, button_style, index == tab_index);
                    response.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::Role::RadioButton,
                            ui.is_enabled(),
                            selected,
                            *label,
                        )
                    });
                    ids.push(response.id);
                    if pending == Some(response.id) {
                        response.request_focus();
                        ui.data_mut(|d| d.remove::<egui::Id>(pending_key));
                    }
                    if response.clicked() {
                        // Like clicking a radio button, a click also focuses the
                        // choice so the arrow keys work afterwards.
                        ui.data_mut(|d| d.insert_temp(pending_key, response.id));
                        ui.ctx().request_repaint();
                        if !selected {
                            target = Some(index);
                        }
                    }
                    if response.has_focus() {
                        let step = ui.input_mut(|i| {
                            let mut step = 0isize;
                            for (key, delta) in [
                                (egui::Key::ArrowRight, 1),
                                (egui::Key::ArrowDown, 1),
                                (egui::Key::ArrowLeft, -1),
                                (egui::Key::ArrowUp, -1),
                            ] {
                                if i.consume_key(egui::Modifiers::NONE, key) {
                                    step += delta;
                                }
                            }
                            step
                        });
                        if step != 0 {
                            let len = choices.len() as isize;
                            let next = (index as isize + step).rem_euclid(len) as usize;
                            target = Some(next);
                            // egui also moves focus with the arrow keys; cancel that move.
                            ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
                            focus_next = Some(next);
                        }
                    }
                }
                if let Some(next) = focus_next {
                    // The new choice becomes the Tab stop next pass; focus it then.
                    ui.data_mut(|d| d.insert_temp(pending_key, ids[next]));
                    ui.ctx().request_repaint();
                }
                if let Some(index) = target
                    && *value != choices[index].0
                {
                    *value = choices[index].0;
                    changed = true;
                }
            })
            .response
        })
        .inner;
    ui.ctx()
        .accesskit_node_builder(group.id, |node| node.set_role(egui::Role::RadioGroup));
    let mut response = group;
    if changed {
        response.mark_changed();
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::UiTheme;
    use egui::{Event, Key, Modifiers, RawInput};

    #[test]
    fn group_is_one_tab_stop_and_arrows_move_selection() {
        let ctx = egui::Context::default();
        let theme = UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
            .unwrap()
            .light;
        let key = |key, modifiers| Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        };
        let mut value = 1;
        // Returns which widget has focus: Some(choice index), or None for the
        // button after the group.
        let run = |events: Vec<Event>, value: &mut i32| {
            let mut after = egui::Id::NULL;
            ctx.run_ui(
                RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    ui.horizontal(|ui| {
                        render(
                            ui,
                            value,
                            &[(0, "A"), (1, "B"), (2, "C")],
                            Style::new(&theme),
                        );
                    });
                    // A disabled sibling must not take the group's focus requests.
                    render(
                        ui,
                        &mut value.clone(),
                        &[(0, "A"), (1, "B"), (2, "C")],
                        Style::new(&theme).disabled(true),
                    );
                    after = button::render(ui, "After", button::Style::new_ghost_sm(&theme)).id;
                },
            )
            .textures_delta
            .clear();
            let focused = ctx.memory(|m| m.focused());
            (focused, after)
        };
        let tab = |m| key(Key::Tab, m);
        run(vec![], &mut value);
        run(vec![tab(Modifiers::NONE)], &mut value);
        let (selected_b, after) = run(vec![], &mut value);
        assert!(
            selected_b.is_some() && selected_b != Some(after),
            "Tab lands on B"
        );
        run(vec![key(Key::ArrowRight, Modifiers::NONE)], &mut value);
        assert_eq!(value, 2, "ArrowRight selects C");
        let (focused_c, _) = run(vec![], &mut value);
        assert!(
            focused_c.is_some() && focused_c != selected_b,
            "focus follows selection"
        );
        run(vec![key(Key::ArrowRight, Modifiers::NONE)], &mut value);
        assert_eq!(value, 0, "wraps to A");
        run(vec![], &mut value);
        run(vec![tab(Modifiers::NONE)], &mut value);
        let (focused, after) = run(vec![], &mut value);
        assert_eq!(focused, Some(after), "Tab leaves the group in one step");
    }

    #[test]
    fn pointer_click_focuses_choice_so_arrows_work() {
        let ctx = egui::Context::default();
        let theme = UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
            .unwrap()
            .light;
        let choices = [(0, "Light"), (1, "Dark"), (2, "Compare")];
        let mut value = 0;
        let rect = std::cell::Cell::new(egui::Rect::NOTHING);
        let run = |events: Vec<Event>, value: &mut i32| {
            ctx.run_ui(
                RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    rect.set(render(ui, value, &choices, Style::new(&theme)).rect);
                    render(
                        ui,
                        &mut value.clone(),
                        &choices,
                        Style::new(&theme).disabled(true),
                    );
                },
            )
            .textures_delta
            .clear();
        };
        run(vec![], &mut value);
        let pos = egui::pos2(rect.get().left() + 8.0, rect.get().center().y);
        let button = |pressed| Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        // Browsers can deliver press and release in the same frame.
        for events in [
            vec![Event::PointerMoved(pos)],
            vec![button(true), button(false)],
            vec![],
        ] {
            run(events, &mut value);
        }
        let right = Event::Key {
            key: Key::ArrowRight,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        run(vec![right], &mut value);
        assert_eq!(value, 1);
    }
}
