//! Theme-driven progress: `Some(0.0..=1.0)` or `None` for indeterminate work.
use crate::theme::Theme;
use egui::{Color32, Response, Sense, Ui};

#[derive(Clone)]
pub struct Style {
    pub width: f32,
    pub height: f32,
    pub track_color: Color32,
    pub fill_color: Color32,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            width: 240.0,
            height: 6.0,
            track_color: theme.colors.gray.l2,
            fill_color: theme.colors.primary.l6,
        }
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}
fn fraction(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
pub fn render(ui: &mut Ui, value: Option<f32>, style: Style) -> Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(
            style.width.max(0.0).min(ui.available_width().max(0.0)),
            style.height.max(1.0),
        ),
        Sense::hover(),
    );
    response.widget_info(|| {
        let mut info = egui::WidgetInfo::new(egui::WidgetType::ProgressIndicator);
        info.value = value.map(|value| (fraction(value) * 100.0) as f64);
        info
    });
    if ui.is_rect_visible(rect) {
        let radius = rect.height() / 2.0;
        ui.painter().rect_filled(rect, radius, style.track_color);
        let (start, end) = match value {
            Some(value) => (0.0, fraction(value)),
            None => {
                ui.ctx().request_repaint();
                let phase = ui.input(|i| i.time) as f32 * 0.8;
                let start = (phase.fract() * 1.3 - 0.3).max(0.0);
                (start, (phase.fract() * 1.3).min(1.0))
            }
        };
        if end > start {
            let mut fill = rect;
            fill.min.x = rect.left() + rect.width() * start;
            fill.max.x = rect.left() + rect.width() * end;
            ui.painter().rect_filled(fill, radius, style.fill_color);
        }
    }
    response
}
#[cfg(test)]
mod tests {
    #[test]
    fn clamps_invalid_progress() {
        for (input, expected) in [
            (f32::NAN, 0.0),
            (f32::INFINITY, 0.0),
            (-1.0, 0.0),
            (2.0, 1.0),
            (0.4, 0.4),
        ] {
            assert_eq!(super::fraction(input), expected);
        }
    }
}
