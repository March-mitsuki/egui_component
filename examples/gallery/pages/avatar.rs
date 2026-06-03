use eframe::egui;

use egui_component::{components::avatar, theme::Theme};

pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    avatar::render(
        ui,
        "https://lh3.googleusercontent.com/a/ACg8ocJ3Mq2OvKiTCgqMBmzHC5wsgSvuq7N8Ph5X1K2hukZZ5zWQaIb-=s576-c-no",
        "March-Mitsuki",
        avatar::Style::new(theme),
    );
}
