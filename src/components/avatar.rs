use crate::theme::Theme;
use egui::{Color32, Response, Sense, Ui};

pub struct Style {
    pub size: f32,
    pub bg_color: Color32,
    pub text_color: Color32,
    pub sense: Sense,
}

impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            size: 32.0,
            bg_color: theme.colors.primary.l6,
            text_color: theme.colors.fg.base,
            sense: Sense::click(),
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn bg_color(mut self, color: Color32) -> Self {
        self.bg_color = color;
        self
    }

    pub fn text_color(mut self, color: Color32) -> Self {
        self.text_color = color;
        self
    }

    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = sense;
        self
    }
}

/// ## Arguments
///   - `ui`: The UI to render the avatar on.
///   - `url`: The URL of the avatar. Supports 'http(s)://' or 'file://'.
///   - `name`: The name of the avatar. Uses the first character as default if image fails.
///   - `style`: The style of the avatar.
pub fn render(
    ui: &mut Ui,
    url: impl Into<String>,
    name: impl Into<String>,
    style: Style,
) -> Response {
    let url = url.into();
    let name = name.into();
    let size = egui::vec2(style.size, style.size);
    let (rect, response) = ui.allocate_at_least(size, style.sense);

    // 用作用域块隔离 painter 借用
    {
        let painter = ui.painter();
        painter.circle_filled(rect.center(), style.size / 2.0, style.bg_color);

        let first_char = name
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_default();

        if !first_char.is_empty() {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                first_char,
                egui::FontId::proportional(style.size * 0.45),
                style.text_color,
            );
        }
    } // painter 借用在此结束

    if !url.is_empty() {
        egui::Image::new(&url)
            .corner_radius(style.size / 2.0) // 根据你的 egui 版本调整类型
            .paint_at(ui, rect);
    }

    response
}
