use super::focus::{self, Placement};
use egui::{Color32, CornerRadius, Stroke, StrokeKind, Ui};

use crate::{
    components::icon_button,
    consts::FONT_MANAGER,
    theme::{Color, TextEditPalette, TextEditSize, TextEditVariant, Theme},
};

#[derive(Clone, PartialEq, Default)]
pub enum InputType {
    #[default]
    Text,
    Password,
}

#[derive(Clone)]
pub struct Style {
    pub padding_x: f32,
    pub height: f32,
    pub corner_radius: CornerRadius,
    pub text_size: f32,
    pub palette: TextEditPalette,
    /// placeholder 文字
    pub placeholder: String,
    /// 是否强制固定宽度（None = 占满可用宽度）
    pub width: Option<f32>,

    pub variant: TextEditVariant,
    pub input_type: InputType,
    pub disabled: bool,
    pub read_only: bool,
    pub icon_button_style: icon_button::Style,
}

pub const TEXT_EDIT_HEIGHT_XS: f32 = 18.0;
pub const TEXT_EDIT_HEIGHT_SM: f32 = 24.0;
pub const TEXT_EDIT_HEIGHT_MD: f32 = 32.0;
pub const TEXT_EDIT_HEIGHT_LG: f32 = 40.0;

impl Style {
    pub fn new(
        theme: &Theme,
        color: Option<Color>,
        size: TextEditSize,
        variant: TextEditVariant,
    ) -> Self {
        let (height, padding_x, text_size, corner_radius) = match size {
            TextEditSize::Xs => (
                TEXT_EDIT_HEIGHT_XS,
                8.0,
                10.0,
                CornerRadius::same(theme.corner_radius.xs as u8),
            ),
            TextEditSize::Sm => (
                TEXT_EDIT_HEIGHT_SM,
                10.0,
                12.0,
                CornerRadius::same(theme.corner_radius.xs as u8),
            ),
            TextEditSize::Md => (
                TEXT_EDIT_HEIGHT_MD,
                12.0,
                14.0,
                CornerRadius::same(theme.corner_radius.md as u8),
            ),
            TextEditSize::Lg => (
                TEXT_EDIT_HEIGHT_LG,
                16.0,
                16.0,
                CornerRadius::same(theme.corner_radius.md as u8),
            ),
        };

        // Flushed 变体不需要圆角和水平内边距
        let (corner_radius, padding_x) = if variant == TextEditVariant::Flushed {
            (CornerRadius::ZERO, 0.0)
        } else {
            (corner_radius, padding_x)
        };

        let button_size = match size {
            TextEditSize::Xs => crate::theme::ButtonSize::Xs,
            TextEditSize::Sm => crate::theme::ButtonSize::Xs,
            TextEditSize::Md => crate::theme::ButtonSize::Sm,
            TextEditSize::Lg => crate::theme::ButtonSize::Md,
        };
        let icon_button_style = icon_button::Style::new(
            theme,
            color.clone(),
            button_size,
            crate::theme::ButtonVariant::Ghost,
        );

        let palette = match_text_edit_palette(theme, color, &variant);
        Self {
            height,
            padding_x,
            text_size,
            corner_radius,
            palette,
            placeholder: String::new(),
            width: None,
            variant,
            input_type: InputType::Text,
            disabled: false,
            read_only: false,
            icon_button_style,
        }
    }

    // ── Subtle 便捷构造 ──

    pub fn new_subtle_xs(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Xs, TextEditVariant::Subtle)
    }

    pub fn new_subtle_xs_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Xs,
            TextEditVariant::Subtle,
        )
    }

    pub fn new_subtle_sm(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Sm, TextEditVariant::Subtle)
    }

    pub fn new_subtle_sm_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Sm,
            TextEditVariant::Subtle,
        )
    }

    pub fn new_subtle_md(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Md, TextEditVariant::Subtle)
    }

    pub fn new_subtle_md_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Md,
            TextEditVariant::Subtle,
        )
    }

    pub fn new_subtle_lg(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Lg, TextEditVariant::Subtle)
    }

    pub fn new_subtle_lg_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Lg,
            TextEditVariant::Subtle,
        )
    }

    // ── Outline 便捷构造 ──

    pub fn new_outline_xs(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Xs, TextEditVariant::Outline)
    }

    pub fn new_outline_xs_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Xs,
            TextEditVariant::Outline,
        )
    }

    pub fn new_outline_sm(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Sm, TextEditVariant::Outline)
    }

    pub fn new_outline_sm_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Sm,
            TextEditVariant::Outline,
        )
    }

    pub fn new_outline_md(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Md, TextEditVariant::Outline)
    }

    pub fn new_outline_md_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Md,
            TextEditVariant::Outline,
        )
    }

    pub fn new_outline_lg(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Lg, TextEditVariant::Outline)
    }

    pub fn new_outline_lg_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Lg,
            TextEditVariant::Outline,
        )
    }

    // ── Flushed 便捷构造 ──

    pub fn new_flushed_xs(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Xs, TextEditVariant::Flushed)
    }

    pub fn new_flushed_xs_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Xs,
            TextEditVariant::Flushed,
        )
    }

    pub fn new_flushed_sm(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Sm, TextEditVariant::Flushed)
    }

    pub fn new_flushed_sm_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Sm,
            TextEditVariant::Flushed,
        )
    }

    pub fn new_flushed_md(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Md, TextEditVariant::Flushed)
    }

    pub fn new_flushed_md_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Md,
            TextEditVariant::Flushed,
        )
    }

    pub fn new_flushed_lg(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Lg, TextEditVariant::Flushed)
    }

    pub fn new_flushed_lg_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Lg,
            TextEditVariant::Flushed,
        )
    }

    // ── Builder setters ──

    pub fn padding_x(mut self, padding_x: f32) -> Self {
        self.padding_x = padding_x;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn corner_radius(mut self, corner_radius: CornerRadius) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    pub fn text_size(mut self, text_size: f32) -> Self {
        self.text_size = text_size;
        self
    }

    pub fn palette(mut self, palette: TextEditPalette) -> Self {
        self.palette = palette;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn width(mut self, width: Option<f32>) -> Self {
        self.width = width;
        self
    }

    pub fn input_type(mut self, input_type: InputType) -> Self {
        self.input_type = input_type;
        self
    }

    /// Disable focus and interaction for the entire input.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Allow selection and copying while preventing edits.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
}

pub fn render(ui: &mut Ui, text: &mut String, style: Style) -> egui::Response {
    ui.add_enabled_ui(!style.disabled, |ui| render_singleline(ui, text, style))
        .inner
}

fn render_singleline(ui: &mut Ui, text: &mut String, style: Style) -> egui::Response {
    let font_id = egui::FontId::new(
        style.text_size,
        FONT_MANAGER.default_egui_font_family(ui.ctx()),
    );
    let width = style.width.unwrap_or_else(|| ui.available_width());
    let (id, rect) = ui.allocate_space(egui::vec2(width, style.height));
    let background = ui.painter().add(egui::Shape::Noop);

    let mut editor_rect = rect;
    let password = style.input_type == InputType::Password;
    let visibility_id = id.with("password_visible");
    let mut password_visible = false;
    let mut icon_rect = egui::Rect::NOTHING;
    if password {
        password_visible = ui.data(|data| data.get_temp(visibility_id).unwrap_or(false));
        let icon_right = rect.right() - style.padding_x;
        let icon_left = icon_right - style.icon_button_style.width;
        icon_rect = egui::Rect::from_min_max(
            egui::pos2(icon_left, rect.top()),
            egui::pos2(icon_right, rect.bottom()),
        );
        editor_rect.set_right((icon_left - 4.0).max(rect.left()));
    }

    let mut editor_ui = ui.new_child(
        egui::UiBuilder::new()
            .id_salt("singleline")
            .max_rect(editor_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    editor_ui.shrink_clip_rect(editor_rect);
    let enabled = editor_ui.is_enabled();
    let visuals = editor_ui.visuals_mut();
    visuals.text_cursor.stroke = style.palette.focus_stroke;
    visuals.selection.bg_fill = style.palette.focus_stroke.color.gamma_multiply(0.25);

    let hint = egui::RichText::new(&style.placeholder)
        .font(font_id.clone())
        .color(style.palette.placeholder_color);
    let mut read_only_text;
    let buffer: &mut dyn egui::TextBuffer = if style.read_only {
        read_only_text = text.as_str();
        &mut read_only_text
    } else {
        text
    };
    let mut response = editor_ui.add(
        egui::TextEdit::singleline(buffer)
            .id(id)
            .interactive(enabled)
            .password(password && !password_visible)
            .font(font_id)
            .text_color(style.palette.text_color)
            .hint_text(hint)
            .frame(egui::Frame::new().inner_margin(egui::Margin {
                left: style.padding_x.round() as i8,
                right: if password {
                    0
                } else {
                    style.padding_x.round() as i8
                },
                ..egui::Margin::ZERO
            }))
            .desired_width(editor_rect.width())
            .min_size(editor_rect.size())
            .align(egui::Align2::LEFT_CENTER),
    );

    // Render the toggle after the input so Tab order is input -> toggle.
    let mut toggle_hovered = false;
    if password {
        let icon = if password_visible {
            crate::icon::ICONS.eye.clone()
        } else {
            crate::icon::ICONS.eye_slash.clone()
        };
        let mut button_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("password_toggle")
                .max_rect(icon_rect)
                .layout(egui::Layout::centered_and_justified(
                    egui::Direction::LeftToRight,
                )),
        );
        let toggle = icon_button::render(&mut button_ui, icon, style.icon_button_style.clone())
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        toggle_hovered = toggle.hovered();
        if toggle.clicked() {
            // Takes effect on the repaint that follows the click.
            ui.data_mut(|data| data.insert_temp(visibility_id, !password_visible));
            ui.ctx().request_repaint();
        }
        // Pointer users keep typing in the input; keyboard users stay on the toggle.
        if toggle.is_pointer_button_down_on() || toggle.clicked_by(egui::PointerButton::Primary) {
            response.request_focus();
        }
    }

    let hover = ui.ctx().animate_bool_with_time(
        id.with("hover"),
        enabled && (response.hovered() || toggle_hovered),
        0.15,
    );
    let focus = ui
        .ctx()
        .animate_bool_with_time(id.with("focus"), focus::focused(&response), 0.15);
    if style.variant != TextEditVariant::Flushed {
        let (target, amount) = if focus > 0.01 {
            (style.palette.focus_bg, focus)
        } else {
            (style.palette.hover_bg, hover)
        };
        let fill = Color32::from(
            egui::Rgba::from(style.palette.normal_bg) * (1.0 - amount)
                + egui::Rgba::from(target) * amount,
        );
        ui.painter().set(
            background,
            egui::Shape::rect_filled(rect, style.corner_radius, fill),
        );
        ui.painter().rect_stroke(
            rect,
            style.corner_radius,
            style.palette.border_stroke,
            StrokeKind::Inside,
        );
    }
    paint_focus(ui, &response, rect, &style);

    response.rect = rect;
    response
}

fn paint_focus(ui: &Ui, response: &egui::Response, rect: egui::Rect, style: &Style) {
    let focused = focus::focused(response);
    if style.variant == TextEditVariant::Flushed {
        let stroke = if focused {
            style.palette.focus_stroke
        } else {
            style.palette.border_stroke
        };
        let y = rect.bottom() - stroke.width / 2.0;
        ui.painter().hline(rect.x_range(), y, stroke);
    } else if focused {
        focus::draw_ring(
            ui,
            rect,
            style.corner_radius,
            style.palette.focus_stroke,
            Placement::Outside,
        );
    }
}

pub fn match_text_edit_palette(
    theme: &Theme,
    color: Option<Color>,
    variant: &TextEditVariant,
) -> TextEditPalette {
    // focus 颜色：如果指定了 color 用 color.normal()，否则用 primary
    let focus_color = match color {
        Some(c) => c.normal(),
        None => theme.colors.primary.normal(),
    };

    match variant {
        TextEditVariant::Subtle => {
            // 有底色填充，无边框，focus 时出现 focus ring
            TextEditPalette {
                normal_bg: theme.colors.fg.base.gamma_multiply(0.06),
                hover_bg: theme.colors.fg.base.gamma_multiply(0.1),
                focus_bg: theme.colors.fg.base.gamma_multiply(0.06),
                text_color: theme.colors.fg.base,
                placeholder_color: theme.colors.fg.base.gamma_multiply(0.35),
                border_stroke: Stroke::new(1.0_f32, Color32::TRANSPARENT),
                focus_stroke: Stroke::new(focus::WIDTH, focus_color),
            }
        }
        TextEditVariant::Outline => {
            // 透明背景，有边框，focus 时边框变为 focus color
            TextEditPalette {
                normal_bg: Color32::TRANSPARENT,
                hover_bg: Color32::TRANSPARENT,
                focus_bg: Color32::TRANSPARENT,
                text_color: theme.colors.fg.base,
                placeholder_color: theme.colors.fg.base.gamma_multiply(0.35),
                border_stroke: Stroke::new(1.0_f32, theme.colors.fg.base.gamma_multiply(0.25)),
                focus_stroke: Stroke::new(focus::WIDTH, focus_color),
            }
        }
        TextEditVariant::Flushed => {
            // 透明背景，只有底部边线，focus 时底线变为 focus color
            TextEditPalette {
                normal_bg: Color32::TRANSPARENT,
                hover_bg: Color32::TRANSPARENT,
                focus_bg: Color32::TRANSPARENT,
                text_color: theme.colors.fg.base,
                placeholder_color: theme.colors.fg.base.gamma_multiply(0.35),
                border_stroke: Stroke::new(1.0_f32, theme.colors.fg.base.gamma_multiply(0.25)),
                focus_stroke: Stroke::new(focus::WIDTH, focus_color),
            }
        }
    }
}

/// A multiline version of the standard input. Styles stay local to this editor.
/// Uses egui's text editing engine for selection, clipboard, undo, and IME.
pub fn render_multiline(
    ui: &mut Ui,
    text: &mut String,
    rows: usize,
    style: Style,
) -> egui::Response {
    ui.add_enabled_ui(!style.disabled, |ui| {
        let font_id = egui::FontId::new(
            style.text_size,
            FONT_MANAGER.default_egui_font_family(ui.ctx()),
        );
        // egui lays out the hint separately from the editable text.
        let hint = egui::RichText::new(&style.placeholder).font(font_id.clone());
        let visuals = ui.visuals_mut();
        visuals.override_text_color = Some(style.palette.text_color);
        visuals.weak_text_color = Some(style.palette.placeholder_color);
        visuals.text_cursor.stroke = style.palette.focus_stroke;
        visuals.selection.bg_fill = style.palette.focus_stroke.color.gamma_multiply(0.25);

        let flushed = style.variant == TextEditVariant::Flushed;
        let frame = egui::Frame::new()
            .fill(style.palette.normal_bg)
            .corner_radius(style.corner_radius)
            .inner_margin(egui::Margin::symmetric(4, 2))
            .stroke(if flushed {
                Stroke::NONE
            } else {
                style.palette.border_stroke
            });
        let mut read_only_text;
        let buffer: &mut dyn egui::TextBuffer = if style.read_only {
            read_only_text = text.as_str();
            &mut read_only_text
        } else {
            text
        };
        let response = ui.add(
            egui::TextEdit::multiline(buffer)
                .interactive(ui.is_enabled())
                .frame(frame)
                .font(font_id)
                .text_color(style.palette.text_color)
                .hint_text(hint)
                .desired_width(style.width.unwrap_or(ui.available_width()))
                .desired_rows(rows),
        );
        paint_focus(ui, &response, response.rect, &style);
        response
    })
    .inner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_modes_control_editing_and_interaction() {
        let theme =
            crate::theme::UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
                .unwrap();

        for multiline in [false, true] {
            let ctx = egui::Context::default();
            let mut text = String::new();
            for (read_only, disabled, input, expected) in [
                (false, false, "", ""),
                (false, false, "a", "a"),
                (true, false, "b", "a"),
                (true, true, "c", "a"),
                (false, false, "", "a"),
                (false, false, "d", "ad"),
            ] {
                let style = Style::new_outline_md(&theme.light)
                    .read_only(read_only)
                    .disabled(disabled);
                let events = if input.is_empty() {
                    vec![]
                } else {
                    vec![egui::Event::Text(input.into())]
                };
                ctx.run_ui(
                    egui::RawInput {
                        events,
                        ..Default::default()
                    },
                    |ui| {
                        let response = if multiline {
                            render_multiline(ui, &mut text, 4, style.clone())
                        } else {
                            render(ui, &mut text, style.clone())
                        };
                        assert_eq!(response.enabled(), !disabled);
                        assert_eq!(response.sense.is_focusable(), !disabled);
                        if !disabled {
                            response.request_focus();
                        }
                    },
                )
                .textures_delta
                .clear();
                assert_eq!(text, expected);
            }
        }
    }

    #[test]
    fn singleline_accepts_native_ime_composition() {
        let ctx = egui::Context::default();
        let theme =
            crate::theme::UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
                .unwrap();
        let style = Style::new_outline_md(&theme.light).width(Some(200.0));
        let mut text = String::new();

        for events in [
            vec![],
            vec![],
            vec![egui::Event::Ime(egui::ImeEvent::Preedit {
                text: "に".into(),
                active_range_chars: None,
            })],
            vec![egui::Event::Ime(egui::ImeEvent::Commit("日本".into()))],
        ] {
            ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    let response = render(ui, &mut text, style.clone());
                    response.request_focus();
                    assert_eq!(response.rect.size(), egui::vec2(200.0, style.height));
                },
            )
            .textures_delta
            .clear();
        }
        assert_eq!(text, "日本");
    }

    #[test]
    fn multiline_placeholder_uses_editor_font_for_cjk() {
        let ctx = egui::Context::default();
        let mut fonts = egui::FontDefinitions::default();
        let cjk = include_bytes!("../../examples/assets/font-fixtures/GalleryCJK-Test.ttc");
        let mut font = egui::FontData::from_static(cjk);
        font.index = 1;
        let font_name = "test-cjk".to_owned();
        fonts.font_data.insert(font_name.clone(), font.into());
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .push(font_name);
        ctx.set_fonts(fonts);

        let theme =
            crate::theme::UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
                .unwrap();
        let style = Style::new_outline_md(&theme.light).placeholder("中");
        let expected = egui::FontId::proportional(style.text_size);
        let mut output = ctx.run_ui(Default::default(), |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(20.0));
            assert!(ui.fonts_mut(|fonts| fonts.has_glyph(&expected, '中')));
            render_multiline(ui, &mut String::new(), 4, style.clone());
        });
        output.textures_delta.clear();

        let hint = output.shapes.iter().find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.text() == "中" => Some(&text.galley),
            _ => None,
        });
        assert_eq!(hint.unwrap().job.sections[0].format.font_id, expected);
    }

    #[test]
    fn password_tab_order_is_input_then_toggle_and_pointer_keeps_input_focus() {
        use egui::{Event, Key, Modifiers, PointerButton};
        let ctx = egui::Context::default();
        let theme =
            crate::theme::UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
                .unwrap();
        let password = Style::new_outline_md(&theme.light)
            .width(Some(200.0))
            .input_type(InputType::Password);
        let plain = Style::new_outline_md(&theme.light).width(Some(200.0));
        let (mut secret, mut next) = (String::new(), String::new());
        let tab = || Event::Key {
            key: Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        // Returns (password id, next id, focused id, password rect).
        let mut run = |events: Vec<Event>, focus_password: bool| {
            let mut out = None;
            ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    let a = render(ui, &mut secret, password.clone());
                    let b = render(ui, &mut next, plain.clone());
                    if focus_password {
                        a.request_focus();
                    }
                    out = Some((a.id, b.id, a.rect));
                },
            )
            .textures_delta
            .clear();
            let (a, b, rect) = out.unwrap();
            (a, b, ctx.memory(|m| m.focused()), rect)
        };
        let (a, _, focused, rect) = run(vec![], true);
        assert_eq!(focused, Some(a));
        let (a, b, focused, _) = run(vec![tab()], false);
        let toggle = focused.expect("Tab from the input lands on the toggle");
        assert!(toggle != a && toggle != b);
        let (_, b, focused, _) = run(vec![tab()], false);
        assert_eq!(
            focused,
            Some(b),
            "Tab from the toggle goes to the next input"
        );

        // A pointer click on the toggle reveals the password and keeps input focus.
        let pos = egui::pos2(rect.right() - 16.0, rect.center().y);
        let button = |pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        for events in [
            vec![Event::PointerMoved(pos)],
            vec![button(true)],
            vec![button(false)],
            vec![],
        ] {
            run(events, false);
        }
        let (a, _, focused, _) = run(vec![], false);
        assert_eq!(focused, Some(a));
        assert!(ctx.data(|d| d.get_temp::<bool>(a.with("password_visible"))) == Some(true));
    }
}
