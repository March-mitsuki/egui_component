use egui::{Context, Ui};
use egui_component_style_macros::FrameModifier;

use crate::theme::Theme;

#[derive(FrameModifier)]
pub struct Style {
    pub frame: egui::Frame,
    pub title: String,
    pub size: (f32, f32),
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        let card_frame = super::utils::CardFrame::new(theme, crate::theme::CardSize::Lg);
        Self {
            title: "Dialog".to_string(),
            size: (400.0, 0.0),
            frame: card_frame.frame,
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
}

pub fn render(ctx: &Context, renderer: impl FnOnce(&mut Ui), style: Style) {
    egui::Window::new(style.title.clone())
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .frame(style.frame)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
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
        });
}

pub fn render_header(
    ui: &mut Ui,
    theme: &Theme,
    title: &str,
    close_icon: egui::ImageSource,
    on_close: impl FnOnce(),
) {
    ui.horizontal(|ui| {
        ui.heading(title);
        ui.allocate_ui_with_layout(
            ui.available_size(),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                if super::icon_button::render(
                    ui,
                    close_icon,
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
