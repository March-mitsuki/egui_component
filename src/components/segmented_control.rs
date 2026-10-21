//! A compact choice group composed from themed buttons.
use crate::{components::button, theme::Theme};

#[derive(Clone)]
pub struct Style {
    pub selected: button::Style,
    pub unselected: button::Style,
    pub disabled: bool,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            selected: button::Style::new_surface_sm_color(theme, theme.colors.primary),
            unselected: button::Style::new_ghost_sm(theme),
            disabled: false,
        }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
/// Returns a changed response when a different choice is activated.
pub fn render<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    value: &mut T,
    choices: &[(T, &str)],
    style: Style,
) -> egui::Response {
    let mut changed = false;
    let mut response = ui
        .add_enabled_ui(!style.disabled, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (choice, label) in choices {
                    let selected = *value == *choice;
                    let mut button_style = if selected {
                        style.selected.clone()
                    } else {
                        style.unselected.clone()
                    };
                    if !ui.is_enabled() {
                        button_style.state = button::ButtonState::Disable;
                    }
                    let response = button::render(ui, label, button_style);
                    response.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::WidgetType::SelectableLabel,
                            ui.is_enabled(),
                            selected,
                            *label,
                        )
                    });
                    if response.clicked() && !selected {
                        *value = *choice;
                        changed = true;
                    }
                }
            })
        })
        .response;
    if changed {
        response.mark_changed();
    }
    response
}
