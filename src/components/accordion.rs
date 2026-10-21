//! A themed, keyboard-accessible disclosure with optional controlled open state.
use crate::theme::Theme;
use egui::{Color32, Id, Stroke, Ui};

#[derive(Clone)]
pub struct Style {
    pub background: Color32,
    pub hover_background: Color32,
    pub foreground: Color32,
    pub stroke: Stroke,
    pub corner_radius: egui::CornerRadius,
    pub padding: i8,
    pub default_open: bool,
    pub disabled: bool,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            background: theme.colors.bg.subtle,
            hover_background: theme.colors.gray.l1,
            foreground: theme.colors.fg.base,
            stroke: Stroke::new(1.0_f32, theme.colors.gray.l2),
            corner_radius: egui::CornerRadius::same(theme.corner_radius.md as u8),
            padding: 12,
            default_open: false,
            disabled: false,
        }
    }
    pub fn default_open(mut self, value: bool) -> Self {
        self.default_open = value;
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
}
/// Use a unique ID per item. `open` enables externally controlled, exclusive groups.
/// Without it, egui stores each item's independent state. Returns the header and optional body.
pub fn render<R>(
    ui: &mut Ui,
    id: Id,
    title: &str,
    description: Option<&str>,
    open: Option<&mut bool>,
    body: impl FnOnce(&mut Ui) -> R,
    style: Style,
) -> egui::CollapsingResponse<R> {
    ui.scope(|ui| {
        let family = crate::consts::FONT_MANAGER.default_egui_font_family(ui.ctx());
        for (text_style, font) in &mut ui.style_mut().text_styles {
            if *text_style != egui::TextStyle::Monospace {
                font.family = family.clone();
            }
        }
        if style.disabled {
            ui.disable();
        }
        let visuals = &mut ui.style_mut().visuals;
        visuals.override_text_color = Some(style.foreground);
        visuals.collapsing_header_frame = true;
        visuals.widgets.inactive.weak_bg_fill = style.background;
        visuals.widgets.hovered.weak_bg_fill = style.hover_background;
        visuals.widgets.active.weak_bg_fill = style.hover_background;
        visuals.widgets.inactive.bg_stroke = style.stroke;
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
        ] {
            widget.corner_radius = style.corner_radius;
            widget.bg_stroke = style.stroke;
        }
        let mut header = egui::CollapsingHeader::new(title)
            .id_salt(id)
            .default_open(style.default_open)
            .show_background(true)
            .icon(|ui, openness, response| {
                let mut rect = response.rect;
                rect.set_center(egui::pos2(ui.max_rect().right() - 14.0, rect.center().y));
                egui::collapsing_header::paint_default_icon(
                    ui,
                    openness,
                    &response.clone().with_new_rect(rect),
                );
            });
        if let Some(value) = open.as_deref() {
            header = header.open(Some(*value));
        }
        let response = header.show(ui, |ui| {
            egui::Frame::new()
                .inner_margin(style.padding)
                .show(ui, |ui| {
                    if let Some(description) = description {
                        ui.label(description);
                        ui.add_space(8.0);
                    }
                    body(ui)
                })
                .inner
        });
        if let Some(open) = open {
            if response.header_response.clicked() {
                *open = !*open;
                ui.ctx().request_repaint();
            }
        }
        response
    })
    .inner
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn whole_header_and_keyboard_toggle_controlled_state_but_disabled_does_not() {
        let ctx = egui::Context::default();
        let theme =
            crate::theme::UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
                .unwrap()
                .light;
        let mut open = false;
        let mut header = egui::Rect::NOTHING;
        let mut run = |events: Vec<egui::Event>, disabled: bool, focus: bool| {
            let _ = ctx.run_ui(
                egui::RawInput {
                    events,
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 300.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    let r = render(
                        ui,
                        Id::new("test"),
                        "Details",
                        None,
                        Some(&mut open),
                        |ui| {
                            ui.label("Body");
                        },
                        Style::new(&theme).disabled(disabled),
                    );
                    header = r.header_response.rect;
                    if focus {
                        r.header_response.request_focus();
                    }
                },
            );
            (open, header)
        };
        let (_, header) = run(vec![], false, false);
        assert!(header.width() > 350.0);
        let pos = header.right_center() - egui::vec2(8.0, 0.0);
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
        assert!(run(vec![], false, true).0);
        let key = |pressed| egui::Event::Key {
            key: egui::Key::Space,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Default::default(),
        };
        run(vec![key(true)], false, false);
        assert!(!run(vec![key(false)], false, false).0);
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
        assert!(!run(vec![], true, false).0);
    }
}
