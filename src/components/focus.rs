//! Shared keyboard focus indicators for themed controls.
use egui::{Color32, CornerRadius, Event, Id, Response, Stroke, StrokeKind, Ui};

pub(super) const WIDTH: f32 = 2.0;
const GAP: f32 = 1.0;

#[derive(Clone, Copy)]
pub(super) enum Placement {
    Outside,
    Inside,
}

pub(super) fn focused(response: &Response) -> bool {
    response.enabled() && response.sense.is_focusable() && response.has_focus()
}

pub(super) fn keyboard_focus(response: &Response) -> bool {
    let ctx = &response.ctx;
    let frame = ctx.cumulative_frame_nr();
    let id = Id::new("egui_component_focus_input").with(ctx.viewport_id());
    let previous = ctx.data(|data| data.get_temp::<(u64, bool)>(id));

    let keyboard = match previous {
        Some((previous_frame, keyboard)) if previous_frame == frame => keyboard,
        _ => {
            // Read raw events so a control consuming a key cannot hide keyboard navigation.
            let input_method = ctx.input(|input| {
                input.raw.events.iter().rev().find_map(|event| match event {
                    Event::Key { pressed: true, .. } => Some(true),
                    Event::PointerButton { pressed: true, .. } => Some(false),
                    _ => None,
                })
            });
            let keyboard = input_method
                .or(previous.map(|(_, keyboard)| keyboard))
                .unwrap_or(true);
            ctx.data_mut(|data| data.insert_temp(id, (frame, keyboard)));
            keyboard
        }
    };
    keyboard && focused(response)
}

pub(super) fn ring(
    ui: &Ui,
    response: &Response,
    rect: egui::Rect,
    radius: CornerRadius,
    color: Color32,
    placement: Placement,
) {
    if keyboard_focus(response) {
        draw_ring(ui, rect, radius, Stroke::new(WIDTH, color), placement);
    }
}

pub(super) fn draw_ring(
    ui: &Ui,
    rect: egui::Rect,
    radius: CornerRadius,
    stroke: Stroke,
    placement: Placement,
) {
    if matches!(placement, Placement::Outside)
        && ui
            .clip_rect()
            .contains_rect(rect.expand(GAP + stroke.width))
    {
        ui.painter().rect_stroke(
            rect.expand(GAP),
            radius + CornerRadius::same(GAP as u8),
            stroke,
            StrokeKind::Outside,
        );
    } else {
        ui.painter().rect_stroke(
            rect.shrink(GAP),
            radius - CornerRadius::same(GAP as u8),
            stroke,
            StrokeKind::Inside,
        );
    }
}

pub(super) fn circle(
    ui: &Ui,
    response: &Response,
    center: egui::Pos2,
    radius: f32,
    color: Color32,
) {
    if keyboard_focus(response) {
        let outer_radius = radius + GAP + WIDTH;
        let bounds = egui::Rect::from_center_size(center, egui::Vec2::splat(2.0 * outer_radius));
        let stroke_radius = if ui.clip_rect().contains_rect(bounds) {
            outer_radius - WIDTH / 2.0
        } else {
            (radius - GAP - WIDTH / 2.0).max(0.0)
        };
        ui.painter()
            .circle_stroke(center, stroke_radius, Stroke::new(WIDTH, color));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{components::button, theme::UiTheme};
    use egui::{Key, Modifiers, PointerButton, Pos2, RawInput, Shape};

    #[test]
    fn button_ring_follows_keyboard_focus_and_is_suppressed_by_pointer_input() {
        let ctx = egui::Context::default();
        let theme = UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
            .unwrap()
            .light;
        let color = Color32::from_rgb(1, 2, 3);
        let button_rect = std::cell::Cell::new(egui::Rect::NOTHING);
        let run = |events, state: button::ButtonState| {
            let output = ctx.run_ui(
                RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    let mut style = button::Style::new_outline_md(&theme).state(state.clone());
                    style.focus_color = color;
                    button_rect.set(button::render(ui, "Action", style).rect);
                },
            );
            output.shapes.iter().any(|shape| {
                matches!(&shape.shape, Shape::Rect(rect) if rect.stroke == Stroke::new(WIDTH, color))
            })
        };
        let tab = Event::Key {
            key: Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        assert!(!run(vec![], button::ButtonState::None));
        assert!(run(vec![tab], button::ButtonState::None));
        assert!(run(
            vec![Event::PointerMoved(Pos2::new(100.0, 100.0))],
            button::ButtonState::None
        ));
        assert!(!run(
            vec![Event::PointerButton {
                pos: button_rect.get().center(),
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            }],
            button::ButtonState::None
        ));
        assert!(!run(vec![], button::ButtonState::Disable));
    }
}
