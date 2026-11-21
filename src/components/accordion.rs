//! A themed, keyboard-accessible disclosure with optional controlled open state.
use super::focus;
use crate::{
    consts::{DEFAULT_FONT_FAMILY_ALIAS, FONT_MANAGER},
    font::weight::FontWeight,
    theme::Theme,
};
use egui::{Color32, Id, RichText, Shape, Stroke, TextStyle, TextWrapMode, Ui, WidgetText};

#[derive(Clone)]
pub struct Style {
    pub focus_ring: crate::theme::FocusRing,
    pub background: Color32,
    /// Optional hover fill. Transparent uses the normal background.
    pub hover_background: Color32,
    pub foreground: Color32,
    pub stroke: Stroke,
    pub corner_radius: egui::CornerRadius,
    /// Vertical padding on each side of the title. Negative values are treated as zero.
    pub header_padding_y: i8,
    /// Left indent of the body when the guide is shown. Negative values are treated as zero.
    pub body_indent: i8,
    /// Draws a vertical guide beside the expanded body.
    pub show_body_guide: bool,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            focus_ring: theme.focus_ring,
            background: Color32::TRANSPARENT,
            hover_background: Color32::TRANSPARENT,
            foreground: theme.colors.fg.base,
            stroke: Stroke::NONE,
            corner_radius: egui::CornerRadius::same(theme.corner_radius.md as u8),
            header_padding_y: 12,
            body_indent: 12,
            show_body_guide: false,
        }
    }
    pub fn show_body_guide(mut self, value: bool) -> Self {
        self.show_body_guide = value;
        self
    }
}
/// Behavior of [`render`], separate from visual styling.
#[derive(Clone, Copy, Debug, Default)]
pub struct Options<'a> {
    /// Secondary text under the title.
    pub description: Option<&'a str>,
    /// Whether an item without `open` starts expanded. Defaults to `false`.
    pub default_open: bool,
    pub disabled: bool,
}
impl<'a> Options<'a> {
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Use a unique ID per item. `open` enables externally controlled, exclusive groups.
/// Without it, egui stores each item's independent state. Returns the header and optional body.
/// Plain titles use the component's semibold font. Pass [`RichText`] to customize the title.
pub fn render<R>(
    ui: &mut Ui,
    id: Id,
    title: impl Into<WidgetText>,
    open: Option<&mut bool>,
    body: impl FnOnce(&mut Ui) -> R,
    style: Style,
    options: Options<'_>,
) -> egui::CollapsingResponse<R> {
    let title = title.into();
    ui.scope(|ui| {
        let family = FONT_MANAGER.default_egui_font_family(ui.ctx());
        for (text_style, font) in &mut ui.style_mut().text_styles {
            if *text_style != egui::TextStyle::Monospace {
                font.family = family.clone();
            }
        }
        if options.disabled {
            ui.disable();
        }
        let visuals = &mut ui.style_mut().visuals;
        visuals.override_text_color = Some(style.foreground);
        visuals.collapsing_header_frame = true;
        visuals.widgets.inactive.weak_bg_fill = style.background;
        let hover_background = if style.hover_background == Color32::TRANSPARENT {
            style.background
        } else {
            style.hover_background
        };
        visuals.widgets.hovered.weak_bg_fill = hover_background;
        visuals.widgets.active.weak_bg_fill = hover_background;
        visuals.widgets.inactive.bg_stroke = style.stroke;
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
        ] {
            widget.corner_radius = style.corner_radius;
            widget.bg_stroke = style.stroke;
        }
        let body_spacing = ui.spacing().clone();
        let header_spacing = ui.spacing_mut();
        header_spacing.indent = 0.0;
        header_spacing.item_spacing.y = 0.0;
        header_spacing.button_padding.y = f32::from(style.header_padding_y.max(0));
        let (title, underline_color) = match title {
            WidgetText::Text(text) => {
                let family = FONT_MANAGER.get_egui_font_family(
                    ui.ctx(),
                    DEFAULT_FONT_FAMILY_ALIAS,
                    FontWeight::SemiBold,
                    false,
                );
                (RichText::new(text).family(family).into(), style.foreground)
            }
            WidgetText::RichText(ref text) => {
                let color = text.color_override().unwrap_or(style.foreground);
                (title, color)
            }
            _ => (title, style.foreground),
        };
        let title = title.into_galley(
            ui,
            Some(TextWrapMode::Wrap),
            (ui.available_width() - 28.0).max(0.0),
            TextStyle::Button,
        );
        let mut header = egui::CollapsingHeader::new(title.clone())
            .id_salt(id)
            .default_open(options.default_open)
            .show_background(true)
            .icon(move |ui, openness, response| {
                // With zero header indent, egui centers the icon slot at the title's left edge.
                let title_anchor = response.rect.center();
                if response.enabled() && response.hovered() {
                    let title_top = title_anchor - egui::vec2(0.0, title.size().y / 2.0);
                    for row in &title.rows {
                        let rect = row
                            .rect_without_leading_space()
                            .translate(title_top.to_vec2());
                        if rect.width() > 0.0 {
                            let y = rect.bottom() - 1.0;
                            ui.painter().hline(
                                rect.x_range(),
                                y,
                                Stroke::new(1.0, underline_color),
                            );
                        }
                    }
                }

                let center = egui::pos2(ui.max_rect().right() - 14.0, title_anchor.y);
                let rotation = egui::emath::Rot2::from_angle(std::f32::consts::PI * openness);
                let color = ui.style().interact(response).fg_stroke.color;
                let points = [
                    egui::vec2(-4.0, -2.0),
                    egui::vec2(0.0, 2.0),
                    egui::vec2(4.0, -2.0),
                ];
                ui.painter().add(Shape::line(
                    points.map(|point| center + rotation * point).to_vec(),
                    Stroke::new(1.5, color),
                ));
            });
        if let Some(value) = open.as_deref() {
            header = header.open(Some(*value));
        }
        let response = header.show_unindented(ui, |ui| {
            *ui.spacing_mut() = body_spacing;
            let frame = egui::Frame::new()
                .inner_margin(egui::Margin {
                    left: if style.show_body_guide {
                        style.body_indent.max(0)
                    } else {
                        0
                    },
                    ..egui::Margin::ZERO
                })
                .show(ui, |ui| {
                    if let Some(description) = options.description {
                        ui.label(description);
                        ui.add_space(8.0);
                    }
                    body(ui)
                });
            if style.show_body_guide {
                let stroke = Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color);
                let start = frame.response.rect.left_top() + egui::vec2(0.5, 0.0);
                let end = egui::pos2(start.x, frame.response.rect.bottom());
                ui.painter().line_segment([start, end], stroke);
            }
            frame.inner
        });
        if let Some(open) = open {
            if response.header_response.clicked() {
                *open = !*open;
                ui.ctx().request_repaint();
            }
        }
        focus::ring(
            ui,
            &response.header_response,
            response.header_response.rect,
            style.corner_radius,
            style.focus_ring,
        );
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
            ctx.run_ui(
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
                        Id::unique("test"),
                        "Details",
                        Some(&mut open),
                        |ui| {
                            ui.label("Body");
                        },
                        Style::new(&theme),
                        Options::default().disabled(disabled),
                    );
                    header = r.header_response.rect;
                    if focus {
                        r.header_response.request_focus();
                    }
                },
            )
            .textures_delta
            .clear();
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
