use egui::{Color32, CornerRadius, Image, Popup, Rect, Rgba, Sense, StrokeKind, Ui, vec2};
use egui_component_style_macros::FrameModifier;

use crate::components::{icon_button, text, tooltip};
use crate::icon::ICONS;
use crate::theme::Theme;

pub struct Style {
    pub button: icon_button::Style,
    pub menu: MenuStyle,
    pub gap: f32,
    pub menu_handle_width: f32,
    pub menu_handle_icon_size: f32,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            button: icon_button::Style::new_ghost_lg(theme).corner_radius(CornerRadius::same(0)),
            menu: MenuStyle::new(theme),
            gap: 0.0,
            menu_handle_width: 12.0,
            menu_handle_icon_size: 12.0,
        }
    }
}

#[derive(FrameModifier, Clone)]
pub struct MenuStyle {
    pub align: egui::RectAlign,
    pub gap: f32,
    pub close_behavior: egui::PopupCloseBehavior,
    pub frame: egui::Frame,
}
impl MenuStyle {
    pub fn new(theme: &Theme) -> Self {
        Self {
            align: egui::RectAlign::BOTTOM_START,
            gap: 0.0,
            close_behavior: egui::PopupCloseBehavior::CloseOnClick,
            frame: egui::Frame::default()
                .fill(theme.colors.bg.accent)
                .stroke(egui::Stroke::NONE)
                .inner_margin(egui::Margin::same(0))
                .shadow(egui::Shadow {
                    offset: [0, 0],
                    blur: 8,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(60),
                })
                .corner_radius(egui::CornerRadius::same(0)),
        }
    }

    pub fn align(mut self, align: egui::RectAlign) -> Self {
        self.align = align;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn close_behavior(mut self, close_behavior: egui::PopupCloseBehavior) -> Self {
        self.close_behavior = close_behavior;
        self
    }
}

pub struct IconWithTooltip<'a> {
    pub icon: egui::ImageSource<'a>,
    pub tooltip: String,
}

pub fn render<'a>(
    ui: &mut Ui,
    id: egui::Id,
    icons: Vec<IconWithTooltip<'a>>,
    mut on_click: impl FnMut(usize),
    theme: &Theme,
    style: Style,
) {
    let total_width = style.button.size + style.gap + style.menu_handle_width;
    let size = vec2(total_width, style.button.size);

    // 1. Allocate space for the entire component
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    // 2. Define sub-rectangles for label and menu handle
    let label_rect = Rect::from_min_size(rect.min, vec2(style.button.size, style.button.size));
    let menu_handle_rect = Rect::from_min_size(
        rect.min + vec2(style.button.size + style.gap, 0.0),
        vec2(style.menu_handle_width, style.button.size),
    );

    // 3. Interactions
    // Create an interaction for the menu handle portion specifically
    let menu_handle_response = ui.interact(
        menu_handle_rect,
        response.id.with("menu_handle"),
        Sense::click(),
    );

    // 4. Hover logic and animations
    // Combine hover states so that hovering the handle also triggers the overall background effect
    let is_hovered = response.hovered() || menu_handle_response.hovered();
    let bg_animation = ui
        .ctx()
        .animate_bool_with_time(response.id, is_hovered, 0.2);

    let menu_handle_hovered = menu_handle_response.hovered();
    let menu_icon_animation =
        ui.ctx()
            .animate_bool_with_time(response.id.with("menu_icon"), menu_handle_hovered, 0.15);

    // 5. Background color calculation
    let is_active =
        response.is_pointer_button_down_on() || menu_handle_response.is_pointer_button_down_on();
    let fill_color = if is_active {
        style.button.palette.active_bg
    } else {
        let base_rgba = Rgba::from(style.button.palette.normal_bg);
        let hover_rgba = Rgba::from(style.button.palette.hover_bg);
        let mixed_rgba = base_rgba * (1.0 - bg_animation) + hover_rgba * bg_animation;
        Color32::from(mixed_rgba)
    };

    // 6. Draw the common background
    ui.painter().rect(
        rect,
        style.button.corner_radius,
        fill_color,
        style.button.palette.stroke,
        StrokeKind::Outside,
    );

    // 7. Draw label icon
    if let Some(icon_with_tooltip) = icons.first() {
        let icon_size = vec2(style.button.icon_size, style.button.icon_size);
        let icon_rect = Rect::from_center_size(label_rect.center(), icon_size);
        Image::new(icon_with_tooltip.icon.clone())
            .tint(style.button.palette.text_color)
            .paint_at(ui, icon_rect);

        tooltip::render(
            &response,
            |ui| {
                text::render(ui, &icon_with_tooltip.tooltip, text::Style::new(theme));
            },
            &tooltip::Style::new(theme),
        );
    }

    // 8. Draw menu handle icon (arrow_down) with positional offset on hover
    let menu_icon_size = vec2(style.menu_handle_icon_size, style.menu_handle_icon_size);
    let arrow_offset = menu_icon_animation * 4.0;
    let menu_icon_rect = Rect::from_center_size(
        menu_handle_rect.center() + vec2(0.0, arrow_offset),
        menu_icon_size,
    );
    Image::new(ICONS.arrow_down.clone())
        .tint(style.button.palette.text_color)
        .paint_at(ui, menu_icon_rect);

    // 9. Handle interactions
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            if label_rect.contains(pos) {
                on_click(0);
            }
        }
    }

    // 10. Popup Menu logic
    // The menu_handle_response created earlier is used to trigger the popup
    Popup::menu(&menu_handle_response)
        .id(id)
        .frame(style.menu.frame)
        .align(style.menu.align)
        .close_behavior(style.menu.close_behavior)
        .gap(style.menu.gap)
        .show(|ui| {
            for (index, icon_with_tooltip) in icons.iter().skip(1).enumerate() {
                let btn_resp =
                    icon_button::render(ui, icon_with_tooltip.icon.clone(), style.button.clone());
                tooltip::render(
                    &btn_resp,
                    |ui| {
                        text::render(ui, &icon_with_tooltip.tooltip, text::Style::new(theme));
                    },
                    &tooltip::Style::new(theme),
                );
                if btn_resp.clicked() {
                    on_click(index + 1);
                }
            }
        });

    // Ensure the pointing hand cursor appears for both the component and its handle
    response.on_hover_cursor(egui::CursorIcon::PointingHand);
    menu_handle_response.on_hover_cursor(egui::CursorIcon::PointingHand);
}
