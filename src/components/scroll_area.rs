use egui::{Context, CornerRadius, Frame, Ui};

use crate::{components::utils::CardFrame, theme::Theme};

#[derive(Clone)]
pub struct Style {
    pub outer_frame: Frame,
    pub inner_frame: Frame,
    /// Optional viewport height cap, useful inside long documents.
    pub max_height: Option<f32>,
    /// Minimum viewport height, including inside another scroll area.
    pub min_height: Option<f32>,
    pub auto_shrink: [bool; 2],
    /// Apply an explicit offset for this frame, e.g. when switching pages.
    pub scroll_offset: Option<egui::Vec2>,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            outer_frame: CardFrame::new(theme, crate::theme::CardSize::Md)
                .inner_margin(egui::Margin::symmetric(4, 8))
                .frame,
            inner_frame: Frame::NONE.inner_margin(egui::Margin::same(4)),
            max_height: None,
            min_height: None,
            auto_shrink: [true, true],
            scroll_offset: None,
        }
    }

    pub fn new_no_frame() -> Self {
        Self {
            outer_frame: Frame::NONE,
            inner_frame: Frame::NONE,
            max_height: None,
            min_height: None,
            auto_shrink: [true, true],
            scroll_offset: None,
        }
    }

    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height.max(0.0));
        self
    }

    pub fn min_height(mut self, height: f32) -> Self {
        self.min_height = Some(height.max(0.0));
        self
    }

    pub fn auto_shrink(mut self, axes: [bool; 2]) -> Self {
        self.auto_shrink = axes;
        self
    }

    pub fn scroll_offset(mut self, offset: egui::Vec2) -> Self {
        self.scroll_offset = Some(offset);
        self
    }

    /// Padding inside the viewport. Unlike `outer_frame` margins, it does not
    /// inset the scrollbar or shrink the scrollable surface.
    pub fn padding(mut self, padding: impl Into<egui::Margin>) -> Self {
        self.inner_frame = self.inner_frame.inner_margin(padding);
        self
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
        let mut scroll = egui::ScrollArea::new(scroll_axes)
            .id_salt(id_salt)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .max_height(style.max_height.unwrap_or_else(|| ui.available_height()))
            .min_scrolled_height(
                style
                    .min_height
                    .unwrap_or(64.0)
                    .min(style.max_height.unwrap_or(f32::INFINITY)),
            )
            .auto_shrink(style.auto_shrink);
        if let Some(offset) = style.scroll_offset {
            scroll = scroll.scroll_offset(offset);
        }
        scroll.show(ui, |ui| {
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
        let style_origin = change_style(ui);
        let scroll_axes = match scroll_axes {
            ScrollAxes::Vertical => [false, true],
            ScrollAxes::Horizontal => [true, false],
            ScrollAxes::Both => [true, true],
        };
        let mut scroll = egui::ScrollArea::new(scroll_axes)
            .id_salt(id_salt)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .max_height(style.max_height.unwrap_or_else(|| ui.available_height()))
            .min_scrolled_height(
                style
                    .min_height
                    .unwrap_or(64.0)
                    .min(style.max_height.unwrap_or(f32::INFINITY)),
            )
            .auto_shrink(style.auto_shrink);
        if let Some(offset) = style.scroll_offset {
            scroll = scroll.scroll_offset(offset);
        }
        scroll.show(ui, |ui| {
            style.inner_frame.show(ui, |ui| {
                renderer(ui);
            });
        });
        ui.set_style(style_origin);
    });
}

/// change style for current ui
///
/// ### Returns
///
/// the style before change. can be used to restore style
pub fn change_style(ui: &mut Ui) -> egui::Style {
    let style_origin = (**ui.style()).clone();
    let mut style = style_origin.clone();
    set_style(&mut style);
    ui.set_style(style);
    style_origin
}

/// mutate style for global context
pub fn mutate_style(ctx: &Context) {
    ctx.global_style_mut(set_style);
}

fn set_style(style: &mut egui::Style) {
    style.spacing.scroll.floating = false;
    style.spacing.scroll.bar_width = 6.0;
    style.spacing.scroll.floating_width = 6.0;
    style.spacing.scroll.bar_inner_margin = 6.0;
    style.spacing.scroll.bar_outer_margin = 0.0;
    style.spacing.scroll.foreground_color = false;
    style.spacing.scroll.active_background_opacity = 1.0;
    style.spacing.scroll.dormant_background_opacity = 1.0;
    style.spacing.scroll.interact_background_opacity = 1.0;
    style.spacing.scroll.active_handle_opacity = 1.0;
    style.spacing.scroll.dormant_handle_opacity = 1.0;
    style.spacing.scroll.interact_handle_opacity = 1.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padding_keeps_the_viewport_at_the_parent_edges() {
        let ctx = Context::default();
        let _ = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640.0, 480.0),
                )),
                ..Default::default()
            },
            |ui| {
                let available = ui.available_rect_before_wrap();
                let mut content_origin = egui::Pos2::ZERO;
                let response = ui.scope(|ui| {
                    render(
                        ui,
                        "padded-viewport",
                        ScrollAxes::Vertical,
                        |ui| {
                            content_origin = ui.next_widget_position();
                            ui.allocate_space(egui::vec2(120.0, 1500.0));
                        },
                        Style::new_no_frame()
                            .auto_shrink([false, false])
                            .padding(egui::Margin::same(24)),
                    );
                });
                assert!((response.response.rect.right() - available.right()).abs() < 1.0);
                assert!((response.response.rect.bottom() - available.bottom()).abs() < 1.0);
                assert!((content_origin.x - available.left() - 24.0).abs() < 1.0);
                assert!((content_origin.y - available.top() - 24.0).abs() < 1.0);
            },
        );
    }

    #[test]
    fn nested_viewport_respects_explicit_minimum_height() {
        let ctx = Context::default();
        let _ = ctx.run_ui(Default::default(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let response = ui.scope(|ui| {
                    render(
                        ui,
                        "nested-height",
                        ScrollAxes::Both,
                        |ui| {
                            ui.allocate_space(egui::vec2(200.0, 600.0));
                        },
                        Style::new_no_frame()
                            .min_height(180.0)
                            .max_height(180.0)
                            .auto_shrink([false, false]),
                    );
                });
                assert!((response.response.rect.height() - 180.0).abs() < 1.0);
            });
        });
    }
}
