use crate::i18n::Locale;
use eframe::egui::{self, Color32};
use egui_component::{
    components::{
        accordion, button, card, color_picker, scroll_area, segmented_control, select, separator,
        slider, switch, text, text_edit,
    },
    theme::{Theme, UiTheme},
    theme_gen::{self, NeutralTone, Options},
};
use serde_json::Value;
use std::collections::BTreeMap;

pub struct Generator {
    base: UiTheme,
    options: Options,
    gallery_preset: bool,
    seed_hex: String,
    overrides: BTreeMap<String, Value>,
    mode: u8,
    edit_dark: bool,
    json: String,
    message: String,
    project: String,
    notifications: bool,
    visibility: usize,
    saved: bool,
}
impl Generator {
    pub fn new(base: UiTheme) -> Self {
        Self {
            base,
            options: Options::default(),
            gallery_preset: false,
            seed_hex: "#6B65F5".into(),
            overrides: Default::default(),
            mode: 0,
            edit_dark: false,
            json: String::new(),
            message: String::new(),
            project: "My project".into(),
            notifications: true,
            visibility: 0,
            saved: false,
        }
    }
    fn generated(&self) -> UiTheme {
        if self.gallery_preset {
            self.base.clone()
        } else {
            theme_gen::generate(&self.base, self.options)
        }
    }
    fn select_gallery_preset(&mut self) {
        self.gallery_preset = true;
        self.overrides.clear();
    }
    fn select_seed(&mut self, seed: Color32) {
        self.gallery_preset = false;
        self.options.seed = seed;
        self.seed_hex = seed.to_hex();
    }
    fn value(&self) -> Value {
        let mut value = serde_json::to_value(self.generated()).unwrap();
        for (path, v) in &self.overrides {
            if let Some(target) = value.pointer_mut(path) {
                *target = v.clone();
            }
        }
        value
    }
    pub fn theme(&self) -> UiTheme {
        serde_json::from_value(self.value()).expect("validated theme overrides")
    }
    pub fn show(&mut self, ui: &mut egui::Ui, shell: &Theme, locale: Locale) -> Option<UiTheme> {
        let mut apply = None;
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(shell.colors.bg.base)
                    .inner_margin(24),
            )
            .show_inside(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
                ui.visuals_mut().widgets.noninteractive.bg_stroke =
                    egui::Stroke::new(1.0_f32, shell.colors.gray.l2);
                scroll_area::render(
                    ui,
                    "theme-generator-scroll",
                    scroll_area::ScrollAxes::Vertical,
                    |ui| {
                        if ui.available_width() > 900.0 {
                            ui.horizontal_top(|ui| {
                                ui.vertical(|ui| {
                                    ui.set_width(310.0);
                                    self.controls(ui, shell, locale, &mut apply);
                                });
                                separator::render(ui, 1.0);
                                ui.vertical(|ui| {
                                    ui.set_width(ui.available_width());
                                    self.preview(ui, shell, locale);
                                });
                            });
                        } else {
                            self.controls(ui, shell, locale, &mut apply);
                            ui.add_space(24.0);
                            self.preview(ui, shell, locale);
                        }
                    },
                    scroll_area::Style::new_no_frame().auto_shrink([false, false]),
                );
            });
        apply
    }
    fn controls(
        &mut self,
        ui: &mut egui::Ui,
        shell: &Theme,
        locale: Locale,
        apply: &mut Option<UiTheme>,
    ) {
        heading(ui, locale.text("generator:customize"), shell);
        themed_label(ui, locale.text("generator:presets"), shell);
        if action(ui, locale.text("generator:galleryPreset"), shell).clicked() {
            self.select_gallery_preset();
        }
        let previous_options = self.options;

        ui.horizontal_wrapped(|ui| {
            for (name, hex) in [
                ("Iris", "#6B65F5"),
                ("Ocean", "#1683BC"),
                ("Forest", "#24845C"),
                ("Rose", "#D44472"),
            ] {
                if action(ui, name, shell).clicked() {
                    self.select_seed(Color32::from_hex(hex).unwrap());
                }
            }
        });
        ui.add_space(12.0);
        themed_label(ui, locale.text("generator:brandSeed"), shell);
        ui.horizontal(|ui| {
            if color_picker::render(
                ui,
                ui.make_persistent_id("seed-color"),
                &mut self.options.seed,
                color_picker::Style::new(shell),
            )
            .changed()
            {
                self.options.seed = Color32::from_rgb(
                    self.options.seed.r(),
                    self.options.seed.g(),
                    self.options.seed.b(),
                );
                self.seed_hex = self.options.seed.to_hex();
            }
            text_edit::render(
                ui,
                &mut self.seed_hex,
                text_edit::Style::new_outline_md(shell).width(Some(140.0)),
            );
        });
        if let Ok(c) = Color32::from_hex(&self.seed_hex) {
            let [r, g, b, _] = c.to_srgba_unmultiplied();
            self.options.seed = Color32::from_rgb(r, g, b);
        } else {
            text::render(
                ui,
                locale.text("generator:enterAValidHexColor"),
                text::Style::new(shell).color(shell.colors.red.l6),
            );
        }
        themed_label(ui, locale.text("generator:neutralTone"), shell);
        segmented_control::render(
            ui,
            &mut self.options.neutral,
            &[
                (NeutralTone::Neutral, locale.text("generator:neutral")),
                (NeutralTone::Cool, locale.text("generator:cool")),
                (NeutralTone::Warm, locale.text("generator:warm")),
            ],
            segmented_control::Style::new(shell),
        );
        themed_label(ui, locale.text("generator:radius"), shell);
        ui.horizontal(|ui| {
            slider::render(
                ui,
                &mut self.options.radius,
                0.0..=16.0,
                slider::Style::new(shell).width(150.0).step(0.5),
            );
            numeric_input(
                ui,
                ui.make_persistent_id("radius-input"),
                &mut self.options.radius,
                0.0,
                16.0,
                shell,
            );
        });
        if self.options.seed != previous_options.seed
            || self.options.neutral != previous_options.neutral
            || self.options.radius != previous_options.radius
        {
            self.gallery_preset = false;
        }
        themed_label(
            ui,
            locale.text("generator:solidColorsMayBeAdjustedForReadableText"),
            shell,
        );
        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            if action(ui, locale.text("generator:applyToGallery"), shell).clicked() {
                *apply = Some(self.theme());
            }
            if action(ui, locale.text("generator:resetAll"), shell).clicked() {
                let base = self.base.clone();
                *self = Self::new(base);
            }
        });
        ui.add_space(16.0);
        let id = ui.make_persistent_id("generator-advanced");
        accordion::render(
            ui,
            id,
            locale.text("generator:advancedSettings"),
            None,
            None,
            |ui| {
                themed_label(
                    ui,
                    locale.text("generator:editedFieldsStayFixedWhenTheSeedChanges"),
                    shell,
                );
                if action(ui, locale.text("generator:resetAllOverrides"), shell).clicked() {
                    self.overrides.clear();
                }
                segmented_control::render(
                    ui,
                    &mut self.edit_dark,
                    &[(false, "Light"), (true, "Dark")],
                    segmented_control::Style::new(shell),
                );
                let generated = serde_json::to_value(self.generated()).unwrap();
                let mut current = self.value();
                let mode = if self.edit_dark { "dark" } else { "light" };
                scroll_area::render(
                    ui,
                    "advanced-fields",
                    scroll_area::ScrollAxes::Vertical,
                    |ui| {
                        edit_value(
                            ui,
                            &mut current[mode],
                            &generated[mode],
                            &format!("/{mode}"),
                            &mut self.overrides,
                            locale,
                            shell,
                        );
                    },
                    scroll_area::Style::new_no_frame()
                        .min_height(320.0)
                        .max_height(460.0),
                );
            },
            accordion::Style::new(shell),
        );
        ui.add_space(16.0);
        heading(ui, locale.text("generator:importExport"), shell);
        ui.horizontal_wrapped(|ui| {
            if action(ui, locale.text("generator:copyJson"), shell).clicked() {
                self.json = serde_json::to_string_pretty(&self.theme()).unwrap();
                ui.ctx().copy_text(self.json.clone());
                self.message = locale.text("generator:jsonCopied").into();
            }
            if action(ui, locale.text("generator:exportJson"), shell).clicked() {
                self.json = serde_json::to_string_pretty(&self.theme()).unwrap();
                self.message = export_json(&self.json);
            }
        });
        themed_label(
            ui,
            locale.text("generator:pasteACompleteUithemeJsonToImport"),
            shell,
        );
        egui::Frame::new()
            .fill(shell.colors.bg.subtle)
            .stroke(egui::Stroke::new(1.0_f32, shell.colors.gray.l2))
            .corner_radius(shell.corner_radius.md)
            .inner_margin(8)
            .show(ui, |ui| {
                scroll_area::render(
                    ui,
                    "theme-json",
                    scroll_area::ScrollAxes::Vertical,
                    |ui| {
                        text_edit::render_multiline(
                            ui,
                            &mut self.json,
                            5,
                            text_edit::Style::new_outline_sm(shell),
                        );
                    },
                    scroll_area::Style::new_no_frame().max_height(160.0),
                );
            });
        if action(ui, locale.text("generator:importJson"), shell).clicked() {
            match UiTheme::from_json_bytes(self.json.as_bytes()).and_then(|t| {
                theme_gen::validate(&t).map_err(anyhow_error)?;
                Ok(t)
            }) {
                Ok(theme) => {
                    let current = serde_json::to_value(theme).unwrap();
                    let generated = serde_json::to_value(self.generated()).unwrap();
                    self.overrides.clear();
                    collect_overrides(&current, &generated, "", &mut self.overrides);
                    self.message = locale
                        .text("generator:importedDifferencesAreKeptAsOverrides")
                        .into();
                }
                Err(e) => self.message = e.to_string(),
            }
        }
        if !self.message.is_empty() {
            themed_label(ui, &self.message, shell);
        }
    }
    fn preview(&mut self, ui: &mut egui::Ui, shell: &Theme, locale: Locale) {
        ui.horizontal_wrapped(|ui| {
            heading(ui, locale.text("generator:livePreview"), shell);
            segmented_control::render(
                ui,
                &mut self.mode,
                &[
                    (0, "Light"),
                    (1, "Dark"),
                    (2, locale.text("generator:compare")),
                ],
                segmented_control::Style::new(shell),
            );
        });
        let theme = self.theme();
        if self.mode == 2 && ui.available_width() > 650.0 {
            ui.columns(2, |cols| {
                self.scene(&mut cols[0], &theme.light, false, locale);
                self.scene(&mut cols[1], &theme.dark, true, locale);
            });
        } else if self.mode == 2 {
            self.scene(ui, &theme.light, false, locale);
            ui.add_space(16.0);
            self.scene(ui, &theme.dark, true, locale);
        } else {
            self.scene(
                ui,
                if self.mode == 1 {
                    &theme.dark
                } else {
                    &theme.light
                },
                self.mode == 1,
                locale,
            );
        }
    }
    fn scene(&mut self, ui: &mut egui::Ui, t: &Theme, dark: bool, locale: Locale) {
        ui.push_id(
            if dark {
                "dark-preview"
            } else {
                "light-preview"
            },
            |ui| {
                ui.scope(|ui| {
                    *ui.visuals_mut() = if dark {
                        egui::Visuals::dark()
                    } else {
                        egui::Visuals::light()
                    };
                    ui.visuals_mut().override_text_color = Some(t.colors.fg.base);
                    egui::Frame::new()
                        .fill(t.colors.bg.base)
                        .stroke(egui::Stroke::new(1.0_f32, t.colors.gray.l2))
                        .corner_radius(t.corner_radius.lg)
                        .inner_margin(20)
                        .show(ui, |ui| {
                            ui.set_min_width((ui.available_width() - 1.0).max(0.0));
                            text::render(
                                ui,
                                locale.text("generator:projectSettings"),
                                text::Style::new_heading(t).size(24.0),
                            );
                            text::render(
                                ui,
                                locale.text("generator:manageYourWorkspaceAndPreferences"),
                                text::Style::new(t).color(t.colors.fg.subtle),
                            );
                            ui.add_space(18.0);
                            card::render(
                                ui,
                                |ui| {
                                    text::render(
                                        ui,
                                        locale.text("generator:projectName"),
                                        text::Style::new(t),
                                    );
                                    text_edit::render(
                                        ui,
                                        &mut self.project,
                                        text_edit::Style::new_outline_md(t),
                                    );
                                    ui.add_space(12.0);
                                    text::render(
                                        ui,
                                        locale.text("generator:visibility"),
                                        text::Style::new(t),
                                    );
                                    let labels = [
                                        locale.text("generator:private"),
                                        locale.text("generator:team"),
                                    ];
                                    let id = ui.make_persistent_id("preview-visibility");
                                    let style = select::Style::new(t, None, select::SelectSize::Md)
                                        .width(Some(180.0));
                                    select::render(
                                        ui,
                                        id,
                                        labels[self.visibility],
                                        style.clone(),
                                        |ui| {
                                            for (index, label) in labels.into_iter().enumerate() {
                                                if select::render_item(
                                                    ui,
                                                    label,
                                                    self.visibility == index,
                                                    &style,
                                                    t,
                                                )
                                                .clicked()
                                                {
                                                    self.visibility = index;
                                                    egui::Popup::close_id(ui.ctx(), id);
                                                }
                                            }
                                        },
                                    );
                                    ui.add_space(12.0);
                                    ui.horizontal_wrapped(|ui| {
                                        text::render(
                                            ui,
                                            locale.text("generator:notifications"),
                                            text::Style::new(t),
                                        );
                                        switch::render(
                                            ui,
                                            &mut self.notifications,
                                            switch::Style::new(t),
                                        );
                                    });
                                    ui.add_space(16.0);
                                    ui.horizontal_wrapped(|ui| {
                                        if button::render(
                                            ui,
                                            locale.text("generator:saveChanges"),
                                            button::Style::new_solid_md_color(t, t.colors.primary),
                                        )
                                        .clicked()
                                        {
                                            self.saved = true;
                                        }
                                        button::render(
                                            ui,
                                            locale.text("generator:cancel"),
                                            button::Style::new_outline_md_color(
                                                t,
                                                t.colors.primary,
                                            ),
                                        );
                                    });
                                    if self.saved {
                                        text::render(
                                            ui,
                                            locale.text("generator:changesSaved"),
                                            text::Style::new(t).color(t.colors.green.l6),
                                        );
                                    }
                                },
                                card::Style::new_md(t),
                            );
                            ui.add_space(18.0);
                            ui.horizontal_wrapped(|ui| {
                                button::render(
                                    ui,
                                    locale.text("generator:surface"),
                                    button::Style::new_surface_sm_color(t, t.colors.primary),
                                );
                                button::render(
                                    ui,
                                    locale.text("generator:ghost"),
                                    button::Style::new_ghost_sm_color(t, t.colors.primary),
                                );
                                ui.add_enabled_ui(false, |ui| {
                                    button::render(
                                        ui,
                                        locale.text("generator:disabled"),
                                        button::Style::new_solid_sm_color(t, t.colors.primary),
                                    );
                                });
                                button::render(
                                    ui,
                                    locale.text("generator:deleteProject"),
                                    button::Style::new_outline_sm_color(t, t.colors.red),
                                );
                            });
                            ui.add_space(18.0);
                            let p = t.colors.primary;
                            themed_label(ui, format!("{} → {}", self.seed_hex, p.l6.to_hex()), t);
                            ui.horizontal_wrapped(|ui| {
                                for (i, c) in
                                    [p.l0, p.l1, p.l2, p.l3, p.l4, p.l5, p.l6, p.l7, p.l8, p.l9]
                                        .into_iter()
                                        .enumerate()
                                {
                                    let (rect, r) = ui.allocate_exact_size(
                                        egui::vec2(24.0, 30.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(rect, 3.0, c);
                                    r.on_hover_text(format!("l{i}: {}", c.to_hex()));
                                }
                            });
                            let fg = p.foreground.unwrap_or(t.colors.fg.base);
                            for (label, a, b) in [
                                (
                                    locale.text("generator:primarybackground"),
                                    p.l6,
                                    t.colors.bg.base,
                                ),
                                (locale.text("generator:buttonnormal"), fg, p.l6),
                                (locale.text("generator:buttonhover"), fg, p.l5),
                                (locale.text("generator:buttonactive"), fg, p.l7),
                                (
                                    locale.text("generator:bodytext"),
                                    t.colors.fg.base,
                                    t.colors.bg.base,
                                ),
                                (
                                    locale.text("generator:secondarytext"),
                                    t.colors.fg.subtle,
                                    t.colors.bg.subtle,
                                ),
                            ] {
                                if a.a() != 255 || b.a() != 255 {
                                    themed_label(ui, format!("{label}: — (alpha)"), t);
                                    continue;
                                }
                                let ratio = theme_gen::contrast_ratio(a, b);
                                themed_label(
                                    ui,
                                    format!(
                                        "{} {label}: {ratio:.2}:1",
                                        if ratio >= 4.5 { "✓" } else { "⚠" }
                                    ),
                                    t,
                                );
                            }
                        });
                });
            },
        );
    }
}
fn themed_label(ui: &mut egui::Ui, label: impl AsRef<str>, theme: &Theme) -> egui::Response {
    text::render(ui, label.as_ref(), text::Style::new(theme))
}
fn heading(ui: &mut egui::Ui, label: &str, theme: &Theme) -> egui::Response {
    text::render(ui, label, text::Style::new_heading(theme).size(20.0))
}
fn action(ui: &mut egui::Ui, label: &str, theme: &Theme) -> egui::Response {
    button::render(ui, label, button::Style::new_surface_sm(theme))
}
// Page-specific parsing only: the widget is the same TextEdit used for ordinary text.
fn numeric_input(
    ui: &mut egui::Ui,
    id: egui::Id,
    value: &mut f32,
    min: f32,
    max: f32,
    theme: &Theme,
) -> bool {
    let before = *value;
    let mut draft = ui
        .data_mut(|data| data.get_temp::<(f32, String)>(id))
        .filter(|(old, _)| *old == *value)
        .map(|(_, text)| text)
        .unwrap_or_else(|| value.to_string());
    let response = ui
        .push_id(id, |ui| {
            text_edit::render(
                ui,
                &mut draft,
                text_edit::Style::new_outline_sm(theme).width(Some(72.0)),
            )
        })
        .inner;
    let parsed = draft
        .parse::<f32>()
        .ok()
        .filter(|n| n.is_finite() && (*n >= min && *n <= max));
    if response.changed() {
        if let Some(n) = parsed {
            *value = n;
        }
    }
    if response.lost_focus() && parsed.is_none() {
        draft = value.to_string();
    }
    ui.data_mut(|data| data.insert_temp(id, (*value, draft)));
    *value != before
}
// Keep import errors in the existing anyhow result without adding a gallery dependency.
fn anyhow_error(message: &'static str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message)
}
fn collect_overrides(
    current: &Value,
    generated: &Value,
    path: &str,
    out: &mut BTreeMap<String, Value>,
) {
    if let Value::Object(object) = current {
        for (key, value) in object {
            collect_overrides(value, &generated[&key], &format!("{path}/{key}"), out);
        }
        // Legacy themes omit optional solid foregrounds; preserve that fallback.
        if generated.get("foreground").is_some() && !object.contains_key("foreground") {
            out.insert(format!("{path}/foreground"), Value::Null);
        }
    } else if current != generated {
        out.insert(path.into(), current.clone());
    }
}
fn edit_value(
    ui: &mut egui::Ui,
    value: &mut Value,
    generated: &Value,
    path: &str,
    overrides: &mut BTreeMap<String, Value>,
    locale: Locale,
    shell: &Theme,
) {
    match value {
        Value::Object(object) => {
            let mut keys = object.keys().cloned().collect::<Vec<_>>();
            let order = [
                "colors",
                "primary",
                "gray",
                "bg",
                "fg",
                "red",
                "green",
                "blue",
                "yellow",
                "orange",
                "corner_radius",
                "text_size",
                "button",
                "avatar",
            ];
            keys.sort_by_key(|key| order.iter().position(|v| v == key).unwrap_or(order.len()));
            for key in keys {
                let value = object.get_mut(&key).unwrap();
                ui.push_id(&key, |ui| {
                    let label = if overrides.contains_key(&format!("{path}/{key}")) {
                        format!("{key} •")
                    } else {
                        key.clone()
                    };
                    text::render(ui, &label, text::Style::new_heading(shell).size(14.0));
                    if value.is_object() {
                        ui.indent(&key, |ui| {
                            edit_value(
                                ui,
                                value,
                                &generated[&key],
                                &format!("{path}/{key}"),
                                overrides,
                                locale,
                                shell,
                            )
                        });
                        separator::render(ui, 1.0);
                    } else {
                        edit_value(
                            ui,
                            value,
                            &generated[&key],
                            &format!("{path}/{key}"),
                            overrides,
                            locale,
                            shell,
                        );
                    }
                });
            }
        }
        Value::Array(a) if a.len() == 4 => {
            if let Ok(mut c) = serde_json::from_value::<Color32>(Value::Array(a.clone())) {
                let mut changed = false;
                ui.horizontal_wrapped(|ui| {
                    changed = color_picker::render(
                        ui,
                        ui.make_persistent_id(path),
                        &mut c,
                        color_picker::Style::new(shell).alpha(true),
                    )
                    .changed();
                    themed_label(ui, c.to_hex(), shell);
                    reset_override(ui, path, overrides, locale, shell);
                });
                if changed {
                    overrides.insert(path.into(), serde_json::to_value(c).unwrap());
                }
            }
        }
        Value::Number(n) => {
            if let Some(x) = n.as_f64() {
                let mut v = x as f32;
                let minimum = if path.contains("/text_size") || path.ends_with("/size") {
                    1.0
                } else {
                    0.0
                };
                ui.horizontal_wrapped(|ui| {
                    if numeric_input(
                        ui,
                        ui.make_persistent_id(path),
                        &mut v,
                        minimum,
                        4096.0,
                        shell,
                    ) {
                        overrides.insert(path.into(), serde_json::json!(v));
                    }
                    reset_override(ui, path, overrides, locale, shell);
                });
            }
        }
        Value::Null => {
            reset_override(ui, path, overrides, locale, shell);
        }
        _ => {}
    }
}
fn reset_override(
    ui: &mut egui::Ui,
    path: &str,
    overrides: &mut BTreeMap<String, Value>,
    locale: Locale,
    shell: &Theme,
) {
    if overrides.contains_key(path)
        && action(ui, locale.text("generator:auto"), shell)
            .on_hover_text(path)
            .clicked()
    {
        overrides.remove(path);
    }
}
#[cfg(not(target_arch = "wasm32"))]
fn export_json(json: &str) -> String {
    // The temporary export is safe to repeat and can be moved into a project.
    let path = std::env::temp_dir().join("egui-component-theme.json");
    match std::fs::write(&path, json) {
        Ok(()) => format!("Exported: {}", path.display()),
        Err(e) => e.to_string(),
    }
}
#[cfg(target_arch = "wasm32")]
fn export_json(json: &str) -> String {
    download_theme(json);
    "Downloaded egui-component-theme.json".into()
}
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function download_theme(json) { const url = URL.createObjectURL(new Blob([json], {type:'application/json'})); const a = document.createElement('a'); a.href = url; a.download = 'egui-component-theme.json'; a.click(); setTimeout(() => URL.revokeObjectURL(url), 1000); }"
)]
extern "C" {
    fn download_theme(json: &str);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn color_presets_after_gallery_preset_update_both_preview_themes() {
        let base = UiTheme::from_json_bytes(include_bytes!("../../theme.json")).unwrap();
        let mut g = Generator::new(base.clone());
        for hex in ["#6B65F5", "#1683BC", "#24845C", "#D44472"] {
            g.select_gallery_preset();
            assert_eq!(
                serde_json::to_value(g.theme()).unwrap(),
                serde_json::to_value(&base).unwrap()
            );
            g.overrides
                .insert("/light/corner_radius/md".into(), serde_json::json!(13.0));
            g.select_seed(Color32::from_hex(hex).unwrap());
            let preview = g.theme();
            let expected = theme_gen::generate(&base, g.options);
            assert_eq!(
                serde_json::to_value(preview.light.colors.primary).unwrap(),
                serde_json::to_value(expected.light.colors.primary).unwrap()
            );
            assert_eq!(
                serde_json::to_value(preview.dark.colors.primary).unwrap(),
                serde_json::to_value(expected.dark.colors.primary).unwrap()
            );
            assert_eq!(preview.light.corner_radius.md, 13.0);
        }
    }
    #[test]
    fn preview_modes_leave_context_and_editor_colors_unchanged() {
        let ctx = egui::Context::default();
        let base = UiTheme::from_json_bytes(include_bytes!("../../theme.json")).unwrap();
        let shell = base.light.clone();
        let mut generator = Generator::new(base);
        let initial = ctx.global_style().visuals.clone();
        let mut left = None;
        for mode in [0, 1, 2, 0, 2, 1] {
            generator.mode = mode;
            for _ in 0..2 {
                let output = ctx.run_ui(
                    egui::RawInput {
                        time: Some(1.0),
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1600.0, 1100.0),
                        )),
                        ..Default::default()
                    },
                    |ui| {
                        generator.show(ui, &shell, Locale::En);
                    },
                );
                assert_eq!(
                    ctx.global_style().visuals,
                    initial,
                    "preview changed global visuals in mode {mode}"
                );
                let colors = output
                    .shapes
                    .iter()
                    .filter_map(|s| match &s.shape {
                        egui::Shape::Text(t) if t.pos.x < 335.0 => Some((
                            t.galley.job.text.clone(),
                            t.galley
                                .job
                                .sections
                                .iter()
                                .map(|s| s.format.color)
                                .collect::<Vec<_>>(),
                        )),
                        egui::Shape::Rect(r) if r.rect.max.x <= 335.0 => {
                            Some((format!("rect {:?}", r.rect), vec![r.fill, r.stroke.color]))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                if let Some(ref expected) = left {
                    assert_eq!(&colors, expected, "editor colors changed in mode {mode}");
                } else {
                    left = Some(colors);
                }
            }
        }
    }
    #[test]
    fn importing_legacy_theme_preserves_every_field() {
        let base = UiTheme::from_json_bytes(include_bytes!("../../theme.json")).unwrap();
        let mut legacy = base.clone();
        legacy.light.colors.primary.foreground = None;
        let mut g = Generator::new(base);
        let current = serde_json::to_value(&legacy).unwrap();
        collect_overrides(
            &current,
            &serde_json::to_value(g.generated()).unwrap(),
            "",
            &mut g.overrides,
        );
        assert_eq!(serde_json::to_value(g.theme()).unwrap(), current);
    }
    #[test]
    fn overrides_survive_seed_changes_and_reset_individually() {
        let base = UiTheme::from_json_bytes(include_bytes!("../../theme.json")).unwrap();
        let mut g = Generator::new(base);
        let old = g.theme();
        g.overrides
            .insert("/light/corner_radius/md".into(), serde_json::json!(13.0));
        g.options.seed = Color32::RED;
        assert_eq!(g.theme().light.corner_radius.md, 13.0);
        assert_ne!(
            g.theme().light.colors.primary.l6,
            old.light.colors.primary.l6
        );
        g.overrides.clear();
        assert_eq!(g.theme().light.corner_radius.md, g.options.radius);
    }
}
