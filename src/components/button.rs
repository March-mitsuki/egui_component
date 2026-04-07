use egui::{Color32, CornerRadius, Stroke, StrokeKind, Ui};

use crate::theme::{ButtonPalette, ButtonSize, ButtonVariant, Color, Theme};

pub struct Style {
    pub padding_x: f32,
    pub height: f32,
    pub corner_radius: CornerRadius,
    pub text_size: f32,
    pub palette: ButtonPalette,
    pub icon_left: ButtonIcon,
    pub icon_right: ButtonIcon,
    /// 图标和文字之间的间距
    pub spacing: f32,
    /// 是否强制固定宽度
    pub width: Option<f32>,
}

#[derive(Clone)]
pub enum ButtonIcon {
    None,
    Some(egui::ImageSource<'static>),
}

pub const BUTTON_HEIGHT_XS: f32 = 18.0;
pub const BUTTON_HEIGHT_SM: f32 = 24.0;
pub const BUTTON_HEIGHT_MD: f32 = 32.0;
pub const BUTTON_HEIGHT_LG: f32 = 40.0;

impl Style {
    /// color 为 None 时，使用 theme.fg （黑/白）作为按钮颜色
    pub fn new(
        theme: &Theme,
        color: Option<Color>,
        size: ButtonSize,
        variant: ButtonVariant,
        icon_left: ButtonIcon,
        icon_right: ButtonIcon,
    ) -> Self {
        let (height, padding_x, text_size, corner_radius, spacing) = match size {
            ButtonSize::Xs => (
                BUTTON_HEIGHT_XS,
                8.0,
                10.0,
                CornerRadius::same(theme.corner_radius.xxs as u8),
                4.0,
            ),
            ButtonSize::Sm => (
                BUTTON_HEIGHT_SM,
                10.0,
                12.0,
                CornerRadius::same(theme.corner_radius.xs as u8),
                6.0,
            ),
            ButtonSize::Md => (
                BUTTON_HEIGHT_MD,
                14.0,
                14.0,
                CornerRadius::same(theme.corner_radius.md as u8),
                6.0,
            ),
            ButtonSize::Lg => (
                BUTTON_HEIGHT_LG,
                18.0,
                16.0,
                CornerRadius::same(theme.corner_radius.md as u8),
                8.0,
            ),
        };
        let palette = match_button_palette(theme, color, variant);
        Self {
            height,
            padding_x,
            text_size,
            corner_radius,
            palette,
            icon_left,
            icon_right,
            spacing,
            width: None,
        }
    }

    pub fn new_solid_xs(theme: &Theme) -> Self {
        Self::new(
            theme,
            None,
            ButtonSize::Xs,
            ButtonVariant::Solid,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_solid_xs_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            ButtonSize::Xs,
            ButtonVariant::Solid,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_solid_sm(theme: &Theme) -> Self {
        Self::new(
            theme,
            None,
            ButtonSize::Sm,
            ButtonVariant::Solid,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_solid_sm_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            ButtonSize::Sm,
            ButtonVariant::Solid,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_solid_md(theme: &Theme) -> Self {
        Self::new(
            theme,
            None,
            ButtonSize::Md,
            ButtonVariant::Solid,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_solid_md_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            ButtonSize::Md,
            ButtonVariant::Solid,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_solid_lg(theme: &Theme) -> Self {
        Self::new(
            theme,
            None,
            ButtonSize::Lg,
            ButtonVariant::Solid,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_solid_lg_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            ButtonSize::Lg,
            ButtonVariant::Solid,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_surface_xs(theme: &Theme) -> Self {
        Self::new(
            theme,
            None,
            ButtonSize::Xs,
            ButtonVariant::Surface,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_surface_xs_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            ButtonSize::Xs,
            ButtonVariant::Surface,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_surface_sm(theme: &Theme) -> Self {
        Self::new(
            theme,
            None,
            ButtonSize::Sm,
            ButtonVariant::Surface,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_surface_sm_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            ButtonSize::Sm,
            ButtonVariant::Surface,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_surface_md(theme: &Theme) -> Self {
        Self::new(
            theme,
            None,
            ButtonSize::Md,
            ButtonVariant::Surface,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_surface_md_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            ButtonSize::Md,
            ButtonVariant::Surface,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_surface_lg(theme: &Theme) -> Self {
        Self::new(
            theme,
            None,
            ButtonSize::Lg,
            ButtonVariant::Surface,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_surface_lg_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            ButtonSize::Lg,
            ButtonVariant::Surface,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_outline_xs(theme: &Theme) -> Self {
        Self::new(
            theme,
            None,
            ButtonSize::Xs,
            ButtonVariant::Outline,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_outline_xs_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            ButtonSize::Xs,
            ButtonVariant::Outline,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_outline_sm(theme: &Theme) -> Self {
        Self::new(
            theme,
            None,
            ButtonSize::Sm,
            ButtonVariant::Outline,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_outline_sm_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            ButtonSize::Sm,
            ButtonVariant::Outline,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_outline_md(theme: &Theme) -> Self {
        Self::new(
            theme,
            None,
            ButtonSize::Md,
            ButtonVariant::Outline,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_outline_md_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            ButtonSize::Md,
            ButtonVariant::Outline,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_outline_lg(theme: &Theme) -> Self {
        Self::new(
            theme,
            None,
            ButtonSize::Lg,
            ButtonVariant::Outline,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_outline_lg_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            ButtonSize::Lg,
            ButtonVariant::Outline,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_ghost_xs(theme: &Theme) -> Self {
        Self::new(
            theme,
            None,
            ButtonSize::Xs,
            ButtonVariant::Ghost,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_ghost_xs_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            ButtonSize::Xs,
            ButtonVariant::Ghost,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_ghost_sm(theme: &Theme) -> Self {
        Self::new(
            theme,
            None,
            ButtonSize::Sm,
            ButtonVariant::Ghost,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_ghost_sm_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            ButtonSize::Sm,
            ButtonVariant::Ghost,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_ghost_md(theme: &Theme) -> Self {
        Self::new(
            theme,
            None,
            ButtonSize::Md,
            ButtonVariant::Ghost,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_ghost_md_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            ButtonSize::Md,
            ButtonVariant::Ghost,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_ghost_lg(theme: &Theme) -> Self {
        Self::new(
            theme,
            None,
            ButtonSize::Lg,
            ButtonVariant::Ghost,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

    pub fn new_ghost_lg_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            ButtonSize::Lg,
            ButtonVariant::Ghost,
            ButtonIcon::None,
            ButtonIcon::None,
        )
    }

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

    pub fn palette(mut self, palette: ButtonPalette) -> Self {
        self.palette = palette;
        self
    }

    pub fn icon_left(mut self, icon_left: ButtonIcon) -> Self {
        self.icon_left = icon_left;
        self
    }

    pub fn icon_right(mut self, icon_right: ButtonIcon) -> Self {
        self.icon_right = icon_right;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set the width of the button. If `None`, the button will automatically
    /// adjust its width to fit its content.
    pub fn width(mut self, width: Option<f32>) -> Self {
        self.width = width;
        self
    }
}

pub fn render(ui: &mut Ui, text: &str, style: Style) -> egui::Response {
    let mut font_id = egui::TextStyle::Button.resolve(ui.style());
    font_id.size = style.text_size;

    let icon_size = (style.text_size * 1.3).round();
    let has_text = !text.is_empty();

    // 计算图标占用的宽度
    let mut icon_width = 0.0_f32;
    if let ButtonIcon::Some(_) = &style.icon_left {
        icon_width += icon_size;
        if has_text {
            icon_width += style.spacing;
        }
    }
    if let ButtonIcon::Some(_) = &style.icon_right {
        icon_width += icon_size;
        if has_text || matches!(style.icon_left, ButtonIcon::Some(_)) {
            icon_width += style.spacing;
        }
    }

    // 固定宽度模式下限制文字可用宽度，否则不限制（自动撑开）
    let max_text_width = style
        .width
        .map(|w| (w - style.padding_x * 2.0 - icon_width).max(0.0))
        .unwrap_or(f32::INFINITY);

    let galley = {
        let mut font_id = egui::TextStyle::Button.resolve(ui.style());
        font_id.size = style.text_size;

        // 先不限宽排一次，看是否超出
        let full_galley = ui.painter().layout_no_wrap(
            text.to_string(),
            font_id.clone(),
            style.palette.text_color,
        );

        if max_text_width == f32::INFINITY || full_galley.size().x <= max_text_width {
            // 没超出，直接用
            full_galley
        } else {
            // 超出了，逐步截断加省略号
            let ellipsis = "…";
            let ellipsis_galley = ui.painter().layout_no_wrap(
                ellipsis.to_string(),
                font_id.clone(),
                style.palette.text_color,
            );
            let available = (max_text_width - ellipsis_galley.size().x).max(0.0);

            // 逐字符二分找到最长能放下的前缀
            let chars: Vec<char> = text.chars().collect();
            let mut lo = 0usize;
            let mut hi = chars.len();
            while lo < hi {
                let mid = (lo + hi + 1) / 2;
                let candidate: String = chars[..mid].iter().collect();
                let g = ui.painter().layout_no_wrap(
                    candidate,
                    font_id.clone(),
                    style.palette.text_color,
                );
                if g.size().x <= available {
                    lo = mid;
                } else {
                    hi = mid - 1;
                }
            }

            let truncated: String = chars[..lo].iter().collect::<String>() + ellipsis;
            ui.painter()
                .layout_no_wrap(truncated, font_id, style.palette.text_color)
        }
    };

    // 用截断后的实际宽度计算内容总宽度
    let mut content_width = if has_text { galley.size().x } else { 0.0 };
    if let ButtonIcon::Some(_) = &style.icon_left {
        content_width += icon_size;
        if has_text {
            content_width += style.spacing;
        }
    }
    if let ButtonIcon::Some(_) = &style.icon_right {
        content_width += icon_size;
        if has_text || matches!(style.icon_left, ButtonIcon::Some(_)) {
            content_width += style.spacing;
        }
    }

    let total_width = style.width.unwrap_or(content_width + style.padding_x * 2.0);
    let size = egui::vec2(total_width, style.height);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    // 绘制背景 (0.2s 渐变)
    let is_hovered = response.hovered();
    let animation_factor = ui
        .ctx()
        .animate_bool_with_time(response.id, is_hovered, 0.2);

    let fill_color = if response.is_pointer_button_down_on() {
        style.palette.active_bg
    } else {
        let base_rgba = egui::Rgba::from(style.palette.normal_bg);
        let hover_rgba = egui::Rgba::from(style.palette.hover_bg);
        let mixed_rgba = base_rgba * (1.0 - animation_factor) + hover_rgba * animation_factor;
        Color32::from(mixed_rgba)
    };

    ui.painter().rect(
        rect,
        style.corner_radius,
        fill_color,
        style.palette.stroke,
        StrokeKind::Outside,
    );

    // 绘制内容（图标和文字水平居中排列）
    let mut cursor_x = rect.center().x - content_width / 2.0;

    // 左侧图标
    if let ButtonIcon::Some(icon) = style.icon_left.clone() {
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(cursor_x + icon_size / 2.0, rect.center().y),
            egui::vec2(icon_size, icon_size),
        );
        egui::Image::new(icon)
            .tint(style.palette.text_color)
            .paint_at(ui, icon_rect);
        cursor_x += icon_size;
        if has_text {
            cursor_x += style.spacing;
        }
    }

    // 文字
    if has_text {
        let text_pos = egui::pos2(cursor_x, rect.center().y - galley.size().y / 2.0);
        ui.painter()
            .galley(text_pos, galley.clone(), style.palette.text_color);
        cursor_x += galley.size().x;
    }

    // 右侧图标
    if let ButtonIcon::Some(icon) = style.icon_right {
        if has_text || matches!(style.icon_left, ButtonIcon::Some(_)) {
            cursor_x += style.spacing;
        }
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(cursor_x + icon_size / 2.0, rect.center().y),
            egui::vec2(icon_size, icon_size),
        );
        egui::Image::new(icon)
            .tint(style.palette.text_color)
            .paint_at(ui, icon_rect);
    }

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn match_button_palette(
    theme: &Theme,
    color: Option<Color>,
    variant: ButtonVariant,
) -> ButtonPalette {
    match color {
        Some(color) => color.to_button_palette(theme, variant),
        None => match variant {
            ButtonVariant::Solid => ButtonPalette {
                normal_bg: theme.colors.fg.base,
                hover_bg: theme.colors.fg.base.gamma_multiply(0.8),
                active_bg: theme.colors.fg.base.gamma_multiply(1.2),
                text_color: theme.colors.bg.base,
                stroke: Stroke::NONE,
            },
            ButtonVariant::Surface => ButtonPalette {
                normal_bg: theme.colors.fg.base.gamma_multiply(0.7),
                hover_bg: theme.colors.fg.base.gamma_multiply(0.6),
                active_bg: theme.colors.fg.base.gamma_multiply(0.8),
                text_color: theme.colors.bg.base,
                stroke: Stroke::new(1.0, theme.colors.fg.base.gamma_multiply(0.6)),
            },
            ButtonVariant::Outline => ButtonPalette {
                normal_bg: Color32::TRANSPARENT,
                hover_bg: theme.colors.fg.base.gamma_multiply(0.1),
                active_bg: theme.colors.fg.base.gamma_multiply(0.2),
                text_color: theme.colors.fg.base,
                stroke: Stroke::new(1.0, theme.colors.fg.base.gamma_multiply(0.4)),
            },
            ButtonVariant::Ghost => ButtonPalette {
                normal_bg: Color32::TRANSPARENT,
                hover_bg: theme.colors.fg.base.gamma_multiply(0.2),
                active_bg: theme.colors.fg.base.gamma_multiply(0.4),
                text_color: theme.colors.fg.base,
                stroke: Stroke::NONE,
            },
        },
    }
}
