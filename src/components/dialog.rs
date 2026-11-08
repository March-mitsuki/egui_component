use egui::{Context, Ui};
use egui_component_style_macros::FrameModifier;

use crate::{icon::ICONS, theme::Theme};

#[derive(FrameModifier, Clone)]
pub struct Style {
    pub frame: egui::Frame,
    pub title: String,
    pub size: (f32, f32),
    pub backdrop_color: egui::Color32,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        let card_frame = super::utils::CardFrame::new(theme, crate::theme::CardSize::Lg);
        Self {
            title: "Dialog".to_string(),
            size: (400.0, 0.0),
            frame: card_frame.frame,
            backdrop_color: egui::Color32::from_black_alpha(100),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn size(mut self, size: (f32, f32)) -> Self {
        self.size = size;
        self
    }

    pub fn backdrop_color(mut self, color: egui::Color32) -> Self {
        self.backdrop_color = color;
        self
    }
}

/// Shows a centered modal that blocks interaction with the background.
///
/// `style.title` identifies the modal and should be unique among dialogs.
/// The caller owns the open state and may use the returned response's
/// `should_close()` to handle backdrop clicks and Escape.
pub fn render(
    ctx: &Context,
    renderer: impl FnOnce(&mut Ui),
    style: Style,
) -> egui::ModalResponse<()> {
    egui::Modal::new(egui::Id::new(&style.title))
        .frame(style.frame)
        .backdrop_color(style.backdrop_color)
        .show(ctx, |ui| {
            if style.size.0 > 0.0 {
                ui.set_width(style.size.0);
            }
            if style.size.1 > 0.0 {
                ui.set_height(style.size.1);
            }

            ui.vertical(|ui| {
                // 渲染实际内容
                renderer(ui);
            });
        })
}

pub fn render_header(ui: &mut Ui, theme: &Theme, title: &str, on_close: impl FnOnce()) {
    ui.horizontal(|ui| {
        super::text::render(ui, title, super::text::Style::new_heading(theme));
        ui.allocate_ui_with_layout(
            ui.available_size(),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                if super::icon_button::render(
                    ui,
                    ICONS.close.clone(),
                    super::icon_button::Style::new_ghost_sm(theme),
                )
                .clicked()
                {
                    on_close()
                }
            },
        );
    });

    ui.add_space(8.0);
}
