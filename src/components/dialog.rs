use egui::{Context, Ui};
use egui_component_style_macros::FrameModifier;

use crate::{icon::ICONS, theme::Theme};

#[derive(FrameModifier, Clone)]
pub struct Style {
    pub frame: egui::Frame,
    pub title: String,
    pub size: (f32, f32),
    pub backdrop_color: egui::Color32,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        let card_frame = super::utils::CardFrame::new(theme, crate::theme::CardSize::Lg);
        Self {
            title: "Dialog".to_string(),
            size: (400.0, 0.0),
            frame: card_frame.frame,
            backdrop_color: egui::Color32::from_black_alpha(100),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn size(mut self, size: (f32, f32)) -> Self {
        self.size = size;
        self
    }

    pub fn backdrop_color(mut self, color: egui::Color32) -> Self {
        self.backdrop_color = color;
        self
    }
}

/// Shows a centered modal that blocks interaction with the background.
///
/// `style.title` identifies the modal and should be unique among dialogs.
/// The caller owns the open state and may use the returned response's
/// `should_close()` to handle backdrop clicks and Escape.
///
/// Focus moves to the first control when the dialog opens, Tab and Shift+Tab
/// wrap inside it, and focus returns to the opener once it is no longer shown.
pub fn render(
    ctx: &Context,
    renderer: impl FnOnce(&mut Ui),
    style: Style,
) -> egui::ModalResponse<()> {
    let id = egui::Id::unique(&style.title);
    super::modal_focus::begin(ctx, id);
    let response = egui::Modal::new(id)
        .frame(style.frame)
        .backdrop_color(style.backdrop_color)
        .show(ctx, |ui| {
            if style.size.0 > 0.0 {
                ui.set_width(style.size.0);
            }
            if style.size.1 > 0.0 {
                ui.set_height(style.size.1);
            }

            super::modal_focus::dialog_scope(ui, |ui| {
                ui.vertical(|ui| {
                    // 渲染实际内容
                    renderer(ui);
                });
            });
        });
    super::modal_focus::end(ctx, id);
    response
}

/// Options for [`render_header`].
#[derive(Clone, Copy, Debug)]
pub struct HeaderOptions<'a> {
    /// Screen-reader name of the close button, like `aria-label`. Defaults to "Close".
    pub close_aria_label: &'a str,
}
impl Default for HeaderOptions<'_> {
    fn default() -> Self {
        Self {
            close_aria_label: "Close",
        }
    }
}

/// Standard title and close button. The title also names the dialog for
/// screen readers.
pub fn render_header(
    ui: &mut Ui,
    theme: &Theme,
    title: &str,
    on_close: impl FnOnce(),
    options: HeaderOptions<'_>,
) {
    ui.horizontal(|ui| {
        let title = super::text::render(ui, title, super::text::Style::new_heading(theme));
        super::modal_focus::label_dialog(ui.ctx(), title.id);
        ui.allocate_ui_with_layout(
            ui.available_size(),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                if super::icon_button::render(
                    ui,
                    ICONS.close.clone(),
                    options.close_aria_label,
                    super::icon_button::Style::new_ghost_sm(theme),
                )
                .clicked()
                {
                    on_close()
                }
            },
        );
    });

    ui.add_space(8.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::button;
    use egui::{Event, Key, Modifiers, RawInput};

    #[test]
    fn focus_enters_wraps_inside_and_returns_to_opener() {
        let theme =
            crate::theme::UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
                .unwrap()
                .light;
        let ctx = egui::Context::default();
        let key = |key, modifiers| Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        };
        let mut open = false;
        let mut names = std::collections::HashMap::new();
        // Returns the name of the focused widget after this pass.
        let mut run = |events: Vec<Event>, focus_opener: bool, open: &mut bool| {
            ctx.run_ui(
                RawInput {
                    events,
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 600.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    let style = button::Style::new_solid_md(&theme);
                    let opener = button::render(ui, "Open", style.clone());
                    if focus_opener {
                        opener.request_focus();
                    }
                    *open |= opener.clicked();
                    names.insert(opener.id, "opener");
                    let behind = button::render(ui, "Behind", style.clone());
                    names.insert(behind.id, "behind");
                    if *open {
                        let r = render(
                            ui.ctx(),
                            |ui| {
                                for name in ["first", "last"] {
                                    names.insert(button::render(ui, name, style.clone()).id, name);
                                }
                            },
                            Style::new(&theme),
                        );
                        *open &= !r.should_close();
                    }
                },
            )
            .textures_delta
            .clear();
            let focused = ctx.memory(|m| m.focused());
            focused.map_or("none", |id| names.get(&id).copied().unwrap_or("other"))
        };
        let tab = key(Key::Tab, Modifiers::NONE);
        let shift_tab = key(Key::Tab, Modifiers::SHIFT);
        let settle = |run: &mut dyn FnMut(Vec<Event>) -> &'static str| {
            let mut last = "none";
            for _ in 0..3 {
                last = run(vec![]);
            }
            last
        };
        run(vec![], true, &mut open);
        run(vec![key(Key::Enter, Modifiers::NONE)], false, &mut open);
        assert!(open);
        let mut step = |events| run(events, false, &mut open);
        assert_eq!(settle(&mut step), "first", "focus moves into the dialog");
        step(vec![tab.clone()]);
        assert_eq!(settle(&mut step), "last");
        step(vec![tab.clone()]);
        assert_eq!(settle(&mut step), "first", "Tab wraps to the first control");
        step(vec![shift_tab]);
        assert_eq!(
            settle(&mut step),
            "last",
            "Shift+Tab wraps to the last control"
        );
        step(vec![key(Key::Escape, Modifiers::NONE)]);
        assert_eq!(settle(&mut step), "opener", "focus returns to the opener");
        let _ = step;
        assert!(!open);
    }
}
