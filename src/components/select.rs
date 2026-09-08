use egui::{Color32, CornerRadius, Popup, Response, Sense, Stroke, Ui};

use crate::theme::{Theme, Color};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectSize {
    Xs,
    Sm,
    Md,
    Lg,
}

#[derive(Clone)]
pub struct Style {
    pub height: f32,
    pub padding_x: f32,
    pub corner_radius: CornerRadius,
    pub text_size: f32,
    pub icon_size: f32,
    pub gap: f32,
    pub width: Option<f32>,
    pub disabled: bool,
    pub is_invalid: bool,
    pub placeholder: String,

    // Palette / Colors
    pub normal_bg: Color32,
    pub hover_bg: Color32,
    pub active_bg: Color32,
    pub disabled_bg: Color32,

    pub text_color: Color32,
    pub placeholder_color: Color32,
    pub disabled_text_color: Color32,

    pub border_stroke: Stroke,
    pub hover_border_stroke: Stroke,
    pub active_border_stroke: Stroke,
    pub disabled_border_stroke: Stroke,

    // Invalid state strokes
    pub invalid_border_stroke: Stroke,
    pub invalid_hover_border_stroke: Stroke,
    pub invalid_active_border_stroke: Stroke,

    // Icon Settings
    pub arrow_icon_size: f32,
    pub arrow_icon_color: Color32,

    // Popup Settings
    pub popup_align: egui::RectAlign,
    pub popup_gap: f32,
    pub popup_close_behavior: egui::PopupCloseBehavior,
    pub popup_frame: egui::Frame,
    pub popup_max_height: f32,
}

impl Style {
    pub fn new(theme: &Theme, color: Option<Color>, size: SelectSize) -> Self {
        let (height, padding_x, text_size, radius, icon_size) = match size {
            SelectSize::Xs => (18.0, 8.0, 10.0, theme.corner_radius.xs as u8, 10.0),
            SelectSize::Sm => (24.0, 10.0, 12.0, theme.corner_radius.xs as u8, 12.0),
            SelectSize::Md => (32.0, 12.0, 14.0, theme.corner_radius.md as u8, 14.0),
            SelectSize::Lg => (40.0, 16.0, 16.0, theme.corner_radius.md as u8, 16.0),
        };

        let focus_color = match color {
            Some(c) => c.normal(),
            None => theme.colors.primary.normal(),
        };

        let active_border_stroke = Stroke::new(1.5, focus_color);

        Self {
            height,
            padding_x,
            corner_radius: CornerRadius::same(radius),
            text_size,
            icon_size,
            gap: 6.0,
            width: None,
            disabled: false,
            is_invalid: false,
            placeholder: "Select...".to_string(),

            // Palette
            normal_bg: theme.colors.fg.base.gamma_multiply(0.06),
            hover_bg: theme.colors.fg.base.gamma_multiply(0.1),
            active_bg: theme.colors.fg.base.gamma_multiply(0.06),
            disabled_bg: theme.colors.fg.base.gamma_multiply(0.03),

            text_color: theme.colors.fg.base,
            placeholder_color: theme.colors.fg.base.gamma_multiply(0.35),
            disabled_text_color: theme.colors.fg.base.gamma_multiply(0.3),

            border_stroke: Stroke::new(1.0, theme.colors.fg.base.gamma_multiply(0.15)),
            hover_border_stroke: Stroke::new(1.0, theme.colors.fg.base.gamma_multiply(0.25)),
            active_border_stroke,
            disabled_border_stroke: Stroke::new(1.0, theme.colors.fg.base.gamma_multiply(0.05)),

            // Invalid state strokes
            invalid_border_stroke: Stroke::new(1.0, theme.colors.red.normal()),
            invalid_hover_border_stroke: Stroke::new(1.0, theme.colors.red.hover()),
            invalid_active_border_stroke: Stroke::new(1.5, theme.colors.red.active()),

            // Icon
            arrow_icon_size: 12.0,
            arrow_icon_color: theme.colors.fg.base.gamma_multiply(0.6),

            // Popup
            popup_align: egui::RectAlign::BOTTOM_START,
            popup_gap: 4.0,
            popup_close_behavior: egui::PopupCloseBehavior::CloseOnClick,
            popup_frame: egui::Frame::default()
                .fill(theme.colors.bg.accent)
                .stroke(egui::Stroke {
                    width: 1.0,
                    color: theme.colors.gray.l2,
                })
                .inner_margin(egui::Margin::same(4))
                .shadow(egui::Shadow {
                    offset: [0, 0],
                    blur: 8,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(60),
                })
                .corner_radius(egui::CornerRadius::same(theme.corner_radius.sm as u8)),
            popup_max_height: 200.0,
        }
    }

    pub fn width(mut self, width: Option<f32>) -> Self {
        self.width = width;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn is_invalid(mut self, is_invalid: bool) -> Self {
        self.is_invalid = is_invalid;
        self
    }

    pub fn placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = placeholder.to_string();
        self
    }

    pub fn popup_max_height(mut self, max_height: f32) -> Self {
        self.popup_max_height = max_height;
        self
    }
}

pub fn render(
    ui: &mut Ui,
    id: egui::Id,
    selected_text: &str,
    style: Style,
    menu_render: impl FnOnce(&mut Ui),
) -> Response {
    // 1. Allocate exact size
    let width = style.width.unwrap_or(ui.available_width());
    let size = egui::vec2(width, style.height);
    let sense = if style.disabled {
        Sense::hover()
    } else {
        Sense::click()
    };
    let (rect, response) = ui.allocate_exact_size(size, sense);

    // 2. Query popup open state
    let is_open = egui::Popup::is_id_open(ui.ctx(), id);



    // 4. Animation factors
    let is_hovered = response.hovered() && !style.disabled;
    let hover_factor = ui.ctx().animate_bool_with_time(response.id.with("hover"), is_hovered, 0.15);
    let open_factor = ui.ctx().animate_bool_with_time(response.id.with("open"), is_open, 0.15);

    // 5. Draw background
    let bg_color = if style.disabled {
        style.disabled_bg
    } else if is_open {
        style.active_bg
    } else {
        let base_rgba = egui::Rgba::from(style.normal_bg);
        let hover_rgba = egui::Rgba::from(style.hover_bg);
        let mixed_rgba = base_rgba * (1.0 - hover_factor) + hover_rgba * hover_factor;
        Color32::from(mixed_rgba)
    };

    // 6. Draw border
    let stroke = if style.disabled {
        style.disabled_border_stroke
    } else if style.is_invalid {
        if is_open {
            style.invalid_active_border_stroke
        } else {
            let base_stroke = style.invalid_border_stroke;
            let hover_stroke = style.invalid_hover_border_stroke;
            let mixed_color = Color32::from(
                egui::Rgba::from(base_stroke.color) * (1.0 - hover_factor)
                    + egui::Rgba::from(hover_stroke.color) * hover_factor,
            );
            Stroke::new(
                base_stroke.width * (1.0 - hover_factor) + hover_stroke.width * hover_factor,
                mixed_color,
            )
        }
    } else if is_open {
        style.active_border_stroke
    } else {
        let base_stroke = style.border_stroke;
        let hover_stroke = style.hover_border_stroke;
        let mixed_color = Color32::from(
            egui::Rgba::from(base_stroke.color) * (1.0 - hover_factor)
                + egui::Rgba::from(hover_stroke.color) * hover_factor,
        );
        Stroke::new(
            base_stroke.width * (1.0 - hover_factor) + hover_stroke.width * hover_factor,
            mixed_color,
        )
    };

    ui.painter().rect(
        rect,
        style.corner_radius,
        bg_color,
        stroke,
        egui::StrokeKind::Outside,
    );

    // 7. Draw content (Selected text + arrow_down icon)
    let text_color = if style.disabled {
        style.disabled_text_color
    } else if selected_text.is_empty() {
        style.placeholder_color
    } else {
        style.text_color
    };

    let display_text = if selected_text.is_empty() {
        &style.placeholder
    } else {
        selected_text
    };

    // Layout text
    let font_family = crate::consts::FONT_MANAGER.default_egui_font_family();
    let font_id = egui::FontId::new(style.text_size, font_family);
    let galley = ui.painter().layout_no_wrap(display_text.to_string(), font_id, text_color);

    // Calculate text position (vertically centered, left aligned with padding)
    let text_pos = egui::pos2(
        rect.left() + style.padding_x,
        rect.center().y - galley.size().y / 2.0,
    );
    ui.painter().galley(text_pos, galley, text_color);

    // Draw Arrow Icon (aligned to right, with rotation animation when open)
    let arrow_color = if style.disabled {
        style.disabled_text_color
    } else {
        style.arrow_icon_color
    };

    let arrow_size = style.arrow_icon_size;
    let arrow_center = egui::pos2(
        rect.right() - style.padding_x - arrow_size / 2.0,
        rect.center().y,
    );

    // Rotate arrow down by 180 degrees (PI radians) when open
    let rotation_angle = open_factor * std::f32::consts::PI;
    let cos = rotation_angle.cos();
    let sin = rotation_angle.sin();
    let rotate_point = |p: egui::Vec2| {
        egui::vec2(
            p.x * cos - p.y * sin,
            p.x * sin + p.y * cos,
        )
    };

    let p0 = arrow_center + rotate_point(egui::vec2(-arrow_size / 2.0, -arrow_size / 4.0));
    let p1 = arrow_center + rotate_point(egui::vec2(0.0, arrow_size / 4.0));
    let p2 = arrow_center + rotate_point(egui::vec2(arrow_size / 2.0, -arrow_size / 4.0));

    let arrow_stroke = Stroke::new(1.5, arrow_color);
    ui.painter().line_segment([p0, p1], arrow_stroke);
    ui.painter().line_segment([p1, p2], arrow_stroke);

    // 8. Popup Menu logic
    Popup::menu(&response)
        .id(id)
        .frame(style.popup_frame)
        .align(style.popup_align)
        .close_behavior(style.popup_close_behavior)
        .gap(style.popup_gap)
        .show(|ui| {
            // Match select box width exactly (accounting for frame border/margin)
            let inner_width = (rect.width() - style.popup_frame.inner_margin.sum().x - style.popup_frame.stroke.width * 2.0).max(0.0);
            ui.set_width(inner_width);

            // Wrap items in ScrollArea
            egui::ScrollArea::vertical()
                .max_height(style.popup_max_height)
                .show(ui, menu_render);
        });

    if style.disabled {
        response.on_hover_cursor(egui::CursorIcon::NotAllowed)
    } else {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    }
}

pub fn render_item(
    ui: &mut Ui,
    text: &str,
    selected: bool,
    style: &Style,
    theme: &Theme,
) -> Response {
    let item_height = (style.height - 8.0).max(22.0);
    let size = egui::vec2(ui.available_width(), item_height);

    // Allocate space for option
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    let is_hovered = response.hovered();

    // Draw background
    let bg_color = if selected {
        // Highlight background for selected item
        style.active_border_stroke.color.gamma_multiply(0.12)
    } else if is_hovered {
        theme.colors.fg.base.gamma_multiply(0.08)
    } else {
        Color32::TRANSPARENT
    };

    if bg_color != Color32::TRANSPARENT {
        // Match popup corner radius but slightly smaller or just use sm
        let corner_radius = CornerRadius::same(theme.corner_radius.xs as u8);
        ui.painter().rect(rect, corner_radius, bg_color, Stroke::NONE, egui::StrokeKind::Outside);
    }

    // Draw text
    let text_color = if selected {
        style.active_border_stroke.color
    } else {
        style.text_color
    };

    let font_family = crate::consts::FONT_MANAGER.default_egui_font_family();
    let font_id = egui::FontId::new(style.text_size, font_family);
    let galley = ui.painter().layout_no_wrap(text.to_string(), font_id, text_color);

    let text_pos = egui::pos2(
        rect.left() + style.padding_x - 4.0, // align nicely
        rect.center().y - galley.size().y / 2.0,
    );
    ui.painter().galley(text_pos, galley, text_color);

    // Draw checkmark icon on the right if selected
    if selected {
        let checkmark_center = egui::pos2(
            rect.right() - style.padding_x,
            rect.center().y,
        );
        let check_stroke = Stroke::new(1.8, style.active_border_stroke.color);
        ui.painter().line_segment(
            [
                checkmark_center + egui::vec2(-4.0, 0.0),
                checkmark_center + egui::vec2(-1.0, 3.0),
            ],
            check_stroke,
        );
        ui.painter().line_segment(
            [
                checkmark_center + egui::vec2(-1.0, 3.0),
                checkmark_center + egui::vec2(5.0, -3.0),
            ],
            check_stroke,
        );
    }

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
