//! A themed swatch with an RGB(A) slider and HEX text editor popup.
use crate::{
    components::{slider, text, text_edit},
    theme::Theme,
};
use egui::{
    Color32, ColorImage, CornerRadius, Id, Pos2, Rect, Response, Sense, Stroke, StrokeKind,
    TextureHandle, TextureOptions, Ui, epaint::RectShape,
};
#[derive(Clone)]
pub struct Style {
    pub width: f32,
    pub height: f32,
    pub border_stroke: Stroke,
    pub corner_radius: CornerRadius,
    pub alpha: bool,
    pub disabled: bool,
    pub theme: Theme,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            width: theme.button.md.size,
            height: theme.button.md.size,
            border_stroke: Stroke::new(1.0_f32, theme.colors.gray.l3),
            corner_radius: theme.corner_radius.sm.into(),
            alpha: false,
            disabled: false,
            theme: theme.clone(),
        }
    }
    pub fn size(mut self, size: f32) -> Self {
        self.width = size;
        self.height = size;
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
    pub fn border_stroke(mut self, border_stroke: Stroke) -> Self {
        self.border_stroke = border_stroke;
        self
    }
    pub fn corner_radius(mut self, corner_radius: CornerRadius) -> Self {
        self.corner_radius = corner_radius;
        self
    }
    pub fn alpha(mut self, alpha: bool) -> Self {
        self.alpha = alpha;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
fn paint_checkerboard(ui: &Ui, rect: Rect, radius: CornerRadius, short_side: f32, theme: &Theme) {
    let colors = [theme.colors.bg.base, theme.colors.gray.l2];
    let key = Id::new("egui_component_color_picker_checkerboard").with(colors);
    let ctx = ui.ctx();
    let cached = ctx.data(|data| data.get_temp::<TextureHandle>(key));
    let texture = match cached {
        Some(texture) => texture,
        _ => {
            let [light, dark] = colors;
            let texture = ctx.load_texture(
                "egui_component_color_picker_checkerboard",
                ColorImage::new([2, 2], vec![light, dark, dark, light]),
                TextureOptions::NEAREST_REPEAT,
            );
            ctx.data_mut(|data| data.insert_temp(key, texture.clone()));
            texture
        }
    };

    // Each repeat contains two square cells, each a quarter of the short side.
    let repeats = rect.size() / (short_side / 2.0);
    let uv = Rect::from_min_size(Pos2::ZERO, repeats);
    ui.painter()
        .add(RectShape::filled(rect, radius, Color32::WHITE).with_texture(texture.id(), uv));
}

/// The caller supplies a stable ID; incomplete HEX edits persist without changing the color.
pub fn render(ui: &mut Ui, id: Id, color: &mut Color32, style: Style) -> Response {
    let before = *color;
    let theme = &style.theme;
    let mut response = ui
        .add_enabled_ui(!style.disabled, |ui| {
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(style.width, style.height), Sense::click());
            let short_side = rect.width().min(rect.height());
            let radius = style.corner_radius.at_most((short_side / 2.0) as u8);
            let fill = if ui.is_enabled() {
                *color
            } else {
                color.gamma_multiply(0.4)
            };
            if short_side > 0.0 && ui.is_rect_visible(rect) {
                if fill.a() < 255 {
                    paint_checkerboard(ui, rect, radius, short_side, theme);
                }
                ui.painter()
                    .rect(rect, radius, fill, style.border_stroke, StrokeKind::Inside);
            }
            super::focus::ring(
                ui,
                &response,
                rect,
                radius,
                theme.colors.primary.normal(),
                super::focus::Placement::Outside,
            );
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::ColorButton,
                    ui.is_enabled(),
                    color.to_hex(),
                )
            });
            egui::Popup::menu(&response)
                .id(id)
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                .frame(menu_button_frame(theme))
                .show(|ui| {
                    ui.set_width(240.0);
                    let mut channels = color.to_srgba_unmultiplied();
                    let mut channel_changed = false;
                    for (i, label) in ["R", "G", "B", "A"]
                        .into_iter()
                        .enumerate()
                        .take(if style.alpha { 4 } else { 3 })
                    {
                        ui.push_id(i, |ui| {
                            ui.horizontal(|ui| {
                                text::render(ui, label, text::Style::new(theme));
                                let mut v = channels[i] as f32;
                                if slider::render(
                                    ui,
                                    &mut v,
                                    0.0..=255.0,
                                    slider::Style::new(theme).width(155.0).step(1.0),
                                )
                                .changed()
                                {
                                    channels[i] = v.round() as u8;
                                    channel_changed = true;
                                }
                                text::render(ui, &channels[i].to_string(), text::Style::new(theme));
                            })
                        });
                    }
                    if !style.alpha {
                        channels[3] = 255;
                    }
                    let updated = Color32::from_rgba_unmultiplied(
                        channels[0],
                        channels[1],
                        channels[2],
                        channels[3],
                    );
                    let key = id.with("hex-draft");
                    let mut draft = ui
                        .data_mut(|data| data.get_temp::<(Color32, String)>(key))
                        .filter(|(source, _)| *source == *color)
                        .map(|(_, s)| s)
                        .unwrap_or_else(|| color.to_hex());
                    if channel_changed && updated != *color {
                        *color = updated;
                        draft = color.to_hex();
                    }
                    if text_edit::render(ui, &mut draft, text_edit::Style::new_outline_sm(theme))
                        .changed()
                        && let Ok(c) = Color32::from_hex(&draft)
                    {
                        *color = if style.alpha {
                            c
                        } else {
                            let [r, g, b, _] = c.to_srgba_unmultiplied();
                            Color32::from_rgb(r, g, b)
                        };
                    }
                    ui.data_mut(|data| data.insert_temp(key, (*color, draft)));
                });
            response
        })
        .inner;
    if *color != before {
        response.mark_changed();
    }
    response
}
fn menu_button_frame(theme: &Theme) -> egui::Frame {
    egui::Frame::new()
        .fill(theme.colors.bg.base)
        .stroke(Stroke::new(1.0_f32, theme.colors.gray.l2))
        .corner_radius(theme.corner_radius.md)
        .inner_margin(12)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn popup_preserves_unedited_alpha_and_global_visuals() {
        let ctx = egui::Context::default();
        let theme =
            crate::theme::UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
                .unwrap();
        let original = Color32::from_rgba_unmultiplied(107, 101, 245, 123);
        let mut color = original;
        let visuals = ctx.global_style().visuals.clone();
        let mut run = |events| {
            let mut rect = egui::Rect::NOTHING;
            let _ = ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    rect = render(
                        ui,
                        Id::new("color"),
                        &mut color,
                        Style::new(&theme.dark).alpha(true),
                    )
                    .rect;
                },
            );
            assert_eq!(color, original);
            assert_eq!(ctx.global_style().visuals, visuals);
            rect
        };
        let pos = run(vec![]).center();
        for pressed in [true, false] {
            run(vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ]);
        }
        run(vec![]);
        assert!(egui::Popup::is_id_open(&ctx, Id::new("color")));
    }
}
