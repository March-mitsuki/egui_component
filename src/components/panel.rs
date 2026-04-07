use egui::{CornerRadius, Margin, Shadow, Stroke, Ui};
// use style_macros::FrameModifier;

use crate::theme::Theme;

// #[derive(FrameModifier)]
pub struct Style {
    pub frame: egui::Frame,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        let card_frame = egui::Frame {
            inner_margin: Margin::same(8),
            outer_margin: Margin::same(0),
            corner_radius: CornerRadius::same(4),
            shadow: Shadow {
                offset: [0, 0],
                blur: 20,
                spread: 2,
                color: theme.colors.gray.l6,
            },
            fill: theme.colors.bg.base,
            stroke: Stroke::new(1.0, theme.colors.gray.border()),
        };
        Self { frame: card_frame }
    }
}

pub fn render(ui: &mut Ui, renderer: impl FnOnce(&mut Ui), style: Style) {
    style.frame.show(ui, |ui| {
        renderer(ui);
    });
}
