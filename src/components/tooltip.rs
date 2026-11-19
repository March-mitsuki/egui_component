use egui::{Response, Tooltip, Ui};
use egui_component_style_macros::FrameModifier;

use crate::theme::Theme;

#[derive(FrameModifier, Clone)]
pub struct Style {
    align: egui::RectAlign,
    gap: f32,
    close_behavior: egui::PopupCloseBehavior,
    frame: egui::Frame,
}

impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            align: egui::RectAlign::BOTTOM,
            gap: 4.0,
            close_behavior: egui::PopupCloseBehavior::CloseOnClick,
            frame: egui::Frame::default()
                .fill(theme.colors.bg.accent)
                .stroke(egui::Stroke {
                    width: 1.0,
                    color: theme.colors.gray.l2,
                })
                .inner_margin(egui::Margin::symmetric(16, 4))
                .shadow(egui::Shadow {
                    offset: [0, 0],
                    blur: 8,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(60),
                })
                .corner_radius(egui::CornerRadius::same(theme.corner_radius.xs as u8)),
        }
    }

    pub fn align(mut self, align: egui::RectAlign) -> Self {
        self.align = align;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn close_behavior(mut self, close_behavior: egui::PopupCloseBehavior) -> Self {
        self.close_behavior = close_behavior;
        self
    }
}

/// Show `content` while `target` is hovered or has keyboard focus (WCAG 1.4.13).
/// Escape dismisses it until the pointer leaves and focus moves away.
pub fn render(target: &Response, content: impl FnOnce(&mut Ui), style: &Style) {
    let ctx = &target.ctx;
    let dismissed_id = target.id.with("egui_component_tooltip_dismissed");
    let hovered = target.enabled() && Tooltip::should_show_tooltip(target, true);
    let focused = super::focus::keyboard_focus(target);
    let mut dismissed = ctx.data(|d| d.get_temp::<bool>(dismissed_id).unwrap_or(false));
    if !target.hovered() && !target.has_focus() {
        dismissed = false;
    }
    let mut open = (hovered || focused) && !dismissed;
    // Consume Escape only while shown, so an enclosing dialog closes on the next press.
    if open && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        dismissed = true;
        open = false;
    }
    ctx.data_mut(|d| d.insert_temp(dismissed_id, dismissed));

    let mut tooltip = Tooltip::for_widget(target);
    tooltip.popup = tooltip
        .popup
        .open(open)
        .frame(style.frame)
        .align(style.align)
        .close_behavior(style.close_behavior)
        .gap(style.gap);
    let target_id = target.id;
    tooltip.show(|ui| {
        // Like `aria-describedby`: screen readers read the tooltip as the
        // target's description. egui gives the tooltip itself its role.
        let tooltip_id = ui.unique_id();
        ui.ctx().accesskit_node_builder(target_id, |node| {
            node.push_described_by(tooltip_id.accesskit_id());
        });
        content(ui);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{components::button, theme::UiTheme};
    use egui::{Event, Key, Modifiers, RawInput};

    #[test]
    fn keyboard_focus_shows_tooltip_and_escape_dismisses_it() {
        let ctx = egui::Context::default();
        let theme = UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
            .unwrap()
            .light;
        let key = |key| Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        let run = |events: Vec<Event>| {
            let mut shown = false;
            ctx.run_ui(
                RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    let r = button::render(ui, "Save", button::Style::new_solid_md(&theme));
                    render(&r, |_| shown = true, &Style::new(&theme));
                },
            )
            .textures_delta
            .clear();
            shown
        };
        assert!(!run(vec![]));
        run(vec![key(Key::Tab)]);
        assert!(run(vec![]), "shown while focused by keyboard");
        assert!(!run(vec![key(Key::Escape)]), "Escape dismisses");
        assert!(!run(vec![]), "stays dismissed while focused");
    }
}
