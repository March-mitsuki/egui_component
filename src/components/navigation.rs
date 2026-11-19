//! Full-width, left-aligned navigation items for sidebars and page outlines.
//!
//! Items are exposed to assistive technology as links. The current item is
//! marked like HTML `aria-current` (see [`Current`]); wrap a list of items in
//! [`group`] to expose it as a navigation landmark.
use super::focus;
use crate::{components::text, theme::Theme};
use egui::{Color32, CornerRadius, Response, Role, Sense, Ui, Vec2, WidgetInfo};

/// How the current item is announced, like HTML `aria-current`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Current {
    /// The current page, e.g. in a sidebar.
    #[default]
    Page,
    /// The current location within a page, e.g. in a table of contents.
    Location,
    /// The current step in a process.
    Step,
}

impl Current {
    fn accesskit(self) -> egui::accesskit::AriaCurrent {
        use egui::accesskit::AriaCurrent;
        match self {
            Self::Page => AriaCurrent::Page,
            Self::Location => AriaCurrent::Location,
            Self::Step => AriaCurrent::Step,
        }
    }
}

#[derive(Clone)]
pub struct Style {
    pub focus_ring: crate::theme::FocusRing,
    pub height: f32,
    pub padding: f32,
    pub radius: u8,
    pub text: text::Style,
    pub selected_background: Color32,
    pub hover_background: Color32,
    pub selected_color: Color32,
    /// How a current item is announced.
    pub current_kind: Current,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            focus_ring: theme.focus_ring,
            height: 34.0,
            padding: 12.0,
            radius: 6,
            text: text::Style::new(theme).size(14.0).selectable(false),
            selected_background: theme.colors.primary.l0,
            hover_background: theme.colors.fg.base.gamma_multiply(0.07),
            selected_color: theme.colors.primary.l6,
            current_kind: Current::Page,
        }
    }

    pub fn current_kind(mut self, kind: Current) -> Self {
        self.current_kind = kind;
        self
    }
}

/// Groups navigation items as a navigation landmark named `label`, like HTML
/// `<nav aria-label>`.
pub fn group<R>(ui: &mut Ui, label: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope_builder(
        egui::UiBuilder::new()
            .accessibility_role(Role::Navigation)
            .accessibility_label(label),
        add_contents,
    )
    .inner
}
/// Renders one item; `current` marks it as the current page or location.
/// Returns the standard clickable response, including keyboard focus semantics.
pub fn render(ui: &mut Ui, label: &str, current: bool, style: Style) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), style.height),
        Sense::click(),
    );
    response.widget_info(|| WidgetInfo::labeled(Role::Link, ui.is_enabled(), label));
    if current {
        let kind = style.current_kind.accesskit();
        ui.ctx()
            .accesskit_node_builder(response.id, |node| node.set_aria_current(kind));
    }
    let hover = ui.ctx().animate_bool_with_time(
        response.id.with("hover"),
        ui.is_enabled() && (response.hovered() || response.has_focus()),
        0.16,
    );
    if current {
        ui.painter().rect_filled(
            rect,
            CornerRadius::same(style.radius),
            style.selected_background,
        );
    }
    if hover > 0.0 {
        ui.painter().rect_filled(
            rect,
            CornerRadius::same(style.radius),
            style.hover_background.gamma_multiply(hover),
        );
    }
    let mut text_style = style.text;
    if current {
        text_style.color = style.selected_color;
    }
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(Vec2::new(style.padding, 0.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
        |ui| {
            text::render(ui, label, text_style.truncate(true));
        },
    );
    focus::ring(
        ui,
        &response,
        rect,
        CornerRadius::same(style.radius),
        style.focus_ring,
    );
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
