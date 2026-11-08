use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{button, drawer},
    theme::Theme,
};

pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    let id = ui.make_persistent_id("drawer-open");
    let mut open = ui.data_mut(|data| data.get_temp::<bool>(id).unwrap_or(false));
    if button::render(
        ui,
        locale.text("demo.drawer:open"),
        button::Style::new_outline_md(theme),
    )
    .clicked()
    {
        open = true;
    }
    // Always render, including after closing, so the exit animation completes.
    let response = drawer::render(
        ui.ctx(),
        id.with("panel"),
        open,
        |drawer| {
            drawer.header(|ui| {
                drawer::render_header(ui, theme, locale.text("demo.drawer:heading"));
            });
            drawer.body(|ui| {
                ui.label(locale.text("demo.drawer:body"));
            });
            drawer.footer(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if button::render(
                        ui,
                        locale.text("demo.drawer:save"),
                        button::Style::new_solid_md(theme),
                    )
                    .clicked()
                    {
                        ui.close();
                    }
                    if button::render(
                        ui,
                        locale.text("demo.drawer:cancel"),
                        button::Style::new_outline_md(theme),
                    )
                    .clicked()
                    {
                        ui.close();
                    }
                });
            });
        },
        drawer::Style::new(theme),
    );
    if response.should_close() {
        open = false;
    }
    ui.data_mut(|data| data.insert_temp(id, open));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gallery_body_is_visible_and_actions_are_at_bottom() {
        let ctx = egui::Context::default();
        let theme =
            egui_component::theme::UiTheme::from_json_bytes(include_bytes!("../../theme.json"))
                .unwrap();
        let mut output = egui::FullOutput::default();
        for n in 0..25 {
            output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1400.0, 900.0),
                    )),
                    time: Some(n as f64 / 60.0),
                    ..Default::default()
                },
                |ui| {
                    let id = ui.make_persistent_id("drawer-open");
                    ui.data_mut(|data| data.insert_temp(id, true));
                    show(ui, &theme.light, Locale::En);
                },
            );
        }
        let mut body_visible = false;
        let mut save_at_bottom = false;
        for clipped in &output.shapes {
            if let egui::Shape::Text(text) = &clipped.shape {
                let rect = egui::Rect::from_min_size(text.pos, text.galley.size());
                if text.galley.job.text == Locale::En.text("demo.drawer:body") {
                    body_visible = clipped.clip_rect.contains_rect(rect);
                }
                if text.galley.job.text == Locale::En.text("demo.drawer:save") {
                    save_at_bottom = rect.top() > 830.0 && rect.bottom() < 890.0;
                }
            }
        }
        assert!(
            body_visible,
            "body text must be inside its visible clip rect"
        );
        assert!(save_at_bottom, "Save must be at the bottom of the viewport");
    }
}
