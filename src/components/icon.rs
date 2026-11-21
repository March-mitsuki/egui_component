use super::focus;
use egui::{Response, Ui};

use crate::theme::Theme;

pub struct Style {
    pub focus_ring: crate::theme::FocusRing,
    pub box_size: f32,
    pub icon_size: f32,
    pub color: egui::Color32,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            focus_ring: theme.focus_ring,
            box_size: theme.text_size.md,
            icon_size: theme.text_size.md,
            color: theme.colors.fg.base,
            // Icons are decorative by default and must not be Tab stops.
        }
    }

    pub fn box_size(mut self, size: f32) -> Self {
        self.box_size = size;
        self
    }

    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }

    pub fn color(mut self, color: egui::Color32) -> Self {
        self.color = color;
        self
    }
}

/// Behavior of [`render`], separate from visual styling.
#[derive(Clone, Copy, Debug)]
pub struct Options<'a> {
    /// Screen-reader name, like `aria-label`; never drawn. With it an
    /// interactive icon is exposed as a button and a hover-only icon as a
    /// named image. Without it a hover-only icon is decorative and hidden from
    /// screen readers.
    pub aria_label: Option<&'a str>,
    /// `Sense::hover()` (default) keeps the icon decorative and out of the Tab order.
    pub sense: egui::Sense,
}
impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            aria_label: None,
            sense: egui::Sense::hover(),
        }
    }
}
impl<'a> Options<'a> {
    pub fn aria_label(mut self, label: &'a str) -> Self {
        self.aria_label = Some(label);
        self
    }

    /// Pass `Sense::click()` or `Sense::drag()` to make the icon interactive,
    /// for example a drag handle. Give an interactive icon an
    /// [`Options::aria_label`]; for a plain icon button prefer
    /// [`icon_button`](super::icon_button).
    pub fn sense(mut self, sense: egui::Sense) -> Self {
        self.sense = sense;
        self
    }
}

/// Debug builds warn once per icon that is interactive but has no `aria_label`.
pub fn render(
    ui: &mut Ui,
    icon: egui::ImageSource,
    style: Style,
    options: Options<'_>,
) -> Response {
    let box_size = egui::vec2(style.box_size, style.box_size);
    let (rect, response) = ui.allocate_at_least(box_size, options.sense);
    let interactive = options.sense.interactive();
    match options.aria_label {
        Some(label) => {
            let role = if interactive {
                egui::Role::Button
            } else {
                egui::Role::Image
            };
            response.widget_info(|| egui::WidgetInfo::labeled(role, ui.is_enabled(), label));
        }
        None if interactive && cfg!(debug_assertions) => {
            let key = response.id.with("icon-missing-aria-label");
            let warned = ui.data_mut(|d| std::mem::replace(d.get_temp_mut_or(key, false), true));
            if !warned {
                tracing::warn!(
                    "interactive icon {:?} has no aria_label; screen readers cannot name it",
                    response.id
                );
            }
        }
        None => {}
    }

    let icon_size = egui::vec2(style.icon_size, style.icon_size);
    let icon_rect = egui::Align2::CENTER_CENTER.align_size_within_rect(icon_size, rect);

    egui::Image::new(icon)
        .tint(style.color)
        .paint_at(ui, icon_rect);

    focus::ring(
        ui,
        &response,
        rect,
        egui::CornerRadius::same(2),
        style.focus_ring,
    );
    response
}
