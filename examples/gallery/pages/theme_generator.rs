use crate::i18n::Locale;
use eframe::egui::{self, Color32};
use egui_component::{
    components::{
        accordion, button, card, color_picker, icon, scroll_area, segmented_control, select,
        separator, slider, switch, text, text_edit,
    },
    icon::lucide,
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
    /// Result of auditing the pasted JSON; never applied to the editor.
    audit: Option<Result<[Vec<theme_gen::ContrastCheck>; 2], String>>,
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
            audit: None,
            project: "My project".into(),
            notifications: true,
            visibility: 0,
            saved: false,
        }
    }
    fn generated(&self) -> UiTheme {
        if self.gallery_preset {
            // The preset keeps its colors; only the focus ring follows the options.
            let mut theme = self.base.clone();
            theme_gen::apply_focus_ring(&mut theme.light, self.options);
            theme_gen::apply_focus_ring(&mut theme.dark, self.options);
            theme
        } else {
            theme_gen::generate(&self.base, self.options)
        }
    }
    /// Audit the pasted JSON in both modes without touching the editor.
    fn check_contrast(&mut self) {
        self.audit = Some(
            UiTheme::from_json_bytes(self.json.as_bytes())
                .map(|t| [theme_gen::audit(&t.light), theme_gen::audit(&t.dark)])
                .map_err(|e| e.to_string()),
        );
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
                    .inner_margin(egui::Margin::symmetric(24, 20)),
            )
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
                scroll_area::render(
                    ui,
                    "theme-generator-scroll",
                    scroll_area::ScrollAxes::Vertical,
                    |ui| {
                        self.page_header(ui, shell, locale);
                        ui.add_space(20.0);
                        if ui.available_width() > 900.0 {
                            ui.horizontal_top(|ui| {
                                ui.vertical(|ui| {
                                    ui.set_width(340.0);
                                    self.controls(ui, shell, locale, &mut apply);
                                });
                                ui.add_space(16.0);
                                ui.vertical(|ui| {
                                    ui.set_width(ui.available_width());
                                    self.preview(ui, shell, locale);
                                });
                            });
                        } else {
                            self.controls(ui, shell, locale, &mut apply);
                            ui.add_space(16.0);
                            self.preview(ui, shell, locale);
                        }
                    },
                    scroll_area::Style::new_no_frame().auto_shrink([false, false]),
                );
            });
        apply
    }
    fn page_header(&mut self, ui: &mut egui::Ui, shell: &Theme, locale: Locale) {
        text::render(
            ui,
            locale.text("nav:themeGenerator"),
            text::Style::new_heading(shell).size(30.0),
        );
        text::render(
            ui,
            locale.text("generator:startWithABrandColorBuildAThemeForEveryComponent"),
            text::Style::new(shell).color(shell.colors.fg.subtle),
        );
    }
    fn controls(
        &mut self,
        ui: &mut egui::Ui,
        shell: &Theme,
        locale: Locale,
        apply: &mut Option<UiTheme>,
    ) {
        let previous_options = self.options;
        section(
            ui,
            shell,
            lucide::PALETTE,
            locale.text("generator:brand"),
            |ui| {
                field_label(ui, locale.text("generator:presets"), shell);
                ui.horizontal_wrapped(|ui| {
                    if preset_chip(
                        ui,
                        locale.text("generator:galleryPreset"),
                        shell.colors.fg.base,
                        lucide::SPARKLES,
                        self.gallery_preset,
                        shell,
                    )
                    .clicked()
                    {
                        self.select_gallery_preset();
                    }
                    for (name, hex) in PRESETS {
                        let color = Color32::from_hex(hex).unwrap();
                        let selected = !self.gallery_preset && self.options.seed == color;
                        if preset_chip(ui, name, color, lucide::CIRCLE, selected, shell).clicked() {
                            self.select_seed(color);
                        }
                    }
                });
                ui.add_space(4.0);
                field_label(ui, locale.text("generator:brandSeed"), shell);
                ui.horizontal(|ui| {
                    if color_picker::render(
                        ui,
                        ui.make_persistent_id("seed-color"),
                        &mut self.options.seed,
                        color_picker::Style::new(shell),
                        Default::default(),
                    )
                    .changed()
                    {
                        let [r, g, b, _] = self.options.seed.to_srgba_unmultiplied();
                        self.options.seed = Color32::from_rgb(r, g, b);
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
                    hint(
                        ui,
                        locale.text("generator:enterAValidHexColor"),
                        shell.colors.red.l6,
                    );
                }
                hint(
                    ui,
                    locale.text("generator:solidColorsMayBeAdjustedForReadableText"),
                    shell.colors.fg.subtle,
                );
            },
        );
        section(
            ui,
            shell,
            lucide::LAYERS,
            locale.text("generator:surfaces"),
            |ui| {
                field_label(ui, locale.text("generator:neutralTone"), shell);
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
                ui.add_space(4.0);
                field_label(ui, locale.text("generator:radius"), shell);
                slider_with_input(ui, "radius", &mut self.options.radius, 0.0..=16.0, shell);
            },
        );
        if self.options.seed != previous_options.seed
            || self.options.neutral != previous_options.neutral
            || self.options.radius != previous_options.radius
        {
            self.gallery_preset = false;
        }
        section(
            ui,
            shell,
            lucide::SCAN,
            locale.text("generator:focusRing"),
            |ui| {
                let mut custom = self.options.focus_ring_color.is_some();
                segmented_control::render(
                    ui,
                    &mut custom,
                    &[
                        (false, locale.text("generator:focusAuto")),
                        (true, locale.text("generator:focusCustom")),
                    ],
                    segmented_control::Style::new(shell),
                );
                let theme = self.theme();
                if custom {
                    let mut color = self
                        .options
                        .focus_ring_color
                        .unwrap_or(theme.light.focus_ring.color);
                    ui.horizontal(|ui| {
                        color_picker::render(
                            ui,
                            ui.make_persistent_id("focus-color"),
                            &mut color,
                            color_picker::Style::new(shell),
                            Default::default(),
                        );
                        field_label(ui, &hex(color), shell);
                    });
                    self.options.focus_ring_color = Some(color);
                } else {
                    self.options.focus_ring_color = None;
                    for (label, t) in [("Light", &theme.light), ("Dark", &theme.dark)] {
                        ui.horizontal(|ui| {
                            swatch(ui, t.focus_ring.color, 20.0, shell);
                            field_label(
                                ui,
                                &format!("{label}  {}", hex(t.focus_ring.color)),
                                shell,
                            );
                        });
                    }
                }
                hint(
                    ui,
                    if custom {
                        locale.text("generator:focusCustomHint")
                    } else {
                        locale.text("generator:focusAutoHint")
                    },
                    shell.colors.fg.subtle,
                );
                ui.add_space(4.0);
                field_label(ui, locale.text("generator:focusWidth"), shell);
                slider_with_input(
                    ui,
                    "focus-width",
                    &mut self.options.focus_ring_width,
                    1.0..=4.0,
                    shell,
                );
            },
        );
        ui.horizontal_wrapped(|ui| {
            if button::render(
                ui,
                locale.text("generator:applyToGallery"),
                button::Style::new_solid_md_color(shell, shell.colors.primary)
                    .icon_left(button::ButtonIcon::Some(lucide::CHECK)),
            )
            .clicked()
            {
                *apply = Some(self.theme());
            }
            if button::render(
                ui,
                locale.text("generator:resetAll"),
                button::Style::new_ghost_md(shell)
                    .icon_left(button::ButtonIcon::Some(lucide::ROTATE_CCW)),
            )
            .clicked()
            {
                let base = self.base.clone();
                *self = Self::new(base);
            }
        });
        ui.add_space(8.0);
        let id = ui.make_persistent_id("generator-advanced");
        section_frame(shell).show(ui, |ui| {
            ui.set_width(ui.available_width());
            accordion::render(
                ui,
                id,
                locale.text("generator:advancedSettings"),
                None,
                None,
                |ui| {
                    hint(
                        ui,
                        locale.text("generator:editedFieldsStayFixedWhenTheSeedChanges"),
                        shell.colors.fg.subtle,
                    );
                    ui.horizontal_wrapped(|ui| {
                        segmented_control::render(
                            ui,
                            &mut self.edit_dark,
                            &[(false, "Light"), (true, "Dark")],
                            segmented_control::Style::new(shell),
                        );
                        if !self.overrides.is_empty()
                            && button::render(
                                ui,
                                locale.text("generator:resetAllOverrides"),
                                button::Style::new_ghost_sm(shell)
                                    .icon_left(button::ButtonIcon::Some(lucide::ROTATE_CCW)),
                            )
                            .clicked()
                        {
                            self.overrides.clear();
                        }
                    });
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
        });
        section(
            ui,
            shell,
            lucide::BRACES,
            locale.text("generator:importExport"),
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    if action(ui, locale.text("generator:copyJson"), lucide::COPY, shell).clicked()
                    {
                        self.json = serde_json::to_string_pretty(&self.theme()).unwrap();
                        ui.ctx().copy_text(self.json.clone());
                        self.message = locale.text("generator:jsonCopied").into();
                    }
                    if action(
                        ui,
                        locale.text("generator:exportJson"),
                        lucide::DOWNLOAD,
                        shell,
                    )
                    .clicked()
                    {
                        self.json = serde_json::to_string_pretty(&self.theme()).unwrap();
                        self.message = export_json(&self.json);
                    }
                });
                hint(
                    ui,
                    locale.text("generator:pasteACompleteUithemeJsonToImport"),
                    shell.colors.fg.subtle,
                );
                text_edit::render_multiline(
                    ui,
                    &mut self.json,
                    5,
                    text_edit::Style::new_outline_sm(shell),
                );
                let mut import = false;
                ui.horizontal_wrapped(|ui| {
                    import = action(
                        ui,
                        locale.text("generator:importJson"),
                        lucide::UPLOAD,
                        shell,
                    )
                    .clicked();
                    if action(
                        ui,
                        locale.text("generator:checkContrast"),
                        lucide::CONTRAST,
                        shell,
                    )
                    .clicked()
                    {
                        self.check_contrast();
                    }
                });
                hint(
                    ui,
                    locale.text("generator:checkContrastHint"),
                    shell.colors.fg.subtle,
                );
                if import {
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
                    hint(ui, &self.message, shell.colors.fg.subtle);
                }
                match &self.audit {
                    None => {}
                    Some(Err(e)) => hint(ui, e, shell.colors.red.l6),
                    Some(Ok([light, dark])) => {
                        ui.add_space(8.0);
                        for (mode, issues) in [("common:light", light), ("common:dark", dark)] {
                            field_label(ui, locale.text(mode), shell);
                            audit_report(ui, shell, issues, locale);
                            ui.add_space(4.0);
                        }
                    }
                }
            },
        );
    }
    fn preview(&mut self, ui: &mut egui::Ui, shell: &Theme, locale: Locale) {
        ui.horizontal(|ui| {
            text::render(
                ui,
                locale.text("generator:livePreview"),
                text::Style::new_heading(shell).size(18.0),
            );
            ui.add_space(12.0);
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
        ui.add_space(4.0);
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
                            theme_details(ui, t, &self.seed_hex, locale);
                        });
                });
            },
        );
    }
}
fn themed_label(ui: &mut egui::Ui, label: impl AsRef<str>, theme: &Theme) -> egui::Response {
    text::render(ui, label.as_ref(), text::Style::new(theme))
}
const PRESETS: [(&str, &str); 4] = [
    ("Iris", "#6B65F5"),
    ("Ocean", "#1683BC"),
    ("Forest", "#24845C"),
    ("Rose", "#D44472"),
];
/// Primary palette, focus ring samples and contrast checks under the preview card.
fn theme_details(ui: &mut egui::Ui, t: &Theme, seed_hex: &str, locale: Locale) {
    let p = t.colors.primary;
    ui.horizontal(|ui| {
        field_label(ui, locale.text("generator:primaryScale"), t);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            field_label(ui, &format!("{seed_hex} → {}", hex(p.l6)), t);
        });
    });
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let levels = [p.l0, p.l1, p.l2, p.l3, p.l4, p.l5, p.l6, p.l7, p.l8, p.l9];
        let width = ((ui.available_width() - 36.0) / 10.0).clamp(18.0, 48.0);
        for (i, c) in levels.into_iter().enumerate() {
            let (rect, r) = ui.allocate_exact_size(egui::vec2(width, 28.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, t.corner_radius.xs, c);
            if i == 6 {
                ui.painter().rect_stroke(
                    rect.expand(1.5),
                    t.corner_radius.xs + 1.5,
                    egui::Stroke::new(1.5, t.colors.fg.base),
                    egui::StrokeKind::Inside,
                );
            }
            r.on_hover_text(format!("l{i}: {}", hex(c)));
        }
    });
    ui.add_space(12.0);
    field_label(ui, locale.text("generator:focusSample"), t);
    ui.horizontal_wrapped(|ui| {
        focus_sample(
            ui,
            t,
            t.colors.bg.base,
            t.colors.fg.base,
            Some(t.colors.gray.l2),
            "Aa",
        );
        focus_sample(ui, t, p.l6, p.foreground, None, "Aa");
        focus_sample(ui, t, t.colors.bg.subtle, t.colors.fg.base, None, "Aa");
    });
    ui.add_space(12.0);
    field_label(ui, locale.text("generator:contrast"), t);
    let backgrounds = [t.colors.bg.base, t.colors.bg.subtle, t.colors.bg.accent];
    let min_on_bg = |c: Color32| {
        backgrounds
            .iter()
            .map(|bg| theme_gen::contrast_ratio(c, *bg))
            .fold(f32::INFINITY, f32::min)
    };
    let fg = p.foreground;
    let rows = [
        (
            locale.text("generator:primarybackground"),
            min_on_bg(p.l6),
            4.5,
        ),
        (locale.text("generator:buttonnormal"), ratio(fg, p.l6), 4.5),
        (locale.text("generator:buttonhover"), ratio(fg, p.l5), 4.5),
        (locale.text("generator:buttonactive"), ratio(fg, p.l7), 4.5),
        (
            locale.text("generator:bodytext"),
            ratio(t.colors.fg.base, t.colors.bg.base),
            4.5,
        ),
        (
            locale.text("generator:secondarytext"),
            ratio(t.colors.fg.subtle, t.colors.bg.subtle),
            4.5,
        ),
        (
            locale.text("generator:focusRingBackground"),
            min_on_bg(t.focus_ring.color),
            3.0,
        ),
        (
            locale.text("generator:focusRingSeparator"),
            ratio(t.focus_ring.color, t.focus_ring.separator_color()),
            9.0,
        ),
    ];
    ui.columns(if ui.available_width() > 480.0 { 2 } else { 1 }, |cols| {
        let n = cols.len();
        let per = rows.len().div_ceil(n);
        for (i, (label, value, min)) in rows.iter().enumerate() {
            contrast_row(&mut cols[i / per], t, label, *value, *min);
        }
    });
    ui.add_space(12.0);
    field_label(ui, locale.text("generator:audit"), t);
    audit_report(ui, t, &theme_gen::audit(t), locale);
}
/// Every failing pair from [`theme_gen::audit`], errors first. Two lines per
/// issue so long token paths fit the narrow sidebar.
fn audit_report(ui: &mut egui::Ui, t: &Theme, issues: &[theme_gen::ContrastCheck], locale: Locale) {
    let line = |ui: &mut egui::Ui, icon_source, color, label: &str| {
        ui.horizontal(|ui| {
            icon::render(
                ui,
                icon_source,
                icon::Style::new(t)
                    .box_size(16.0)
                    .icon_size(14.0)
                    .color(color),
                Default::default(),
            );
            text::render(ui, label, text::Style::new(t).size(13.0));
        });
    };
    if issues.is_empty() {
        line(
            ui,
            lucide::CIRCLE_CHECK,
            t.colors.green.l6,
            locale.text("generator:auditPass"),
        );
        return;
    }
    for issue in issues {
        let (icon_source, color, severity) = match issue.severity {
            theme_gen::Severity::Error => (
                lucide::CIRCLE_X,
                t.colors.red.l6,
                locale.text("generator:auditError"),
            ),
            theme_gen::Severity::Warning => (
                lucide::TRIANGLE_ALERT,
                t.colors.orange.l6,
                locale.text("generator:auditWarning"),
            ),
        };
        line(
            ui,
            icon_source,
            color,
            &format!("{} / {}", issue.foreground, issue.background),
        );
        ui.horizontal(|ui| {
            ui.add_space(16.0 + ui.spacing().item_spacing.x);
            text::render(
                ui,
                &format!("{severity} · {:.2}:1 < {}:1", issue.ratio, issue.required),
                text::Style::new(t).size(12.0).color(t.colors.fg.subtle),
            );
        });
    }
}
/// `#rrggbb`, with alpha only when it is not opaque.
fn hex(color: Color32) -> String {
    let hex = color.to_hex();
    if color.a() == 255 {
        hex[..7].to_owned()
    } else {
        hex
    }
}
fn ratio(a: Color32, b: Color32) -> f32 {
    if a.a() != 255 || b.a() != 255 {
        f32::NAN
    } else {
        theme_gen::contrast_ratio(a, b)
    }
}
fn contrast_row(ui: &mut egui::Ui, t: &Theme, label: &str, value: f32, min: f32) {
    ui.horizontal(|ui| {
        let (icon, color) = if value.is_nan() {
            (lucide::INFO, t.colors.fg.subtle)
        } else if value >= min {
            (lucide::CIRCLE_CHECK, t.colors.green.l6)
        } else {
            (lucide::TRIANGLE_ALERT, t.colors.orange.l6)
        };
        icon::render(
            ui,
            icon,
            icon::Style::new(t)
                .box_size(16.0)
                .icon_size(14.0)
                .color(color),
            Default::default(),
        );
        text::render(ui, label, text::Style::new(t).size(13.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(12.0);
            let value = if value.is_nan() {
                "— (alpha)".to_owned()
            } else {
                format!("{value:.2}:1")
            };
            text::render(
                ui,
                &value,
                text::Style::new(t).size(13.0).color(t.colors.fg.subtle),
            );
        });
    });
}
/// A control-shaped tile painted with the theme's focus ring, as it looks on
/// keyboard focus.
fn focus_sample(
    ui: &mut egui::Ui,
    t: &Theme,
    fill: Color32,
    text_color: Color32,
    border: Option<Color32>,
    label: &str,
) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(72.0, 36.0), egui::Sense::hover());
    let radius = egui::CornerRadius::same(t.corner_radius.sm as u8);
    let painter = ui.painter();
    painter.rect_filled(rect, radius, fill);
    if let Some(border) = border {
        painter.rect_stroke(
            rect,
            radius,
            egui::Stroke::new(1.0, border),
            egui::StrokeKind::Inside,
        );
    }
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(14.0),
        text_color,
    );
    let ring = t.focus_ring;
    painter.rect_stroke(rect, radius, ring.stroke(), egui::StrokeKind::Inside);
    let inner = ring.width.round() as u8;
    painter.rect_stroke(
        rect.shrink(ring.width),
        egui::CornerRadius::same(radius.nw.saturating_sub(inner)),
        egui::Stroke::new(1.0, ring.separator_color()),
        egui::StrokeKind::Inside,
    );
}
fn section_frame(theme: &Theme) -> egui::Frame {
    egui::Frame::new()
        .fill(theme.colors.bg.base)
        .stroke(egui::Stroke::new(1.0_f32, theme.colors.gray.l2))
        .corner_radius(theme.corner_radius.lg)
        .inner_margin(egui::Margin::same(16))
}
/// A titled card grouping related settings.
fn section(
    ui: &mut egui::Ui,
    theme: &Theme,
    icon_source: egui::ImageSource<'static>,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    section_frame(theme).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            icon::render(
                ui,
                icon_source,
                icon::Style::new(theme)
                    .box_size(18.0)
                    .icon_size(16.0)
                    .color(theme.colors.primary.l6),
                Default::default(),
            );
            text::render(ui, title, text::Style::new_heading(theme).size(15.0));
        });
        ui.add_space(4.0);
        add_contents(ui);
    });
    ui.add_space(8.0);
}
fn field_label(ui: &mut egui::Ui, label: &str, theme: &Theme) -> egui::Response {
    text::render(
        ui,
        label,
        text::Style::new(theme)
            .size(13.0)
            .color(theme.colors.fg.subtle),
    )
}
fn hint(ui: &mut egui::Ui, label: &str, color: Color32) {
    ui.label(egui::RichText::new(label).size(12.0).color(color));
}
fn swatch(ui: &mut egui::Ui, color: Color32, size: f32, theme: &Theme) {
    let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(size), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, theme.corner_radius.xs, color);
    ui.painter().rect_stroke(
        rect,
        theme.corner_radius.xs,
        egui::Stroke::new(1.0, theme.colors.gray.l2),
        egui::StrokeKind::Inside,
    );
}
/// An outline chip that shows its preset color through the icon and label.
fn preset_chip(
    ui: &mut egui::Ui,
    label: &str,
    color: Color32,
    icon_source: egui::ImageSource<'static>,
    selected: bool,
    theme: &Theme,
) -> egui::Response {
    let mut style =
        button::Style::new_outline_sm(theme).icon_left(button::ButtonIcon::Some(icon_source));
    style.palette.text_color = color;
    style.palette.stroke = egui::Stroke::new(
        1.0_f32,
        if selected {
            color
        } else {
            theme.colors.gray.l2
        },
    );
    if selected {
        style.palette.normal_bg = theme.colors.bg.subtle;
    }
    button::render(ui, label, style)
}
fn slider_with_input(
    ui: &mut egui::Ui,
    id: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    theme: &Theme,
) {
    ui.horizontal(|ui| {
        slider::render(
            ui,
            value,
            range.clone(),
            slider::Style::new(theme).width(180.0).step(0.5),
        );
        numeric_input(
            ui,
            ui.make_persistent_id(format!("{id}-input")),
            value,
            *range.start(),
            *range.end(),
            theme,
        );
    });
}
fn action(
    ui: &mut egui::Ui,
    label: &str,
    icon_source: egui::ImageSource<'static>,
    theme: &Theme,
) -> egui::Response {
    button::render(
        ui,
        label,
        button::Style::new_outline_sm(theme).icon_left(button::ButtonIcon::Some(icon_source)),
    )
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
                "focus_ring",
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
                        Default::default(),
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
        && action(ui, locale.text("generator:auto"), lucide::ROTATE_CCW, shell)
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
                let mut output = ctx.run_ui(
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
                output.textures_delta.clear();
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
    fn importing_theme_preserves_every_field() {
        let base = UiTheme::from_json_bytes(include_bytes!("../../theme.json")).unwrap();
        let mut imported = base.clone();
        imported.light.colors.red.foreground = Color32::from_rgb(1, 2, 3);
        imported.dark.focus_ring.width = 3.0;
        let mut g = Generator::new(base);
        let current = serde_json::to_value(&imported).unwrap();
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
    #[test]
    fn checking_pasted_json_reports_issues_without_importing() {
        let base = UiTheme::from_json_bytes(include_bytes!("../../theme.json")).unwrap();
        let mut pasted = base.clone();
        pasted.dark.colors.fg.subtle = pasted.dark.colors.bg.base;
        let mut g = Generator::new(base);
        let before = serde_json::to_value(g.theme()).unwrap();
        g.json = serde_json::to_string(&pasted).unwrap();
        g.check_contrast();
        let Some(Ok([light, dark])) = &g.audit else {
            panic!("pasted JSON parses");
        };
        assert!(light.is_empty());
        assert_eq!(dark[0].foreground, "fg.subtle");
        assert_eq!(dark[0].severity, theme_gen::Severity::Error);
        assert_eq!(serde_json::to_value(g.theme()).unwrap(), before);

        g.json = "{}".into();
        g.check_contrast();
        assert!(matches!(g.audit, Some(Err(_))));
    }
}
