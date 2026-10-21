use crate::i18n::Locale;
use eframe::egui;
use egui_component::{
    components::{button, card, field, scroll_area, text, text_edit},
    form::{Form, FormUi, Number},
    theme::Theme,
};

#[derive(Clone)]
struct UserDraft {
    name: String,
    age: Number<u8>,
}

impl Default for UserDraft {
    fn default() -> Self {
        Self {
            name: String::new(),
            age: Number::new(18),
        }
    }
}

#[derive(Default)]
pub struct FormPage {
    external_draft: UserDraft,
    saved: [Option<(String, u8)>; 3],
}

impl FormPage {
    pub fn show(&mut self, ui: &mut egui::Ui, theme: &Theme, locale: Locale) {
        scroll_area::render(
            ui,
            "form_page_scroll",
            scroll_area::ScrollAxes::Vertical,
            |ui| {
                text::render(
                    ui,
                    locale.text("demo.form:title"),
                    text::Style::new_heading(theme),
                );
                text::render(
                    ui,
                    locale.text("demo.form:managed"),
                    text::Style::new(theme),
                );
                text::render(
                    ui,
                    locale.text("demo.form:validation"),
                    text::Style::new(theme),
                );
                ui.add_space(12.0);
                for index in 0..3 {
                    card::render(
                        ui,
                        |ui| {
                            text::render(
                                ui,
                                match index {
                                    0 => locale.text("demo.form:userA"),
                                    1 => locale.text("demo.form:userB"),
                                    _ => locale.text("demo.form:userC"),
                                },
                                text::Style::new_heading(theme),
                            );
                            let form = Form::new(("gallery_user", index));
                            let (submitted, reset, valid, changed, snapshot) = if index < 2 {
                                let result = form.show::<UserDraft, _>(ui, |form, ui, draft| {
                                    fields(form, ui, draft, theme, locale)
                                });
                                (
                                    result.submitted,
                                    result.reset_requested,
                                    result.valid,
                                    result.changed,
                                    result.data,
                                )
                            } else {
                                let result = form.show_with_data(
                                    ui,
                                    &mut self.external_draft,
                                    |form, ui, draft| fields(form, ui, draft, theme, locale),
                                );
                                if result.reset_requested {
                                    self.external_draft = UserDraft::default();
                                }
                                (
                                    result.submitted,
                                    result.reset_requested,
                                    result.valid,
                                    result.changed,
                                    self.external_draft.clone(),
                                )
                            };
                            if reset {
                                self.saved[index] = None;
                            } else if submitted {
                                self.saved[index] = Some((
                                    snapshot.name.trim().to_owned(),
                                    snapshot.age.value().unwrap(),
                                ));
                            }
                            text::render(
                                ui,
                                &locale.format(
                                    "demo.form:status",
                                    &[
                                        ("valid", &valid.to_string()),
                                        ("changed", &changed.to_string()),
                                    ],
                                ),
                                text::Style::new(theme),
                            );
                            if let Some((name, age)) = &self.saved[index] {
                                text::render(
                                    ui,
                                    &locale.format(
                                        "demo.form:saved",
                                        &[("name", name), ("age", &age.to_string())],
                                    ),
                                    text::Style::new(theme),
                                );
                            }
                        },
                        card::Style::new_md(theme),
                    );
                    ui.add_space(16.0);
                }
                card::render(
                    ui,
                    |ui| {
                        text::render(
                            ui,
                            locale.text("demo.form:api"),
                            text::Style::new_heading(theme),
                        );
                        text::render(
                            ui,
                            r#"// 托管草稿，返回值携带当前数据快照
let result = Form::new(("edit_user", user_id))
    .show::<UserDraft, _>(ui, |form, ui, draft| {
        // 使用 field_with_error 接入自定义输入框及错误文本
        // 自定义按钮点击后调用 form.request_submit()
    });
if result.submitted { save(result.data); }
// 托管模式重置会自动恢复 UserDraft::default()

// 应用持有数据：无 Default / Clone / Send / Sync 要求
Form::new("external").show_with_data(ui, &mut self.draft, fields);
// 此模式由应用响应 reset_requested，恢复自己的草稿

// 应用也可以持有交互状态
Form::new("explicit").show_with_state(ui, &mut self.draft,
    &mut self.form_state, fields);

// 结束托管编辑会话时，清除缓存
Form::new(("edit_user", user_id)).remove::<UserDraft>(ui);"#,
                            text::Style::new(theme).selectable(true),
                        );
                    },
                    card::Style::new_md(theme),
                );
            },
            scroll_area::Style::new_no_frame(),
        );
    }
}

fn fields(
    form: &mut FormUi<'_>,
    ui: &mut egui::Ui,
    draft: &mut UserDraft,
    theme: &Theme,
    locale: Locale,
) {
    let style = field::Style::new(theme).direction(field::FieldDirection::Vertical);
    field::render(
        ui,
        locale.text("demo.form:name"),
        |ui| {
            form.field_with_error(
                ui,
                "name",
                &mut draft.name,
                |ui, name| {
                    text_edit::render(
                        ui,
                        name,
                        text_edit::Style::new_outline_md(theme)
                            .placeholder(locale.text("demo.form:enterName")),
                    )
                },
                |name| {
                    if name.trim().is_empty() {
                        Err("demo.form:enterName".into())
                    } else {
                        Ok(())
                    }
                },
                |ui, error| {
                    text::render(
                        ui,
                        locale.text(error),
                        text::Style::new(theme).color(theme.colors.red.normal()),
                    );
                },
            );
        },
        style.clone(),
    );
    ui.add_space(8.0);
    field::render(
        ui,
        locale.text("demo.form:age"),
        |ui| {
            form.field_with_error(
                ui,
                "age",
                &mut draft.age,
                |ui, age| {
                    text_edit::render(ui, age.raw_mut(), text_edit::Style::new_outline_md(theme))
                },
                |age| match age.value() {
                    Ok(18..=120) => Ok(()),
                    _ => Err("demo.form:enterAge".into()),
                },
                |ui, error| {
                    text::render(
                        ui,
                        locale.text(error),
                        text::Style::new(theme).color(theme.colors.red.normal()),
                    );
                },
            );
        },
        style,
    );
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if button::render(
            ui,
            locale.text("common:save"),
            button::Style::new_solid_md(theme),
        )
        .clicked()
        {
            form.request_submit();
        }
        if button::render(
            ui,
            locale.text("common:reset"),
            button::Style::new_outline_md(theme),
        )
        .clicked()
        {
            form.request_reset();
        }
    });
}
