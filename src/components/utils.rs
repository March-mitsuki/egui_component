use egui::{Color32, Context, CornerRadius, Frame, Margin, Shadow, Stroke, Ui};
use egui_component_style_macros::FrameModifier;

use crate::theme::{CardSize, Theme};

#[derive(FrameModifier)]
pub struct CardFrame {
    pub frame: Frame,
}
impl From<CardFrame> for Frame {
    fn from(card_frame: CardFrame) -> Self {
        card_frame.frame
    }
}
impl CardFrame {
    pub fn new(theme: &Theme, size: CardSize) -> Self {
        let margin = match size {
            CardSize::Sm => Margin::same(8),
            CardSize::Md => Margin::same(16),
            CardSize::Lg => Margin::same(24),
        };
        let corner_radius = match size {
            CardSize::Sm => CornerRadius::same(theme.corner_radius.sm as u8),
            CardSize::Md => CornerRadius::same(theme.corner_radius.md as u8),
            CardSize::Lg => CornerRadius::same(theme.corner_radius.lg as u8),
        };

        Self {
            frame: Frame::new()
                .inner_margin(margin)
                .outer_margin(Margin::same(0))
                .corner_radius(corner_radius)
                .shadow(Shadow {
                    offset: [0, 0],
                    blur: 20,
                    spread: 2,
                    color: Color32::from_black_alpha(60),
                })
                .fill(theme.colors.bg.base)
                .stroke(Stroke::new(1.0, theme.colors.gray.border())),
        }
    }

    pub fn show(&self, ui: &mut Ui, renderer: impl FnOnce(&mut Ui)) {
        self.frame.show(ui, renderer);
    }
}

/// change style for current ui
///
/// ### Returns
///
/// the style before change. can be used to restore style
pub fn change_style(ui: &mut Ui, change_fn: impl FnOnce(&mut egui::Style)) -> egui::Style {
    let style_origin = (**ui.style()).clone();
    let mut style = style_origin.clone();
    change_fn(&mut style);
    ui.set_style(style);
    style_origin
}

/// mutate style for global context
pub fn mutate_style(ctx: &Context, change_fn: impl FnOnce(&mut egui::Style)) {
    ctx.global_style_mut(change_fn);
}
