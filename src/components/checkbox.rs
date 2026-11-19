//! A checkbox, alone or with a clickable text label.
//!
//! Space toggles a focused checkbox; Enter does not (per W3C, Enter belongs to
//! form submission). `indeterminate` is visual only, like HTML: the value stays
//! a `bool`, and clicking an indeterminate checkbox checks it.
use egui::{
    Color32, CornerRadius, Pos2, Rect, Response, Role, Sense, Stroke, StrokeKind, TextWrapMode, Ui,
    WidgetInfo, WidgetText, pos2, vec2,
};

use super::focus::{self, Placement};
use super::text;
use crate::theme::{Color, Theme};

#[derive(Clone)]
pub struct Style {
    /// Side length of the box.
    pub size: f32,
    pub corner_radius: f32,
    /// Fill of a checked or indeterminate box.
    pub checked_bg_color: Color32,
    /// Border of an unchecked box.
    pub border_color: Color32,
    /// Check mark and indeterminate dash.
    pub check_color: Color32,
    pub label_style: text::Style,
    /// Space between the box and its label.
    pub gap: f32,
    pub focus_ring: crate::theme::FocusRing,
    pub disabled: bool,
    pub indeterminate: bool,
}

impl Style {
    fn sized(theme: &Theme, color: Color, size: f32, radius: f32, text_size: f32) -> Self {
        Self {
            size,
            corner_radius: radius,
            checked_bg_color: color.l6,
            border_color: theme.colors.fg.base.gamma_multiply(0.35),
            check_color: color.foreground,
            label_style: text::Style::new(theme).size(text_size).selectable(false),
            gap: 8.0,
            focus_ring: theme.focus_ring,
            disabled: false,
            indeterminate: false,
        }
    }

    pub fn new(theme: &Theme) -> Self {
        Self::new_md(theme)
    }

    pub fn new_sm(theme: &Theme) -> Self {
        Self::new_sm_color(theme, theme.colors.primary)
    }

    pub fn new_md(theme: &Theme) -> Self {
        Self::new_md_color(theme, theme.colors.primary)
    }

    pub fn new_lg(theme: &Theme) -> Self {
        Self::new_lg_color(theme, theme.colors.primary)
    }

    pub fn new_sm_color(theme: &Theme, color: Color) -> Self {
        Self::sized(theme, color, 14.0, 3.0, theme.text_size.sm)
    }

    pub fn new_md_color(theme: &Theme, color: Color) -> Self {
        Self::sized(theme, color, 16.0, 4.0, theme.text_size.md)
    }

    pub fn new_lg_color(theme: &Theme, color: Color) -> Self {
        Self::sized(theme, color, 20.0, 5.0, theme.text_size.lg)
    }

    pub fn checked_bg_color(mut self, color: Color32) -> Self {
        self.checked_bg_color = color;
        self
    }

    pub fn border_color(mut self, color: Color32) -> Self {
        self.border_color = color;
        self
    }

    pub fn check_color(mut self, color: Color32) -> Self {
        self.check_color = color;
        self
    }

    pub fn label_style(mut self, label_style: text::Style) -> Self {
        self.label_style = label_style;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Show a dash instead of a check mark, e.g. for a partially selected
    /// "select all". Visual only; clicking still sets `checked` to `true`.
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }
}

/// Render only the box. Inside [`field`](super::field), clicking the field
/// label toggles it.
pub fn render(ui: &mut Ui, checked: &mut bool, style: Style) -> Response {
    show(ui, checked, None, style)
}

/// Render the box followed by `label`. Box and label are one control: clicking
/// either toggles it, and it is a single Tab stop.
pub fn render_with_label(ui: &mut Ui, checked: &mut bool, label: &str, style: Style) -> Response {
    show(ui, checked, Some(label), style)
}

fn show(ui: &mut Ui, checked: &mut bool, label: Option<&str>, mut style: Style) -> Response {
    // A disabled parent Ui also disables (and dims) the checkbox.
    style.disabled |= !ui.is_enabled();
    let galley = label.map(|label| {
        WidgetText::from(text::rich_text(ui, label, &style.label_style)).into_galley(
            ui,
            Some(TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Body,
        )
    });
    let text_size = galley.as_ref().map_or(egui::Vec2::ZERO, |g| g.size());
    let width = style.size + galley.as_ref().map_or(0.0, |_| style.gap + text_size.x);
    let height = style.size.max(text_size.y);
    let sense = if style.disabled {
        Sense::hover()
    } else {
        Sense::click()
    };
    let (rect, mut response) = ui.allocate_exact_size(vec2(width, height), sense);

    if super::activation::toggle_requested(ui, &response) && !style.disabled {
        *checked = !*checked;
        response.mark_changed();
    }
    let indeterminate = style.indeterminate && !response.changed();
    response.widget_info(|| {
        let mut info = WidgetInfo::selected(
            Role::CheckBox,
            !style.disabled,
            *checked,
            label.unwrap_or_default(),
        );
        if indeterminate {
            info.selected = None;
        }
        info
    });

    let box_rect = Rect::from_min_size(
        pos2(rect.left(), rect.center().y - style.size / 2.0),
        vec2(style.size, style.size),
    );
    paint_box(ui, &response, box_rect, *checked, indeterminate, &style);
    if let Some(galley) = galley {
        let pos = pos2(
            box_rect.right() + style.gap,
            rect.center().y - galley.size().y / 2.0,
        );
        if style.disabled {
            let color = style.label_style.color.gamma_multiply(0.5);
            ui.painter()
                .galley_with_override_text_color(pos, galley, color);
        } else {
            ui.painter().galley(pos, galley, style.label_style.color);
        }
    }

    focus::ring(
        ui,
        &response,
        if label.is_some() { rect } else { box_rect },
        CornerRadius::same(style.corner_radius.round() as u8),
        style.focus_ring,
        Placement::Outside,
    );
    if style.disabled {
        response.on_hover_cursor(egui::CursorIcon::NotAllowed)
    } else {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    }
}

fn paint_box(
    ui: &Ui,
    response: &Response,
    rect: Rect,
    checked: bool,
    indeterminate: bool,
    style: &Style,
) {
    let on = checked || indeterminate;
    let t = ui.ctx().animate_bool_with_time(response.id, on, 0.12);
    let alpha = if style.disabled { 0.4 } else { 1.0 };
    let radius = CornerRadius::same(style.corner_radius.round() as u8);
    let painter = ui.painter();

    let fill = style.checked_bg_color.gamma_multiply(t * alpha);
    let mut border = style.border_color;
    if response.hovered() && !style.disabled {
        border = style.checked_bg_color;
    }
    let border = lerp_color(border, style.checked_bg_color, t).gamma_multiply(alpha);
    painter.rect(
        rect,
        radius,
        fill,
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );

    if t <= 0.0 {
        return;
    }
    let mark = Stroke::new(
        (style.size / 8.0).max(1.5),
        style.check_color.gamma_multiply(t * alpha),
    );
    let at = |x: f32, y: f32| -> Pos2 { rect.min + vec2(x, y) * rect.width() };
    if indeterminate {
        painter.line_segment([at(0.27, 0.5), at(0.73, 0.5)], mark);
    } else {
        painter.line(vec![at(0.25, 0.52), at(0.43, 0.70), at(0.76, 0.32)], mark);
    }
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    Color32::from(egui::Rgba::from(a) * (1.0 - t) + egui::Rgba::from(b) * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{components::field, theme::UiTheme};
    use egui::{Event, Key, Modifiers, PointerButton, RawInput};

    fn theme() -> Theme {
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

    fn click(pos: Pos2) -> Vec<Vec<Event>> {
        let button = |pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        vec![
            vec![Event::PointerMoved(pos)],
            vec![button(true)],
            vec![button(false)],
            vec![],
            vec![],
        ]
    }

    /// Runs one pass rendering a labeled checkbox, returning its response.
    fn pass(
        ctx: &egui::Context,
        events: Vec<Event>,
        checked: &mut bool,
        style: &Style,
    ) -> Response {
        let mut out = None;
        ctx.run_ui(
            RawInput {
                events,
                ..Default::default()
            },
            |ui| out = Some(render_with_label(ui, checked, "Accept", style.clone())),
        )
        .textures_delta
        .clear();
        out.unwrap()
    }

    #[test]
    fn clicking_box_or_label_toggles() {
        let ctx = egui::Context::default();
        let style = Style::new(&theme());
        let mut checked = false;
        let r = pass(&ctx, vec![], &mut checked, &style);
        let (box_pos, label_pos) = (
            pos2(r.rect.left() + 4.0, r.rect.center().y),
            pos2(r.rect.right() - 4.0, r.rect.center().y),
        );
        for events in click(box_pos) {
            pass(&ctx, events, &mut checked, &style);
        }
        assert!(checked, "box click checks");
        for events in click(label_pos) {
            pass(&ctx, events, &mut checked, &style);
        }
        assert!(!checked, "label click unchecks");
    }

    #[test]
    fn space_toggles_but_enter_does_not() {
        let ctx = egui::Context::default();
        let style = Style::new(&theme());
        let mut checked = false;
        pass(&ctx, vec![], &mut checked, &style).request_focus();
        pass(&ctx, vec![], &mut checked, &style);
        pass(&ctx, vec![key(Key::Enter)], &mut checked, &style);
        assert!(!checked, "Enter is for form submission");
        let r = pass(&ctx, vec![key(Key::Space)], &mut checked, &style);
        assert!(checked && r.changed());
    }

    #[test]
    fn disabled_ignores_input_and_indeterminate_click_checks() {
        let ctx = egui::Context::default();
        let theme = theme();
        let mut checked = false;
        let disabled = Style::new(&theme).disabled(true);
        let r = pass(&ctx, vec![], &mut checked, &disabled);
        for events in click(r.rect.center()) {
            pass(&ctx, events, &mut checked, &disabled);
        }
        assert!(!checked);

        let mixed = Style::new(&theme).indeterminate(true);
        let r = pass(&ctx, vec![], &mut checked, &mixed);
        for events in click(r.rect.center()) {
            pass(&ctx, events, &mut checked, &mixed);
        }
        assert!(checked, "clicking an indeterminate checkbox checks it");
    }

    #[test]
    fn field_label_toggles_bare_checkbox() {
        let ctx = egui::Context::default();
        let theme = theme();
        let mut checked = false;
        let mut label_pos = Pos2::ZERO;
        let mut run = |events: Vec<Event>, checked: &mut bool| {
            ctx.run_ui(
                RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    let r = field::render(
                        ui,
                        "Subscribe",
                        |ui| render(ui, checked, Style::new(&theme)),
                        field::Style::new(&theme),
                    );
                    label_pos = pos2(r.response.rect.left() + 10.0, r.response.rect.center().y);
                },
            )
            .textures_delta
            .clear();
            label_pos
        };
        let pos = run(vec![], &mut checked);
        for events in click(pos) {
            run(events, &mut checked);
        }
        assert!(checked);
    }
}
