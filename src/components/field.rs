use egui::{Frame, InnerResponse, Ui};

use crate::theme::Theme;
use crate::components::text;

#[derive(Clone)]
pub struct Style {
    pub direction: FieldDirection,
    /// Only effective in horizontal direction
    pub label_width: f32,
    pub label_style: text::Style,
    pub frame: Frame,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            direction: FieldDirection::Horizontal,
            label_width: 120.0,
            label_style: text::Style::new(theme),
            frame: Frame::NONE,
        }
    }

    pub fn direction(mut self, direction: FieldDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn label_width(mut self, label_width: f32) -> Self {
        self.label_width = label_width;
        self
    }

    pub fn label_style(mut self, label_style: text::Style) -> Self {
        self.label_style = label_style;
        self
    }
}

#[derive(Clone, Copy)]
pub enum FieldDirection {
    Horizontal,
    Vertical,
}

pub fn render<R>(
    ui: &mut Ui,
    label: &str,
    input_content: impl FnOnce(&mut Ui) -> R,
    style: Style,
) -> InnerResponse<R> {
    match style.direction {
        FieldDirection::Horizontal => ui.horizontal(|ui| {
            ui.allocate_ui(egui::Vec2::new(style.label_width, 0.0), |ui| {
                ui.set_width(style.label_width);
                text::render(ui, label, style.label_style);
            });
            let resp = input_content(ui);
            resp
        }),
        FieldDirection::Vertical => ui.vertical(|ui| {
            text::render(ui, label, style.label_style);
            let resp = input_content(ui);
            resp
        }),
    }
}
