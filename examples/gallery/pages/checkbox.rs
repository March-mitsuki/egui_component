use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{checkbox, field, text},
    theme::Theme,
};

pub fn show(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    let title_style = text::Style::new_heading(theme).size(16.0);

    ui.vertical(|ui| {
        text::render(
            ui,
            locale.text("common:sizes"),
            title_style.clone(),
            Default::default(),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            bare(
                ui,
                "checkbox_sm",
                true,
                checkbox::Style::new_sm(theme),
                Default::default(),
            );
            ui.add_space(16.0);
            bare(
                ui,
                "checkbox_md",
                true,
                checkbox::Style::new_md(theme),
                Default::default(),
            );
            ui.add_space(16.0);
            bare(
                ui,
                "checkbox_lg",
                true,
                checkbox::Style::new_lg(theme),
                Default::default(),
            );
        });

        ui.add_space(24.0);
        text::render(
            ui,
            locale.text("common:colors"),
            title_style.clone(),
            Default::default(),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            for (id, color) in [
                ("checkbox_primary", theme.colors.primary),
                ("checkbox_green", theme.colors.green),
                ("checkbox_red", theme.colors.red),
                ("checkbox_blue", theme.colors.blue),
            ] {
                bare(
                    ui,
                    id,
                    true,
                    checkbox::Style::new_md_color(theme, color),
                    Default::default(),
                );
                ui.add_space(16.0);
            }
        });

        ui.add_space(24.0);
        text::render(
            ui,
            locale.text("demo.checkbox:withLabel"),
            title_style.clone(),
            Default::default(),
        );
        ui.add_space(8.0);
        labeled(
            ui,
            "checkbox_terms",
            locale.text("demo.checkbox:terms"),
            checkbox::Style::new(theme),
            Default::default(),
        );
        ui.add_space(4.0);
        labeled(
            ui,
            "checkbox_newsletter",
            locale.text("demo.checkbox:newsletter"),
            checkbox::Style::new(theme),
            Default::default(),
        );
        ui.add_space(4.0);
        text::render(
            ui,
            locale.text("demo.checkbox:keyboard"),
            text::Style::new(theme).color(theme.colors.fg.base.gamma_multiply(0.6)),
            Default::default(),
        );

        ui.add_space(24.0);
        text::render(
            ui,
            locale.text("demo.checkbox:selectAll"),
            title_style.clone(),
            Default::default(),
        );
        ui.add_space(8.0);
        select_all(ui, theme, locale);

        ui.add_space(24.0);
        text::render(
            ui,
            locale.text("demo.checkbox:inField"),
            title_style.clone(),
            Default::default(),
        );
        ui.add_space(8.0);
        let id = ui.make_persistent_id("checkbox_field");
        let mut on = ui.data(|d| d.get_temp::<bool>(id).unwrap_or(false));
        // Clicking the field label toggles the checkbox.
        field::render(
            ui,
            locale.text("demo.checkbox:remember"),
            |ui| checkbox::render(ui, &mut on, checkbox::Style::new(theme), Default::default()),
            field::Style::new(theme),
        );
        ui.data_mut(|d| d.insert_temp(id, on));

        ui.add_space(24.0);
        text::render(
            ui,
            locale.text("common:disabledHeading"),
            title_style,
            Default::default(),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let style = checkbox::Style::new(theme);
            let disabled = checkbox::Options::default().disabled(true);
            bare(ui, "checkbox_disabled_off", false, style.clone(), disabled);
            ui.add_space(16.0);
            bare(ui, "checkbox_disabled_on", true, style.clone(), disabled);
            ui.add_space(16.0);
            labeled(
                ui,
                "checkbox_disabled_label",
                locale.text("demo.checkbox:terms"),
                style,
                disabled,
            );
        });
    });
}

/// A "select all" checkbox that is indeterminate when only some items are checked.
fn select_all(ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
    let id = ui.make_persistent_id("checkbox_select_all");
    let mut items = ui.data(|d| d.get_temp::<[bool; 3]>(id).unwrap_or([true, false, false]));
    let all = items.iter().all(|&item| item);
    let any = items.iter().any(|&item| item);
    let mut checked = all;
    if checkbox::render_with_label(
        ui,
        &mut checked,
        locale.text("demo.checkbox:selectAll"),
        checkbox::Style::new(theme),
        checkbox::Options::default().indeterminate(any && !all),
    )
    .changed()
    {
        items = [checked; 3];
    }
    ui.indent("checkbox_select_all_items", |ui| {
        for (item, key) in items.iter_mut().zip([
            "demo.checkbox:itemA",
            "demo.checkbox:itemB",
            "demo.checkbox:itemC",
        ]) {
            checkbox::render_with_label(
                ui,
                item,
                locale.text(key),
                checkbox::Style::new(theme),
                Default::default(),
            );
        }
    });
    ui.data_mut(|d| d.insert_temp(id, items));
}

fn bare(
    ui: &mut egui::Ui,
    id: &str,
    default: bool,
    style: checkbox::Style,
    options: checkbox::Options<'_>,
) {
    let id = ui.make_persistent_id(id);
    let mut checked = ui.data(|d| d.get_temp::<bool>(id).unwrap_or(default));
    if checkbox::render(ui, &mut checked, style, options).changed() {
        ui.data_mut(|d| d.insert_temp(id, checked));
    }
}

fn labeled(
    ui: &mut egui::Ui,
    id: &str,
    label: &str,
    style: checkbox::Style,
    options: checkbox::Options<'_>,
) {
    let id = ui.make_persistent_id(id);
    let mut checked = ui.data(|d| d.get_temp::<bool>(id).unwrap_or(false));
    if checkbox::render_with_label(ui, &mut checked, label, style, options).changed() {
        ui.data_mut(|d| d.insert_temp(id, checked));
    }
}
