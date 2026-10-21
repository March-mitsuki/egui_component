use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{slider, text, text_edit},
    theme::Theme,
};
pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    let id = ui.make_persistent_id("slider-demo");
    let mut value = ui.data_mut(|data| data.get_temp::<f32>(id).unwrap_or(40.0));
    text::render(ui, locale.text("slider:help"), text::Style::new(theme));
    ui.horizontal(|ui| {
        slider::render(
            ui,
            &mut value,
            0.0..=100.0,
            slider::Style::new(theme).width(220.0).step(1.0),
        );
        let key = id.with("input");
        let mut draft = ui
            .data_mut(|data| data.get_temp::<(f32, String)>(key))
            .filter(|(v, _)| *v == value)
            .map(|(_, s)| s)
            .unwrap_or_else(|| format!("{value:.1}"));
        if text_edit::render(
            ui,
            &mut draft,
            text_edit::Style::new_outline_sm(theme).width(Some(90.0)),
        )
        .changed()
            && let Ok(v) = draft.parse::<f32>()
            && v.is_finite()
            && (0.0..=100.0).contains(&v)
        {
            value = v;
        }
        ui.data_mut(|data| data.insert_temp(key, (value, draft)));
    });
    ui.data_mut(|data| data.insert_temp(id, value));
    ui.add_space(16.0);
    text::render(
        ui,
        locale.text("common:disabledHeading"),
        text::Style::new(theme),
    );
    slider::render(
        ui,
        &mut value,
        0.0..=100.0,
        slider::Style::new(theme).disabled(true),
    );
}
