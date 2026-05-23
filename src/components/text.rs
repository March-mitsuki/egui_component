use egui::{Color32, Label, Response, RichText, Ui};

use crate::{
    consts::{DEFAULT_FONT_FAMILY_ALIAS, FONT_MANAGER},
    font::weight::FontWeight,
    theme::Theme,
};

#[derive(Clone)]
pub struct Style {
    pub selectable: bool,
    pub truncate: bool,
    pub color: Color32,
    pub size: f32,
    pub font_family: String,
    pub font_weight: FontWeight,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            selectable: false,
            truncate: false,
            color: theme.colors.fg.base,
            size: theme.text_size.md,
            font_family: DEFAULT_FONT_FAMILY_ALIAS.to_string(),
            font_weight: FontWeight::Regular,
        }
    }

    pub fn new_heading(theme: &Theme) -> Self {
        Self {
            selectable: false,
            truncate: false,
            color: theme.colors.fg.base,
            size: theme.text_size.lg,
            font_family: DEFAULT_FONT_FAMILY_ALIAS.to_string(),
            font_weight: FontWeight::Bold,
        }
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn font_family(mut self, font_family: String) -> Self {
        self.font_family = font_family;
        self
    }

    pub fn font_weight(mut self, font_weight: FontWeight) -> Self {
        self.font_weight = font_weight;
        self
    }
}

pub fn render(ui: &mut Ui, text: &str, style: Style) -> Response {
    let mut label = Label::new(
        RichText::new(text)
            .family(FONT_MANAGER.get_egui_font_family(&style.font_family, style.font_weight))
            .color(style.color)
            .size(style.size),
    )
    .selectable(style.selectable);

    if style.truncate {
        label = label.truncate();
    }

    ui.add(label)
}
