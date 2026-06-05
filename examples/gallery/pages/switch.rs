use eframe::egui;
use egui_component::{components::switch, components::text, theme::Theme};

pub fn show(ui: &mut egui::Ui, theme: &Theme) {
    let title_style = text::Style::new_heading(theme).size(16.0);

    ui.vertical(|ui| {
        text::render(ui, "Sizes / 尺寸", title_style.clone());
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            render_interactive_switch(ui, "switch_size_sm", false, switch::Style::new_sm(theme));
            ui.add_space(16.0);
            render_interactive_switch(ui, "switch_size_md", false, switch::Style::new_md(theme));
            ui.add_space(16.0);
            render_interactive_switch(ui, "switch_size_lg", true, switch::Style::new_lg(theme));
        });

        ui.add_space(24.0);

        text::render(ui, "Colors / 颜色", title_style.clone());
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            render_interactive_switch(
                ui,
                "switch_color_primary",
                true,
                switch::Style::new_md(theme),
            );
            ui.add_space(16.0);
            render_interactive_switch(
                ui,
                "switch_color_green",
                true,
                switch::Style::new_md_color(theme, theme.colors.green),
            );
            ui.add_space(16.0);
            render_interactive_switch(
                ui,
                "switch_color_red",
                true,
                switch::Style::new_md_color(theme, theme.colors.red),
            );
            ui.add_space(16.0);
            render_interactive_switch(
                ui,
                "switch_color_blue",
                true,
                switch::Style::new_md_color(theme, theme.colors.blue),
            );
        });

        ui.add_space(24.0);

        text::render(ui, "Disabled / 禁用", title_style);
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            render_interactive_switch(
                ui,
                "switch_disabled_off",
                false,
                switch::Style::new_md(theme).disabled(true),
            );
            ui.add_space(16.0);
            render_interactive_switch(
                ui,
                "switch_disabled_on",
                true,
                switch::Style::new_md(theme).disabled(true),
            );
        });
    });
}

fn render_interactive_switch(
    ui: &mut egui::Ui,
    id_source: &str,
    default: bool,
    style: switch::Style,
) {
    let id = ui.make_persistent_id(id_source);
    let mut val = ui.memory(|mem| mem.data.get_temp::<bool>(id).unwrap_or(default));
    let response = switch::render(ui, &mut val, style);
    if response.changed() {
        ui.memory_mut(|mem| mem.data.insert_temp(id, val));
    }
}
