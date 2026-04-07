use egui::{Color32, CornerRadius, Image, Rect, Rgba, Sense, StrokeKind, Ui, vec2};

use crate::theme::{ButtonPalette, ButtonSize, ButtonVariant, Color, Theme};

use super::button::match_button_palette;
use super::button::{BUTTON_HEIGHT_LG, BUTTON_HEIGHT_MD, BUTTON_HEIGHT_SM, BUTTON_HEIGHT_XS};

pub struct Style {
    pub size: f32,
    pub icon_size: f32,
    pub corner_radius: CornerRadius,
    pub palette: ButtonPalette,
}

impl Style {
    pub fn new(theme: &Theme, color: Option<Color>, size: ButtonSize, variant: ButtonVariant) -> Self {
        let (size, icon_size, corner_radius) = match size {
            ButtonSize::Xs => (BUTTON_HEIGHT_XS, 10.0, CornerRadius::same(theme.corner_radius.xs as u8)),
            ButtonSize::Sm => (BUTTON_HEIGHT_SM, 14.0, CornerRadius::same(theme.corner_radius.sm as u8)),
            ButtonSize::Md => (BUTTON_HEIGHT_MD, 20.0, CornerRadius::same(theme.corner_radius.sm as u8)),
            ButtonSize::Lg => (BUTTON_HEIGHT_LG, 26.0, CornerRadius::same(theme.corner_radius.md as u8)),
        };
        let palette = match_button_palette(theme, color, variant);
        Self { size, icon_size, corner_radius, palette }
    }

    pub fn new_solid_xs(theme: &Theme) -> Self {
        Self::new(theme, None, ButtonSize::Xs, ButtonVariant::Solid)
    }

    pub fn new_solid_xs_color(theme: &Theme, color: Color) -> Self {
        Self::new(theme, Some(color), ButtonSize::Xs, ButtonVariant::Solid)
    }

    pub fn new_solid_sm(theme: &Theme) -> Self {
        Self::new(theme, None, ButtonSize::Sm, ButtonVariant::Solid)
    }

    pub fn new_solid_sm_color(theme: &Theme, color: Color) -> Self {
        Self::new(theme, Some(color), ButtonSize::Sm, ButtonVariant::Solid)
    }

    pub fn new_solid_md(theme: &Theme) -> Self {
        Self::new(theme, None, ButtonSize::Md, ButtonVariant::Solid)
    }

    pub fn new_solid_md_color(theme: &Theme, color: Color) -> Self {
        Self::new(theme, Some(color), ButtonSize::Md, ButtonVariant::Solid)
    }

    pub fn new_solid_lg(theme: &Theme) -> Self {
        Self::new(theme, None, ButtonSize::Lg, ButtonVariant::Solid)
    }

    pub fn new_solid_lg_color(theme: &Theme, color: Color) -> Self {
        Self::new(theme, Some(color), ButtonSize::Lg, ButtonVariant::Solid)
    }

    pub fn new_surface_xs(theme: &Theme) -> Self {
        Self::new(theme, None, ButtonSize::Xs, ButtonVariant::Surface)
    }

    pub fn new_surface_xs_color(theme: &Theme, color: Color) -> Self {
        Self::new(theme, Some(color), ButtonSize::Xs, ButtonVariant::Surface)
    }

    pub fn new_surface_sm(theme: &Theme) -> Self {
        Self::new(theme, None, ButtonSize::Sm, ButtonVariant::Surface)
    }

    pub fn new_surface_sm_color(theme: &Theme, color: Color) -> Self {
        Self::new(theme, Some(color), ButtonSize::Sm, ButtonVariant::Surface)
    }

    pub fn new_surface_md(theme: &Theme) -> Self {
        Self::new(theme, None, ButtonSize::Md, ButtonVariant::Surface)
    }

    pub fn new_surface_md_color(theme: &Theme, color: Color) -> Self {
        Self::new(theme, Some(color), ButtonSize::Md, ButtonVariant::Surface)
    }

    pub fn new_surface_lg(theme: &Theme) -> Self {
        Self::new(theme, None, ButtonSize::Lg, ButtonVariant::Surface)
    }

    pub fn new_surface_lg_color(theme: &Theme, color: Color) -> Self {
        Self::new(theme, Some(color), ButtonSize::Lg, ButtonVariant::Surface)
    }

    pub fn new_outline_xs(theme: &Theme) -> Self {
        Self::new(theme, None, ButtonSize::Xs, ButtonVariant::Outline)
    }

    pub fn new_outline_xs_color(theme: &Theme, color: Color) -> Self {
        Self::new(theme, Some(color), ButtonSize::Xs, ButtonVariant::Outline)
    }

    pub fn new_outline_sm(theme: &Theme) -> Self {
        Self::new(theme, None, ButtonSize::Sm, ButtonVariant::Outline)
    }

    pub fn new_outline_sm_color(theme: &Theme, color: Color) -> Self {
        Self::new(theme, Some(color), ButtonSize::Sm, ButtonVariant::Outline)
    }

    pub fn new_outline_md(theme: &Theme) -> Self {
        Self::new(theme, None, ButtonSize::Md, ButtonVariant::Outline)
    }

    pub fn new_outline_md_color(theme: &Theme, color: Color) -> Self {
        Self::new(theme, Some(color), ButtonSize::Md, ButtonVariant::Outline)
    }

    pub fn new_outline_lg(theme: &Theme) -> Self {
        Self::new(theme, None, ButtonSize::Lg, ButtonVariant::Outline)
    }

    pub fn new_outline_lg_color(theme: &Theme, color: Color) -> Self {
        Self::new(theme, Some(color), ButtonSize::Lg, ButtonVariant::Outline)
    }

    pub fn new_ghost_xs(theme: &Theme) -> Self {
        Self::new(theme, None, ButtonSize::Xs, ButtonVariant::Ghost)
    }

    pub fn new_ghost_xs_color(theme: &Theme, color: Color) -> Self {
        Self::new(theme, Some(color), ButtonSize::Xs, ButtonVariant::Ghost)
    }

    pub fn new_ghost_sm(theme: &Theme) -> Self {
        Self::new(theme, None, ButtonSize::Sm, ButtonVariant::Ghost)
    }

    pub fn new_ghost_sm_color(theme: &Theme, color: Color) -> Self {
        Self::new(theme, Some(color), ButtonSize::Sm, ButtonVariant::Ghost)
    }

    pub fn new_ghost_md(theme: &Theme) -> Self {
        Self::new(theme, None, ButtonSize::Md, ButtonVariant::Ghost)
    }

    pub fn new_ghost_md_color(theme: &Theme, color: Color) -> Self {
        Self::new(theme, Some(color), ButtonSize::Md, ButtonVariant::Ghost)
    }

    pub fn new_ghost_lg(theme: &Theme) -> Self {
        Self::new(theme, None, ButtonSize::Lg, ButtonVariant::Ghost)
    }

    pub fn new_ghost_lg_color(theme: &Theme, color: Color) -> Self {
        Self::new(theme, Some(color), ButtonSize::Lg, ButtonVariant::Ghost)
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn icon_size(mut self, icon_size: f32) -> Self {
        self.icon_size = icon_size;
        self
    }

    pub fn corner_radius(mut self, corner_radius: CornerRadius) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    pub fn palette(mut self, palette: ButtonPalette) -> Self {
        self.palette = palette;
        self
    }
}

pub fn render(ui: &mut Ui, icon: egui::ImageSource, style: Style) -> egui::Response {
    // 预留正方形空间
    let size = vec2(style.size, style.size);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    // 绘制背景 (0.2s 渐变)
    let is_hovered = response.hovered();
    let animation_factor = ui.ctx().animate_bool_with_time(response.id, is_hovered, 0.2);

    let fill_color = if response.is_pointer_button_down_on() {
        style.palette.active_bg
    } else {
        let base_rgba = Rgba::from(style.palette.normal_bg);
        let hover_rgba = Rgba::from(style.palette.hover_bg);
        let mixed_rgba = base_rgba * (1.0 - animation_factor) + hover_rgba * animation_factor;
        Color32::from(mixed_rgba)
    };

    ui.painter().rect(rect, style.corner_radius, fill_color, style.palette.stroke, StrokeKind::Outside);

    // 居中绘制图标
    let icon_size = vec2(style.icon_size, style.icon_size);
    let icon_rect = Rect::from_center_size(rect.center(), icon_size);
    Image::new(icon).tint(style.palette.text_color).paint_at(ui, icon_rect);

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
