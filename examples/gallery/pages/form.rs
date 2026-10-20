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
    saved: [Option<String>; 3],
}

impl FormPage {
    pub fn show(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        scroll_area::render(
            ui,
            "form_page_scroll",
            scroll_area::ScrollAxes::Vertical,
            |ui| {
                text::render(ui, "Form / 表单", text::Style::new_heading(theme));
                text::render(
                    ui,
                    "show 托管草稿和交互状态，无需在应用里保存表单数据。",
                    text::Style::new(theme),
                );
                text::render(
                    ui,
                    "离开字段或尝试保存后显示错误；重置恢复默认值。",
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
                                    0 => "用户 A · 托管草稿",
                                    1 => "用户 B · 同类型、独立 ID 的托管草稿",
                                    _ => "用户 C · show_with_data 借用应用数据",
                                },
                                text::Style::new_heading(theme),
                            );
                            let form = Form::new(("gallery_user", index));
                            let (submitted, reset, valid, changed, snapshot) = if index < 2 {
                                let result = form.show::<UserDraft, _>(ui, |form, ui, draft| {
                                    fields(form, ui, draft, theme)
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
                                    |form, ui, draft| fields(form, ui, draft, theme),
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
                                self.saved[index] = Some(format!(
                                    "已保存：{}，{} 岁",
                                    snapshot.name.trim(),
                                    snapshot.age.value().unwrap()
                                ));
                            }
                            text::render(
                                ui,
                                &format!("当前有效：{valid} · 本帧修改：{changed}"),
                                text::Style::new(theme),
                            );
                            if let Some(saved) = &self.saved[index] {
                                text::render(ui, saved, text::Style::new(theme));
                            }
                        },
                        card::Style::new_md(theme),
                    );
                    ui.add_space(16.0);
                }
                card::render(
                    ui,
                    |ui| {
                        text::render(ui, "API 示例", text::Style::new_heading(theme));
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

fn fields(form: &mut FormUi<'_>, ui: &mut egui::Ui, draft: &mut UserDraft, theme: &Theme) {
    let style = field::Style::new(theme).direction(field::FieldDirection::Vertical);
    field::render(
        ui,
        "姓名",
        |ui| {
            form.field_with_error(
                ui,
                "name",
                &mut draft.name,
                |ui, name| {
                    text_edit::render(
                        ui,
                        name,
                        text_edit::Style::new_outline_md(theme).placeholder("请输入姓名"),
                    )
                },
                |name| {
                    if name.trim().is_empty() {
                        Err("请输入姓名".into())
                    } else {
                        Ok(())
                    }
                },
                |ui, error| {
                    text::render(
                        ui,
                        error,
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
        "年龄（18–120）",
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
                    _ => Err("请输入 18 到 120 之间的整数".into()),
                },
                |ui, error| {
                    text::render(
                        ui,
                        error,
                        text::Style::new(theme).color(theme.colors.red.normal()),
                    );
                },
            );
        },
        style,
    );
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if button::render(ui, "保存", button::Style::new_solid_md(theme)).clicked() {
            form.request_submit();
        }
        if button::render(ui, "重置", button::Style::new_outline_md(theme)).clicked() {
            form.request_reset();
        }
    });
}
