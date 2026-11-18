use egui::{CursorIcon, Frame, InnerResponse, Response, Sense, Ui};

use crate::components::text;
use crate::theme::Theme;

#[derive(Clone)]
pub struct Style {
    pub direction: FieldDirection,
    /// Only effective in horizontal direction
    pub label_width: f32,
    pub label_style: text::Style,
    pub frame: Frame,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            direction: FieldDirection::Horizontal,
            label_width: 120.0,
            label_style: text::Style::new(theme),
            frame: Frame::NONE,
        }
    }

    pub fn direction(mut self, direction: FieldDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn label_width(mut self, label_width: f32) -> Self {
        self.label_width = label_width;
        self
    }

    pub fn label_style(mut self, label_style: text::Style) -> Self {
        self.label_style = label_style;
        self
    }
}

#[derive(Clone, Copy)]
pub enum FieldDirection {
    Horizontal,
    Vertical,
}

/// Render a label and its input control.
///
/// `input_content` must return the control's [`Response`]. Clicking the label
/// focuses that control; toggle controls such as [`switch`](super::switch)
/// also perform their primary action (on the next pass).
pub fn render(
    ui: &mut Ui,
    label: &str,
    input_content: impl FnOnce(&mut Ui) -> Response,
    style: Style,
) -> InnerResponse<Response> {
    // Selectable text would compete with the label's click sense.
    let label_style = style.label_style.selectable(false);
    let render_label = |ui: &mut Ui| {
        let text = text::render(ui, label, label_style);
        // `Sense::CLICK` without `FOCUSABLE`: like an HTML <label>, the label
        // reacts to clicks but is not a Tab stop.
        ui.interact(text.rect, text.id.with("field_label"), Sense::CLICK)
    };
    let body = |ui: &mut Ui, label: Response| {
        let input = input_content(ui);
        let target = input.enabled() && input.sense.is_focusable();
        if target {
            label.clone().on_hover_cursor(CursorIcon::PointingHand);
            if label.clicked() {
                input.request_focus();
                super::activation::request(ui.ctx(), input.id);
            }
        }
        input
    };
    match style.direction {
        FieldDirection::Horizontal => ui.horizontal(|ui| {
            let label = ui
                .allocate_ui(egui::Vec2::new(style.label_width, 0.0), |ui| {
                    ui.set_width(style.label_width);
                    render_label(ui)
                })
                .inner;
            body(ui, label)
        }),
        FieldDirection::Vertical => ui.vertical(|ui| {
            let label = render_label(ui);
            body(ui, label)
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{components::switch, components::text_edit, theme::UiTheme};
    use egui::{Event, Modifiers, PointerButton, Pos2, RawInput};

    fn click(pos: Pos2) -> Vec<Vec<Event>> {
        let button = |pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        vec![
            vec![Event::PointerMoved(pos)],
            vec![button(true)],
            vec![button(false)],
            vec![],
            vec![],
        ]
    }

    #[test]
    fn label_click_focuses_input_and_toggles_switch() {
        let ctx = egui::Context::default();
        let theme = UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
            .unwrap()
            .light;
        let mut text = String::new();
        let mut on = false;
        let mut label_rects = (egui::Rect::NOTHING, egui::Rect::NOTHING);
        let mut text_focused = false;
        let run = |events: Vec<Event>,
                   text: &mut String,
                   on: &mut bool,
                   rects: &mut (egui::Rect, egui::Rect),
                   focused: &mut bool| {
            ctx.run_ui(
                RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    let r = render(
                        ui,
                        "Name",
                        |ui| text_edit::render(ui, text, text_edit::Style::new_outline_md(&theme)),
                        Style::new(&theme),
                    );
                    *focused = r.inner.has_focus();
                    rects.0 = r.response.rect.with_max_x(r.response.rect.left() + 20.0);
                    let r = render(
                        ui,
                        "Notify",
                        |ui| switch::render(ui, on, switch::Style::new(&theme)),
                        Style::new(&theme),
                    );
                    rects.1 = r.response.rect.with_max_x(r.response.rect.left() + 20.0);
                },
            )
            .textures_delta
            .clear();
        };
        run(
            vec![],
            &mut text,
            &mut on,
            &mut label_rects,
            &mut text_focused,
        );
        for events in click(label_rects.0.center()) {
            run(
                events,
                &mut text,
                &mut on,
                &mut label_rects,
                &mut text_focused,
            );
        }
        assert!(text_focused);
        assert!(!on);
        for events in click(label_rects.1.center()) {
            run(
                events,
                &mut text,
                &mut on,
                &mut label_rects,
                &mut text_focused,
            );
        }
        assert!(on);
        assert!(!text_focused);
        for events in click(label_rects.1.center()) {
            run(
                events,
                &mut text,
                &mut on,
                &mut label_rects,
                &mut text_focused,
            );
        }
        assert!(!on, "a second label click toggles back exactly once");
    }

    #[test]
    fn tab_skips_labels() {
        let ctx = egui::Context::default();
        let theme = UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
            .unwrap()
            .light;
        let (mut first, mut second) = (String::new(), String::new());
        let mut ids = (egui::Id::NULL, egui::Id::NULL);
        for (pass, events) in [vec![], vec![]]
            .into_iter()
            .chain([vec![Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }]])
            .enumerate()
        {
            ctx.run_ui(
                RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    let a = render(
                        ui,
                        "First",
                        |ui| {
                            text_edit::render(
                                ui,
                                &mut first,
                                text_edit::Style::new_outline_md(&theme),
                            )
                        },
                        Style::new(&theme),
                    );
                    let b = render(
                        ui,
                        "Second",
                        |ui| {
                            text_edit::render(
                                ui,
                                &mut second,
                                text_edit::Style::new_outline_md(&theme),
                            )
                        },
                        Style::new(&theme),
                    );
                    if pass == 0 {
                        a.inner.request_focus();
                    }
                    ids = (a.inner.id, b.inner.id);
                },
            )
            .textures_delta
            .clear();
        }
        assert_eq!(ctx.memory(|m| m.focused()), Some(ids.1));
    }
}
