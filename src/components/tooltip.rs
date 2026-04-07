use egui::{Response, Tooltip, Ui};
// use style_macros::FrameModifier;

use crate::theme::Theme;

// #[derive(FrameModifier)]
pub struct Style {
    align: egui::RectAlign,
    gap: f32,
    close_behavior: egui::PopupCloseBehavior,
    frame: egui::Frame,
}

impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            align: egui::RectAlign::BOTTOM,
            gap: 4.0,
            close_behavior: egui::PopupCloseBehavior::CloseOnClick,
            frame: egui::Frame::default()
                .fill(theme.colors.bg.accent)
                .stroke(egui::Stroke {
                    width: 1.0,
                    color: theme.colors.gray.l2,
                })
                .inner_margin(egui::Margin::symmetric(16, 4))
                .shadow(egui::Shadow {
                    offset: [0, 0],
                    blur: 8,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(60),
                })
                .corner_radius(egui::CornerRadius::same(theme.corner_radius.xs as u8)),
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

pub fn render(target: &Response, content: impl FnOnce(&mut Ui), style: &Style) {
    let mut tooltip = Tooltip::for_enabled(target);
    tooltip.popup = tooltip
        .popup
        .frame(style.frame)
        .align(style.align)
        .close_behavior(style.close_behavior)
        .gap(style.gap);
    tooltip.show(content);
}
