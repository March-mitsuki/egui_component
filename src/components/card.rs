use egui::Ui;
use egui_component_style_macros::FrameModifier;

use crate::theme::{CardSize, Theme};

#[derive(FrameModifier)]
pub struct Style {
    pub frame: egui::Frame,
}
impl Style {
    pub fn new(theme: &Theme, size: CardSize) -> Self {
        Self {
            frame: super::utils::CardFrame::new(theme, size).frame,
        }
    }

    pub fn new_sm(theme: &Theme) -> Self {
        Self::new(theme, CardSize::Sm)
    }

    pub fn new_md(theme: &Theme) -> Self {
        Self::new(theme, CardSize::Md)
    }

    pub fn new_lg(theme: &Theme) -> Self {
        Self::new(theme, CardSize::Lg)
    }
}

pub fn render(ui: &mut Ui, renderer: impl FnOnce(&mut Ui), style: Style) {
    style.frame.show(ui, |ui| {
        renderer(ui);
    });
}
