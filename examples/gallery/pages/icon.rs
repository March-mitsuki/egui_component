use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{icon, scroll_area, text, tooltip},
    icon::ICONS,
    theme::Theme,
};

pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    let title_style = text::Style::new_heading(theme).size(16.0);

    ui.vertical(|ui| {
        text::render(ui, locale.text("demo.icon:all"), title_style.clone());
        ui.add_space(8.0);

        let icons = [
            ("no_img", ICONS.no_img.clone()),
            ("play", ICONS.play.clone()),
            ("pause", ICONS.pause.clone()),
            ("stop", ICONS.stop.clone()),
            ("volume1", ICONS.volume1.clone()),
            ("add", ICONS.add.clone()),
            ("language", ICONS.language.clone()),
            ("trash", ICONS.trash.clone()),
            ("close", ICONS.close.clone()),
            ("arrow_down", ICONS.arrow_down.clone()),
            ("lock", ICONS.lock.clone()),
            ("lock_open", ICONS.lock_open.clone()),
            ("eye", ICONS.eye.clone()),
            ("eye_slash", ICONS.eye_slash.clone()),
        ];

        ui.scope(|ui| {
            ui.set_max_height(350.0);
            scroll_area::render(
                ui,
                "icons_scroll",
                scroll_area::ScrollAxes::Vertical,
                |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(12.0, 12.0);

                        let icon_frame = egui::Frame::NONE
                            .inner_margin(egui::Margin::same(6))
                            .corner_radius(egui::CornerRadius::same(6))
                            .stroke(egui::Stroke {
                                width: 1.0,
                                color: theme.colors.gray.normal(),
                            });

                        for (name, img) in &icons {
                            let response = icon_frame.show(ui, |ui| {
                                icon::render(ui, img.clone(), icon::Style::new(theme))
                            });
                            tooltip::render(
                                &response.response,
                                |ui| {
                                    text::render(ui, name, text::Style::new(theme));
                                },
                                &tooltip::Style::new(theme),
                            );
                        }
                    });
                },
                scroll_area::Style::new_no_frame(),
            );
        });
    });
}
