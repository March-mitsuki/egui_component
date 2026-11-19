//! A themed swatch with an HSV color plane and HEX/RGB(A) editors.
use crate::{
    components::{button, text, text_edit},
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
/// The input responsible for a validation error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorField {
    Hex,
    Red,
    Green,
    Blue,
    Alpha,
}

/// Structured validation errors; applications supply localized messages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColorInputError {
    Empty { field: ColorField },
    InvalidHexLength { digits: usize },
    InvalidHexDigit,
    InvalidNumber { field: ColorField },
    OutOfRange { field: ColorField },
}

/// Parse RGB, RGBA, RRGGBB or RRGGBBAA, with optional `#` and surrounding whitespace.
/// Channels are straight (not premultiplied) sRGBA bytes, including at zero alpha.
pub fn hex_to_rgba(hex: &str) -> Result<[u8; 4], ColorInputError> {
    let hex = hex.trim();
    let digits = hex.strip_prefix('#').unwrap_or(hex);
    if digits.is_empty() {
        return Err(ColorInputError::Empty {
            field: ColorField::Hex,
        });
    }
    let len = digits.chars().count();
    if !matches!(len, 3 | 4 | 6 | 8) {
        return Err(ColorInputError::InvalidHexLength { digits: len });
    }
    if !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(ColorInputError::InvalidHexDigit);
    }
    let nibble = |b: u8| (b as char).to_digit(16).unwrap() as u8;
    let mut rgba = [255; 4];
    if len <= 4 {
        for (out, b) in rgba.iter_mut().zip(digits.bytes()) {
            *out = nibble(b) * 17;
        }
    } else {
        for (out, pair) in rgba
            .iter_mut()
            .zip(digits.as_bytes().as_chunks::<2>().0.iter())
        {
            *out = nibble(pair[0]) * 16 + nibble(pair[1]);
        }
    }
    Ok(rgba)
}

/// Format straight sRGBA bytes as lowercase `#rrggbbaa`, without precision loss.
pub fn rgba_to_hex([r, g, b, a]: [u8; 4]) -> String {
    format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
}

/// Optional behavior configuration, separate from visual styling.
#[derive(Default)]
pub struct Options<'a> {
    /// Called when a committed input is invalid. Without this, only an error outline is shown.
    pub error_message: Option<&'a dyn Fn(&ColorInputError) -> String>,
}

#[derive(Clone)]
struct Editor {
    source: Color32,
    alpha: bool,
    rgba: [u8; 4],
    hsv: egui::ecolor::Hsva,
    hex_mode: bool,
    drafts: [String; 4],
    errors: [Option<ColorInputError>; 4],
}
impl Editor {
    fn new(source: Color32, alpha: bool) -> Self {
        let rgba = source.to_srgba_unmultiplied();
        let mut state = Self {
            source,
            alpha,
            rgba,
            hsv: egui::ecolor::Hsva::from_srgba_unmultiplied(rgba),
            hex_mode: true,
            drafts: Default::default(),
            errors: Default::default(),
        };
        state.refresh();
        state
    }
    fn refresh(&mut self) {
        self.errors = Default::default();
        self.drafts = if self.hex_mode {
            let mut hex = rgba_to_hex(self.rgba);
            if !self.alpha {
                hex.truncate(7);
            }
            [hex, String::new(), String::new(), String::new()]
        } else {
            [
                self.rgba[0].to_string(),
                self.rgba[1].to_string(),
                self.rgba[2].to_string(),
                format!("{:.3}", self.rgba[3] as f32 / 255.0),
            ]
        };
    }
    fn set_rgba(&mut self, mut rgba: [u8; 4]) {
        if !self.alpha {
            rgba[3] = 255;
        }
        self.rgba = rgba;
        let mut hsv = egui::ecolor::Hsva::from_srgba_unmultiplied(rgba);
        if hsv.s == 0.0 {
            hsv.h = self.hsv.h;
        }
        if hsv.v == 0.0 {
            hsv.s = self.hsv.s;
        }
        self.hsv = hsv;
    }
    fn commit(&mut self, index: usize) -> bool {
        let result = if self.hex_mode {
            hex_to_rgba(&self.drafts[0])
        } else {
            parse_channel(&self.drafts[index], index).map(|value| {
                let mut rgba = self.rgba;
                rgba[index] = value;
                rgba
            })
        };
        match result {
            Ok(rgba) => {
                self.set_rgba(rgba);
                // Normalize this field only: preserve other invalid drafts.
                let drafts = self.drafts.clone();
                let errors = self.errors.clone();
                self.refresh();
                for i in 0..4 {
                    if i != index {
                        self.drafts[i] = drafts[i].clone();
                        self.errors[i] = errors[i].clone();
                    }
                }
                true
            }
            Err(error) => {
                self.errors[index] = Some(error);
                false
            }
        }
    }
}
fn parse_channel(input: &str, index: usize) -> Result<u8, ColorInputError> {
    let field = [
        ColorField::Red,
        ColorField::Green,
        ColorField::Blue,
        ColorField::Alpha,
    ][index];
    let input = input.trim();
    if input.is_empty() {
        return Err(ColorInputError::Empty { field });
    }
    let value: f64 = input
        .parse()
        .map_err(|_| ColorInputError::InvalidNumber { field })?;
    if !value.is_finite() {
        return Err(ColorInputError::InvalidNumber { field });
    }
    let max = if index == 3 { 1.0 } else { 255.0 };
    if !(0.0..=max).contains(&value) {
        return Err(ColorInputError::OutOfRange { field });
    }
    if index != 3 && value.fract() != 0.0 {
        return Err(ColorInputError::InvalidNumber { field });
    }
    Ok((value * if index == 3 { 255.0 } else { 1.0 }).round() as u8)
}

// Tessellated gradients avoid texture allocation and preserve the exact HSV plane.
fn gradient(ui: &Ui, rect: Rect, columns: usize, rows: usize, color: impl Fn(f32, f32) -> Color32) {
    let mut mesh = egui::epaint::Mesh::default();
    for y in 0..=rows {
        for x in 0..=columns {
            let u = x as f32 / columns as f32;
            let v = y as f32 / rows as f32;
            mesh.colored_vertex(
                egui::pos2(egui::lerp(rect.x_range(), u), egui::lerp(rect.y_range(), v)),
                color(u, v),
            );
        }
    }
    for y in 0..rows {
        for x in 0..columns {
            let a = (y * (columns + 1) + x) as u32;
            let b = a + columns as u32 + 1;
            mesh.add_triangle(a, a + 1, b);
            mesh.add_triangle(a + 1, b + 1, b);
        }
    }
    ui.painter().add(mesh);
}
fn handle(ui: &Ui, position: Pos2) {
    ui.painter().circle_stroke(
        position,
        6.0,
        Stroke::new(3.0, Color32::from_black_alpha(90)),
    );
    ui.painter()
        .circle_stroke(position, 5.0, Stroke::new(2.0, Color32::WHITE));
}
fn picker_ui(ui: &mut Ui, state: &mut Editor, theme: &Theme, options: &Options<'_>) -> bool {
    ui.set_width(280.0);
    let mut changed = false;
    let (plane, response) =
        ui.allocate_exact_size(egui::vec2(280.0, 170.0), Sense::click_and_drag());
    if let Some(pos) = response.interact_pointer_pos() {
        state.hsv.s = ((pos.x - plane.left()) / plane.width()).clamp(0.0, 1.0);
        state.hsv.v = (1.0 - (pos.y - plane.top()) / plane.height()).clamp(0.0, 1.0);
        changed = true;
    }
    gradient(ui, plane, 24, 24, |s, y| {
        egui::ecolor::Hsva::new(state.hsv.h, s, 1.0 - y, 1.0).into()
    });
    handle(
        ui,
        egui::pos2(
            plane.left() + state.hsv.s * plane.width(),
            plane.bottom() - state.hsv.v * plane.height(),
        ),
    );
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        let (preview, _) = ui.allocate_exact_size(egui::vec2(44.0, 44.0), Sense::hover());
        paint_checkerboard(ui, preview, CornerRadius::same(8), 16.0, theme);
        ui.painter().rect_filled(
            preview,
            8,
            Color32::from_rgba_unmultiplied(
                state.rgba[0],
                state.rgba[1],
                state.rgba[2],
                state.rgba[3],
            ),
        );
        ui.vertical(|ui| {
            for alpha in [false, true]
                .into_iter()
                .take(if state.alpha { 2 } else { 1 })
            {
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(224.0, 18.0), Sense::click_and_drag());
                if let Some(pos) = response.interact_pointer_pos() {
                    let value = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                    if alpha {
                        state.hsv.a = value;
                    } else {
                        state.hsv.h = value;
                    }
                    changed = true;
                }
                if alpha {
                    paint_checkerboard(ui, rect, CornerRadius::ZERO, 16.0, theme);
                    gradient(ui, rect, 32, 1, |x, _| {
                        egui::ecolor::Hsva { a: x, ..state.hsv }.into()
                    });
                } else {
                    gradient(ui, rect, 36, 1, |x, _| {
                        egui::ecolor::Hsva::new(x, 1.0, 1.0, 1.0).into()
                    });
                }
                handle(
                    ui,
                    egui::pos2(
                        rect.left() + if alpha { state.hsv.a } else { state.hsv.h } * rect.width(),
                        rect.center().y,
                    ),
                );
            }
        });
    });
    if changed {
        if !state.alpha {
            state.hsv.a = 1.0;
        }
        state.rgba = state.hsv.to_srgba_unmultiplied();
        state.refresh();
    }
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        let label = if state.hex_mode {
            "HEX"
        } else if state.alpha {
            "RGBA"
        } else {
            "RGB"
        };
        if button::render(ui, label, button::Style::new_ghost_sm(theme)).clicked() {
            state.hex_mode = !state.hex_mode;
            state.refresh();
        }
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let count = if state.hex_mode {
                    1
                } else if state.alpha {
                    4
                } else {
                    3
                };
                let width = (ui.available_width()
                    - ui.spacing().item_spacing.x * (count - 1) as f32)
                    / count as f32;
                for i in 0..count {
                    ui.push_id(i, |ui| {
                        let response = text_edit::render(
                            ui,
                            &mut state.drafts[i],
                            text_edit::Style::new_outline_sm(theme)
                                .padding_x(5.0)
                                .width(Some(width)),
                        );
                        let response = response.on_hover_text(if state.hex_mode {
                            "HEX"
                        } else {
                            ["R (0–255)", "G (0–255)", "B (0–255)", "A (0–1)"][i]
                        });
                        if response.changed() {
                            // Update valid input immediately, but leave the user's spelling
                            // and caret untouched until the edit is committed.
                            let drafts = state.drafts.clone();
                            changed |= state.commit(i);
                            state.drafts = drafts;
                            state.errors[i] = None;
                        }
                        if response.lost_focus()
                            || (response.has_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                        {
                            changed |= state.commit(i);
                        }
                        if state.errors[i].is_some() {
                            ui.painter().rect_stroke(
                                response.rect,
                                theme.corner_radius.xs,
                                Stroke::new(1.0, theme.colors.red.normal()),
                                StrokeKind::Inside,
                            );
                        }
                    });
                }
            });
            if let (Some(error), Some(message)) =
                (state.errors.iter().flatten().next(), options.error_message)
            {
                text::render(
                    ui,
                    &message(error),
                    text::Style::new(theme)
                        .color(theme.colors.red.normal())
                        .size(11.0),
                );
            }
        });
    });
    if changed {
        ui.ctx().request_repaint();
    }
    changed
}

fn paint_checkerboard(ui: &Ui, rect: Rect, radius: CornerRadius, short_side: f32, theme: &Theme) {
    let colors = [theme.colors.bg.base, theme.colors.gray.l2];
    let key = Id::unique("egui_component_color_picker_checkerboard").with(colors);
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
    render_with_options(ui, id, color, style, Options::default())
}

/// Render with caller-owned error localization. Valid edits update immediately;
/// Enter or focus loss normalizes the field and reveals validation errors.
/// The callback is borrowed for this frame and is never stored in egui state.
pub fn render_with_options(
    ui: &mut Ui,
    id: Id,
    color: &mut Color32,
    style: Style,
    options: Options<'_>,
) -> Response {
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
            super::focus::ring(ui, &response, rect, radius, theme.focus_ring);
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::ColorWell, ui.is_enabled(), color.to_hex())
            });
            egui::Popup::menu(&response)
                .id(id)
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                .frame(menu_button_frame(theme))
                .show(|ui| {
                    let key = id.with("editor");
                    let mut state = ui
                        .data_mut(|data| data.get_temp::<Editor>(key))
                        .filter(|state| state.source == *color && state.alpha == style.alpha)
                        .unwrap_or_else(|| Editor::new(*color, style.alpha));
                    if picker_ui(ui, &mut state, theme, &options) {
                        let [r, g, b, a] = state.rgba;
                        *color = Color32::from_rgba_unmultiplied(r, g, b, a);
                        state.source = *color;
                    }
                    ui.data_mut(|data| data.insert_temp(key, state));
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
    fn hex_formats_are_lossless_and_accept_optional_hash() {
        for (input, expected) in [
            (" abc ", [170, 187, 204, 255]),
            ("#abcd", [170, 187, 204, 221]),
            ("89B4FA", [137, 180, 250, 255]),
            ("#12345600", [18, 52, 86, 0]),
            ("12345601", [18, 52, 86, 1]),
        ] {
            assert_eq!(hex_to_rgba(input), Ok(expected));
            assert_eq!(hex_to_rgba(&rgba_to_hex(expected)), Ok(expected));
        }
        for alpha in 0..=255 {
            let rgba = [137, 180, 250, alpha];
            assert_eq!(hex_to_rgba(&rgba_to_hex(rgba)), Ok(rgba));
        }
    }

    #[test]
    fn invalid_inputs_have_structured_errors() {
        assert_eq!(
            hex_to_rgba("#"),
            Err(ColorInputError::Empty {
                field: ColorField::Hex
            })
        );
        assert_eq!(
            hex_to_rgba("12"),
            Err(ColorInputError::InvalidHexLength { digits: 2 })
        );
        for input in ["#ggg", "##abc", "颜色啊"] {
            assert_eq!(hex_to_rgba(input), Err(ColorInputError::InvalidHexDigit));
        }
        assert_eq!(
            parse_channel("256", 0),
            Err(ColorInputError::OutOfRange {
                field: ColorField::Red
            })
        );
        assert_eq!(
            parse_channel("1.5", 0),
            Err(ColorInputError::InvalidNumber {
                field: ColorField::Red
            })
        );
        assert_eq!(
            parse_channel("NaN", 3),
            Err(ColorInputError::InvalidNumber {
                field: ColorField::Alpha
            })
        );
        assert_eq!(parse_channel("0.5", 3), Ok(128));
        assert_eq!(
            parse_channel("1.1", 3),
            Err(ColorInputError::OutOfRange {
                field: ColorField::Alpha
            })
        );
    }

    #[test]
    fn drafts_preserve_color_and_other_invalid_fields() {
        let mut editor = Editor::new(Color32::RED, true);
        editor.drafts[0] = "#oops".into();
        assert!(!editor.commit(0));
        assert_eq!(editor.rgba, [255, 0, 0, 255]);
        assert_eq!(editor.drafts[0], "#oops");
        editor.drafts[0] = "12345600".into();
        assert!(editor.commit(0));
        assert_eq!(editor.drafts[0], "#12345600");
        assert_eq!(editor.rgba, [18, 52, 86, 0]);
        editor.hex_mode = false;
        editor.refresh();
        editor.drafts[0] = "999".into();
        assert!(!editor.commit(0));
        editor.drafts[1] = "100".into();
        assert!(editor.commit(1));
        assert_eq!(editor.drafts[0], "999");
        assert!(editor.errors[0].is_some());
        assert_eq!(editor.rgba, [18, 100, 86, 0]);
    }

    #[test]
    fn black_and_gray_keep_hue_and_opaque_edits_remove_alpha() {
        let mut editor = Editor::new(Color32::BLUE, false);
        let hue = editor.hsv.h;
        editor.set_rgba([0, 0, 0, 0]);
        assert_eq!(editor.hsv.h, hue);
        assert_eq!(editor.hsv.s, 1.0);
        assert_eq!(editor.rgba[3], 255);
        editor.set_rgba([128, 128, 128, 0]);
        assert_eq!(editor.hsv.h, hue);
    }

    #[test]
    fn both_modes_fit_popup_and_localize_errors() {
        let ctx = egui::Context::default();
        let theme =
            crate::theme::UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
                .unwrap();
        let calls = std::cell::Cell::new(0);
        let message = |error: &ColorInputError| {
            assert_eq!(*error, ColorInputError::InvalidHexDigit);
            calls.set(calls.get() + 1);
            "This is a localized error message that should wrap inside the color picker.".to_owned()
        };
        for alpha in [false, true] {
            for hex_mode in [false, true] {
                let mut state = Editor::new(Color32::BLUE, alpha);
                state.hex_mode = hex_mode;
                state.refresh();
                if hex_mode {
                    state.errors[0] = Some(ColorInputError::InvalidHexDigit);
                }
                ctx.run_ui(egui::RawInput::default(), |ui| {
                    let response = ui
                        .scope(|ui| {
                            assert!(!picker_ui(
                                ui,
                                &mut state,
                                &theme.dark,
                                &Options {
                                    error_message: Some(&message)
                                }
                            ));
                        })
                        .response;
                    assert!(response.rect.width() <= 281.0, "{:?}", response.rect);
                })
                .textures_delta
                .clear();
            }
        }
        assert!(calls.get() >= 2);
    }

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
            ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    rect = render(
                        ui,
                        Id::unique("color"),
                        &mut color,
                        Style::new(&theme.dark).alpha(true),
                    )
                    .rect;
                },
            )
            .textures_delta
            .clear();
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
        assert!(egui::Popup::is_id_open(&ctx, Id::unique("color")));
    }
}
