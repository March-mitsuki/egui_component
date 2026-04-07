use egui::{CornerRadius, Frame, Ui};

use crate::theme::Theme;

use super::utils::CardFrame;

pub struct Style {
    pub outer_frame: Frame,
    pub inner_frame: Frame,
}

impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            outer_frame: CardFrame::new(theme, crate::theme::CardSize::Md)
                .frame
                .inner_margin(egui::Margin::symmetric(4, 8)),
            inner_frame: Frame::NONE.inner_margin(egui::Margin {
                left: 4,
                right: 12,
                top: 4,
                bottom: 4,
            }),
        }
    }

    pub fn new_no_frame() -> Self {
        Self {
            outer_frame: Frame::NONE,
            inner_frame: Frame::NONE,
        }
    }

    pub fn corner_radius(mut self, corner_radius: CornerRadius) -> Self {
        self.outer_frame = self.outer_frame.corner_radius(corner_radius);
        self
    }
}

pub enum ScrollAxes {
    Vertical,
    Horizontal,
    Both,
}

pub fn render_global(
    ui: &mut Ui,
    id_salt: impl std::hash::Hash,
    scroll_axes: ScrollAxes,
    renderer: impl FnOnce(&mut Ui),
    style: Style,
) {
    style.outer_frame.show(ui, |ui| {
        let scroll_axes = match scroll_axes {
            ScrollAxes::Vertical => [false, true],
            ScrollAxes::Horizontal => [true, false],
            ScrollAxes::Both => [true, true],
        };
        egui::ScrollArea::new(scroll_axes)
            .id_salt(id_salt)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .max_height(ui.available_height())
            .auto_shrink(true)
            .show(ui, |ui| {
                style.inner_frame.show(ui, |ui| {
                    renderer(ui);
                });
            });
    });
}

pub fn render(
    ui: &mut Ui,
    id_salt: impl std::hash::Hash,
    scroll_axes: ScrollAxes,
    renderer: impl FnOnce(&mut Ui),
    style: Style,
) {
    style.outer_frame.show(ui, |ui| {
        let style_origin = super::utils::change_style(ui, set_style);
        let scroll_axes = match scroll_axes {
            ScrollAxes::Vertical => [false, true],
            ScrollAxes::Horizontal => [true, false],
            ScrollAxes::Both => [true, true],
        };
        egui::ScrollArea::new(scroll_axes)
            .id_salt(id_salt)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .max_height(ui.available_height())
            .auto_shrink(true)
            .show(ui, |ui| {
                style.inner_frame.show(ui, |ui| {
                    renderer(ui);
                });
            });
        ui.set_style(style_origin);
    });
}

fn set_style(style: &mut egui::Style) {
    style.spacing.scroll.bar_width = 6.0;
    style.spacing.scroll.floating_width = 6.0;
    style.spacing.scroll.bar_inner_margin = 0.0;
    style.spacing.scroll.bar_outer_margin = 0.0;
    style.spacing.scroll.foreground_color = false;
    style.spacing.scroll.active_background_opacity = 1.0;
    style.spacing.scroll.dormant_background_opacity = 1.0;
    style.spacing.scroll.interact_background_opacity = 1.0;
    style.spacing.scroll.active_handle_opacity = 1.0;
    style.spacing.scroll.dormant_handle_opacity = 1.0;
    style.spacing.scroll.interact_handle_opacity = 1.0;
}
