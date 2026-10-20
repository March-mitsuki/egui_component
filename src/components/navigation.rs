//! Full-width, left-aligned navigation items for sidebars and page outlines.
use crate::{components::text, theme::Theme};
use egui::{Color32, CornerRadius, Response, Sense, Ui, Vec2, WidgetInfo, WidgetType};

#[derive(Clone)]
pub struct Style {
    pub height: f32,
    pub padding: f32,
    pub radius: u8,
    pub text: text::Style,
    pub selected_background: Color32,
    pub hover_background: Color32,
    pub selected_color: Color32,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            height: 34.0,
            padding: 12.0,
            radius: 6,
            text: text::Style::new(theme).size(14.0),
            selected_background: theme.colors.primary.l0,
            hover_background: theme.colors.fg.base.gamma_multiply(0.07),
            selected_color: theme.colors.primary.l6,
        }
    }
}
/// Returns the standard clickable response, including keyboard focus semantics.
pub fn render(ui: &mut Ui, label: &str, selected: bool, style: Style) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), style.height),
        Sense::click(),
    );
    response.widget_info(|| {
        WidgetInfo::selected(
            WidgetType::SelectableLabel,
            ui.is_enabled(),
            selected,
            label,
        )
    });
    let hover = ui.ctx().animate_bool_with_time(
        response.id.with("hover"),
        ui.is_enabled() && (response.hovered() || response.has_focus()),
        0.16,
    );
    if selected {
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
    if selected {
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
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
