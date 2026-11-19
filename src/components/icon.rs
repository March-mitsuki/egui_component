use super::focus::{self, Placement};
use egui::{Response, Ui};

use crate::theme::Theme;

pub struct Style {
    pub focus_ring: crate::theme::FocusRing,
    pub box_size: f32,
    pub icon_size: f32,
    pub color: egui::Color32,
    pub sense: egui::Sense,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            focus_ring: theme.focus_ring,
            box_size: theme.text_size.md,
            icon_size: theme.text_size.md,
            color: theme.colors.fg.base,
            // Icons are decorative by default and must not be Tab stops.
            sense: egui::Sense::hover(),
        }
    }

    pub fn box_size(mut self, size: f32) -> Self {
        self.box_size = size;
        self
    }

    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }

    pub fn color(mut self, color: egui::Color32) -> Self {
        self.color = color;
        self
    }

    /// Pass `Sense::click()` to make the icon clickable and focusable. Prefer
    /// [`icon_button`](super::icon_button), which also provides an accessible name.
    pub fn sense(mut self, sense: egui::Sense) -> Self {
        self.sense = sense;
        self
    }
}

pub fn render(ui: &mut Ui, icon: egui::ImageSource, style: Style) -> Response {
    let box_size = egui::vec2(style.box_size, style.box_size);
    let (rect, response) = ui.allocate_at_least(box_size, style.sense);

    let icon_size = egui::vec2(style.icon_size, style.icon_size);
    let icon_rect = egui::Align2::CENTER_CENTER.align_size_within_rect(icon_size, rect);

    egui::Image::new(icon)
        .tint(style.color)
        .paint_at(ui, icon_rect);

    focus::ring(
        ui,
        &response,
        rect,
        egui::CornerRadius::same(2),
        style.focus_ring,
        Placement::Outside,
    );
    response
}
