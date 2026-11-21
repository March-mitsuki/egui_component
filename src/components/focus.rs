//! Shared keyboard focus indicators for themed controls.
use crate::theme::FocusRing;
use egui::{CornerRadius, Event, Id, Response, Stroke, StrokeKind, Ui};

/// Width of the contrasting line between the ring and the control's fill.
const SEPARATOR: f32 = 1.0;

pub(super) fn focused(response: &Response) -> bool {
    response.enabled() && response.sense.is_focusable() && response.has_focus()
}

/// Keep arrow keys for a focused widget that handles them itself (sliders,
/// 2D pickers), instead of letting egui move focus to a neighbor.
pub(super) fn lock_arrows(ui: &Ui, response: &Response) {
    if response.has_focus() {
        ui.memory_mut(|m| {
            m.set_focus_lock_filter(
                response.id,
                egui::EventFilter {
                    horizontal_arrows: true,
                    vertical_arrows: true,
                    ..Default::default()
                },
            );
        });
    }
}

pub(super) fn keyboard_focus(response: &Response) -> bool {
    let ctx = &response.ctx;
    let frame = ctx.cumulative_frame_nr();
    let id = Id::unique("egui_component_focus_input").with(ctx.viewport_id());
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

/// Draws the keyboard focus ring when `response` has keyboard focus.
///
/// Rings are drawn inside the control, so they are never clipped and take no
/// extra space. A thin line in a contrasting color separates the ring from the
/// control's fill, so the ring stays visible on any fill (WCAG technique C40).
pub(super) fn ring(
    ui: &Ui,
    response: &Response,
    rect: egui::Rect,
    radius: CornerRadius,
    focus_ring: FocusRing,
) {
    if keyboard_focus(response) {
        draw_ring(ui, rect, radius, focus_ring);
    }
}

/// Draws a focus ring unconditionally, for controls that track focus themselves.
pub(super) fn draw_ring(ui: &Ui, rect: egui::Rect, radius: CornerRadius, focus_ring: FocusRing) {
    let painter = ui.painter();
    let width = focus_ring.width;
    painter.rect_stroke(rect, radius, focus_ring.stroke(), StrokeKind::Inside);
    painter.rect_stroke(
        rect.shrink(width),
        shrink_radius(radius, width),
        Stroke::new(SEPARATOR, focus_ring.separator_color()),
        StrokeKind::Inside,
    );
}

fn shrink_radius(radius: CornerRadius, by: f32) -> CornerRadius {
    let by = by.round().clamp(0.0, 255.0) as u8;
    CornerRadius {
        nw: radius.nw.saturating_sub(by),
        ne: radius.ne.saturating_sub(by),
        sw: radius.sw.saturating_sub(by),
        se: radius.se.saturating_sub(by),
    }
}

/// Circular variant of [`ring`] for round controls such as slider thumbs.
pub(super) fn circle(
    ui: &Ui,
    response: &Response,
    center: egui::Pos2,
    radius: f32,
    focus_ring: FocusRing,
) {
    if keyboard_focus(response) {
        let width = focus_ring.width.min(radius);
        let painter = ui.painter();
        painter.circle_stroke(
            center,
            radius - width / 2.0,
            Stroke::new(width, focus_ring.color),
        );
        painter.circle_stroke(
            center,
            (radius - width - SEPARATOR / 2.0).max(0.0),
            Stroke::new(SEPARATOR, focus_ring.separator_color()),
        );
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
        let ring = FocusRing {
            color: egui::Color32::from_rgb(1, 2, 3),
            width: 3.0,
        };
        let button_rect = std::cell::Cell::new(egui::Rect::NOTHING);
        let run = |events, state: button::ButtonState| {
            let mut output = ctx.run_ui(
                RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    let mut style = button::Style::new_outline_md(&theme);
                    style.focus_ring = ring;
                    let options = button::Options::default().state(state);
                    button_rect.set(button::render(ui, "Action", style, options).rect);
                },
            );
            output.textures_delta.clear();
            let rects: Vec<_> = output
                .shapes
                .iter()
                .filter_map(|shape| match &shape.shape {
                    Shape::Rect(rect) => Some(rect.clone()),
                    _ => None,
                })
                .collect();
            let ring_shape = rects.iter().find(|rect| rect.stroke == ring.stroke());
            if let Some(ring_shape) = ring_shape {
                // Drawn inside the control, followed by the separator line.
                assert_eq!(ring_shape.rect, button_rect.get());
                assert_eq!(ring_shape.stroke_kind, StrokeKind::Inside);
                assert!(rects.iter().any(|rect| {
                    rect.rect == button_rect.get().shrink(ring.width)
                        && rect.stroke == Stroke::new(SEPARATOR, ring.separator_color())
                }));
            }
            ring_shape.is_some()
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
