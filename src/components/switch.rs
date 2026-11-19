//! An on/off switch.
//!
//! Space toggles a focused switch; Enter does not, matching checkbox, so Enter
//! stays free for form submission.
use super::focus;
use egui::{Color32, CornerRadius, Response, Sense, Stroke, StrokeKind, Ui};

use crate::theme::{Color, Theme};

#[derive(Clone)]
pub struct Style {
    pub focus_ring: crate::theme::FocusRing,
    pub height: f32,
    pub width: f32,
    pub on_bg_color: Color32,
    pub off_bg_color: Color32,
    pub knob_color: Color32,
    pub disabled: bool,
}

impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            height: 20.0,
            width: 36.0,
            on_bg_color: theme.colors.primary.l6,
            off_bg_color: theme.colors.gray.l3,
            knob_color: theme.colors.primary.foreground,
            disabled: false,
            focus_ring: theme.focus_ring,
        }
    }

    pub fn new_sm(theme: &Theme) -> Self {
        Self {
            height: 16.0,
            width: 28.0,
            on_bg_color: theme.colors.primary.l6,
            off_bg_color: theme.colors.gray.l3,
            knob_color: theme.colors.primary.foreground,
            disabled: false,
            focus_ring: theme.focus_ring,
        }
    }

    pub fn new_md(theme: &Theme) -> Self {
        Self::new(theme)
    }

    pub fn new_lg(theme: &Theme) -> Self {
        Self {
            height: 24.0,
            width: 44.0,
            on_bg_color: theme.colors.primary.l6,
            off_bg_color: theme.colors.gray.l3,
            knob_color: theme.colors.primary.foreground,
            disabled: false,
            focus_ring: theme.focus_ring,
        }
    }

    pub fn new_sm_color(theme: &Theme, color: Color) -> Self {
        Self {
            height: 16.0,
            width: 28.0,
            on_bg_color: color.l6,
            off_bg_color: theme.colors.gray.l3,
            knob_color: color.foreground,
            disabled: false,
            focus_ring: theme.focus_ring,
        }
    }

    pub fn new_md_color(theme: &Theme, color: Color) -> Self {
        Self {
            height: 20.0,
            width: 36.0,
            on_bg_color: color.l6,
            off_bg_color: theme.colors.gray.l3,
            knob_color: color.foreground,
            disabled: false,
            focus_ring: theme.focus_ring,
        }
    }

    pub fn new_lg_color(theme: &Theme, color: Color) -> Self {
        Self {
            height: 24.0,
            width: 44.0,
            on_bg_color: color.l6,
            off_bg_color: theme.colors.gray.l3,
            knob_color: color.foreground,
            disabled: false,
            focus_ring: theme.focus_ring,
        }
    }

    pub fn on_bg_color(mut self, color: Color32) -> Self {
        self.on_bg_color = color;
        self
    }

    pub fn off_bg_color(mut self, color: Color32) -> Self {
        self.off_bg_color = color;
        self
    }

    pub fn knob_color(mut self, color: Color32) -> Self {
        self.knob_color = color;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

pub fn render(ui: &mut Ui, on: &mut bool, style: Style) -> Response {
    let size = egui::vec2(style.width, style.height);
    let sense = if style.disabled {
        Sense::hover()
    } else {
        Sense::click()
    };
    let (rect, mut response) = ui.allocate_exact_size(size, sense);

    if super::activation::toggle_requested(ui, &response) && !style.disabled {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::Role::Switch,
            !style.disabled && ui.is_enabled(),
            *on,
            "",
        )
    });

    // Smooth transition animations
    let animation_factor = ui.ctx().animate_bool_with_time(response.id, *on, 0.15);

    let active_bg = style.on_bg_color;
    let inactive_bg = style.off_bg_color;

    // Background color (disabled has opacity adjustment)
    let fill_color = if style.disabled {
        let base = if *on { active_bg } else { inactive_bg };
        base.gamma_multiply(0.4)
    } else {
        let active_rgba = egui::Rgba::from(active_bg);
        let inactive_rgba = egui::Rgba::from(inactive_bg);
        let mixed_rgba = inactive_rgba * (1.0 - animation_factor) + active_rgba * animation_factor;
        Color32::from(mixed_rgba)
    };

    // Draw track
    let corner_radius = CornerRadius::same((rect.height() / 2.0).round() as u8);
    ui.painter().rect(
        rect,
        corner_radius,
        fill_color,
        Stroke::NONE,
        StrokeKind::Inside,
    );

    // Draw knob
    let margin = 2.0;
    let r = (style.height - 2.0 * margin) / 2.0;
    let x_start = rect.left() + margin + r;
    let x_end = rect.right() - margin - r;
    let knob_center_x = x_start + (x_end - x_start) * animation_factor;
    let knob_center = egui::pos2(knob_center_x, rect.center().y);

    let knob_color = if style.disabled {
        style.knob_color.gamma_multiply(0.6)
    } else {
        style.knob_color
    };

    // Knob shadow
    if !style.disabled {
        let shadow_center = knob_center + egui::vec2(0.0, 1.0);
        ui.painter()
            .circle_filled(shadow_center, r, Color32::from_black_alpha(35));
    }

    ui.painter().circle_filled(knob_center, r, knob_color);

    focus::ring(ui, &response, rect, corner_radius, style.focus_ring);
    if style.disabled {
        response.on_hover_cursor(egui::CursorIcon::NotAllowed)
    } else {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::UiTheme;
    use egui::{Event, Key, Modifiers, RawInput};

    fn key(key: Key) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }
    }

    fn pass(ctx: &egui::Context, events: Vec<Event>, on: &mut bool, style: &Style) -> Response {
        let mut out = None;
        ctx.run_ui(
            RawInput {
                events,
                ..Default::default()
            },
            |ui| out = Some(render(ui, on, style.clone())),
        )
        .textures_delta
        .clear();
        out.unwrap()
    }

    #[test]
    fn space_toggles_but_enter_does_not() {
        let theme = UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
            .unwrap()
            .light;
        let ctx = egui::Context::default();
        let style = Style::new(&theme);
        let mut on = false;
        pass(&ctx, vec![], &mut on, &style).request_focus();
        pass(&ctx, vec![], &mut on, &style);
        pass(&ctx, vec![key(Key::Enter)], &mut on, &style);
        assert!(!on, "Enter is for form submission");
        let r = pass(&ctx, vec![key(Key::Space)], &mut on, &style);
        assert!(on && r.changed());
    }
}
