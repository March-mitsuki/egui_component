//! A theme-driven slider. Numeric text entry is deliberately left to TextEdit.
use crate::theme::Theme;
use egui::{Color32, Response, Sense, Stroke, Ui, WidgetInfo};
use std::ops::RangeInclusive;

#[derive(Clone)]
pub struct Style {
    pub width: f32,
    pub height: f32,
    pub track_height: f32,
    pub thumb_radius: f32,
    pub track_color: Color32,
    pub fill_color: Color32,
    pub thumb_color: Color32,
    pub focus_color: Color32,
    pub disabled: bool,
    /// Keyboard increment. None uses one hundredth of the range.
    pub step: Option<f32>,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            width: 160.0,
            height: 32.0,
            track_height: 4.0,
            thumb_radius: 7.0,
            track_color: theme.colors.gray.l2,
            fill_color: theme.colors.primary.l6,
            thumb_color: theme.colors.primary.l6,
            focus_color: theme.colors.primary.l6,
            disabled: false,
            step: None,
        }
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }
}
/// Supports dragging, track clicks, arrow keys, Home, End.
pub fn render(ui: &mut Ui, value: &mut f32, range: RangeInclusive<f32>, style: Style) -> Response {
    let enabled = ui.is_enabled() && !style.disabled;
    let (rect, mut response) = ui.allocate_exact_size(
        egui::vec2(
            style.width.min(ui.available_width()).max(20.0),
            style.height,
        ),
        if enabled {
            Sense::click_and_drag()
        } else {
            Sense::hover()
        },
    );
    let (lo, hi) = (*range.start(), *range.end());
    if !lo.is_finite() || !hi.is_finite() || hi <= lo {
        return response;
    }
    let before = *value;
    if !value.is_finite() {
        *value = lo;
    }
    *value = value.clamp(lo, hi);
    let x0 = rect.left() + style.thumb_radius;
    let x1 = (rect.right() - style.thumb_radius).max(x0 + 1.0);
    if enabled {
        if (response.clicked() || response.dragged())
            && let Some(pos) = response.interact_pointer_pos()
        {
            *value = lo + ((pos.x - x0) / (x1 - x0)).clamp(0.0, 1.0) * (hi - lo);
            response.request_focus();
        }
        if response.has_focus() {
            let step = style
                .step
                .filter(|s| s.is_finite() && *s > 0.0)
                .unwrap_or((hi - lo) / 100.0);
            ui.input_mut(|input| {
                if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight)
                    || input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)
                {
                    *value += step;
                }
                if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft)
                    || input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)
                {
                    *value -= step;
                }
                if input.consume_key(egui::Modifiers::NONE, egui::Key::Home) {
                    *value = lo;
                }
                if input.consume_key(egui::Modifiers::NONE, egui::Key::End) {
                    *value = hi;
                }
            });
        }
    }
    *value = value.clamp(lo, hi);
    if *value != before {
        response.mark_changed();
    }
    response.widget_info(|| WidgetInfo::slider(enabled, *value as f64, ""));
    // No Context visuals are read: colors come exclusively from Style.
    let fade = |c: Color32| if enabled { c } else { c.gamma_multiply(0.4) };
    let center = egui::pos2(x0 + (*value - lo) / (hi - lo) * (x1 - x0), rect.center().y);
    let track = egui::Rect::from_min_max(
        egui::pos2(x0, rect.center().y - style.track_height / 2.0),
        egui::pos2(x1, rect.center().y + style.track_height / 2.0),
    );
    ui.painter()
        .rect_filled(track, style.track_height, fade(style.track_color));
    ui.painter().rect_filled(
        egui::Rect::from_min_max(track.min, egui::pos2(center.x, track.max.y)),
        style.track_height,
        fade(style.fill_color),
    );
    if response.has_focus() {
        ui.painter().circle_stroke(
            center,
            style.thumb_radius + 3.0,
            Stroke::new(1.5_f32, style.focus_color),
        );
    }
    ui.painter()
        .circle_filled(center, style.thumb_radius, fade(style.thumb_color));
    response.on_hover_cursor(if enabled {
        egui::CursorIcon::PointingHand
    } else {
        egui::CursorIcon::Default
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pointer_keyboard_and_disabled_slider() {
        let ctx = egui::Context::default();
        let theme =
            crate::theme::UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
                .unwrap()
                .light;
        let mut value = 0.0;
        let mut run = |events: Vec<egui::Event>, disabled: bool, focus: bool| {
            let mut rect = egui::Rect::NOTHING;
            let _ = ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    let r = render(
                        ui,
                        &mut value,
                        -10.0..=10.0,
                        Style::new(&theme).width(200.0).step(1.0).disabled(disabled),
                    );
                    rect = r.rect;
                    if focus {
                        r.request_focus();
                    }
                },
            );
            (value, rect)
        };
        let (_, rect) = run(vec![], false, false);
        let pos = rect.right_center() - egui::vec2(7.0, 0.0);
        for pressed in [true, false] {
            run(
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ],
                false,
                false,
            );
        }
        assert_eq!(run(vec![], false, true).0, 10.0);
        let key = |key, pressed| egui::Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Default::default(),
        };
        assert_eq!(
            run(vec![key(egui::Key::ArrowLeft, true)], false, false).0,
            9.0
        );
        run(vec![key(egui::Key::ArrowLeft, false)], false, false);
        assert_eq!(run(vec![key(egui::Key::Home, true)], false, false).0, -10.0);
        for pressed in [true, false] {
            run(
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ],
                true,
                false,
            );
        }
        assert_eq!(run(vec![], true, false).0, -10.0);
    }
}
