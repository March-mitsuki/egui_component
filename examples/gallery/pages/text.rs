use eframe::egui;
use egui_component::{
    components::text,
    consts::{DEFAULT_FONT_FAMILY_ALIAS, FONT_MANAGER},
    font::weight::FontWeight,
    theme::Theme,
};

pub struct FontDemo {
    primary: &'static str,
    cjk_fallback: bool,
    status: String,
}

impl Default for FontDemo {
    fn default() -> Self {
        Self {
            primary: "CJK fallback",
            cjk_fallback: true,
            status: "Choose a font to update the running application.".into(),
        }
    }
}

impl FontDemo {
    pub fn show(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.heading("Dynamic fonts");
        ui.label(
            "Switch the component font at runtime. Native egui controls keep their own fonts.",
        );
        ui.horizontal(|ui| {
            for (label, font_name) in [("Sans", "Ubuntu-Light"), ("Monospace", "Hack")] {
                if ui.button(label).clicked() {
                    // These bytes could also come from an asynchronous download.
                    let fonts = egui::FontDefinitions::default();
                    let data = &fonts.font_data[font_name];
                    match FONT_MANAGER
                        .replace_family(DEFAULT_FONT_FAMILY_ALIAS, &[(data.as_ref().as_ref(), 0)])
                    {
                        Ok(()) => {
                            self.primary = label;
                            self.status =
                                format!("Queued {label}; egui activates fonts on the next pass.");
                        }
                        Err(error) => self.status = error.to_string(),
                    }
                    ui.ctx().request_repaint();
                }
            }
            if ui.button("Clear primary").clicked() {
                FONT_MANAGER.remove_family(DEFAULT_FONT_FAMILY_ALIAS);
                self.primary = "CJK fallback";
                self.status = "Primary removed; the fallback chain is used.".into();
                ui.ctx().request_repaint();
            }
            if ui.button("Try invalid font").clicked() {
                let error = FONT_MANAGER
                    .replace_family(DEFAULT_FONT_FAMILY_ALIAS, &[(b"invalid font", 0)])
                    .expect_err("Invalid data must be rejected");
                self.status = format!("Rejected: {error}. Previous font preserved.");
                ui.ctx().request_repaint();
            }
        });
        if ui
            .checkbox(&mut self.cjk_fallback, "Enable CJK glyph fallback")
            .changed()
        {
            FONT_MANAGER.set_fallback_chain(if self.cjk_fallback {
                &["gallery-cjk"]
            } else {
                &[]
            });
            ui.ctx().request_repaint();
        }
        ui.label(format!("Primary: {}", self.primary));
        ui.label(&self.status);
        ui.add_space(16.0);
        for (label, weight) in [
            ("Regular", FontWeight::Regular),
            ("Bold (nearest available weight)", FontWeight::Bold),
        ] {
            ui.label(label);
            text::render(
                ui,
                "Hello, dynamic fonts! 0123456789 — 中文字体回退",
                text::Style::new(theme).size(24.0).font_weight(weight),
            );
        }
        ui.add_space(12.0);
        ui.label("Sans / Monospace do not contain CJK glyphs: toggle fallback to compare.");
    }
}
