use std::sync::Arc;

use egui::{Color32, FontSelection, Label, Response, RichText, Stroke, Ui, WidgetText};

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
    /// A line under the text, as in egui's `TextFormat`. `Stroke::NONE` (default) draws none.
    pub underline: Stroke,
    /// A line through the text, as in egui's `TextFormat`. `Stroke::NONE` (default) draws none.
    pub strikethrough: Stroke,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            selectable: true,
            truncate: false,
            color: theme.colors.fg.base,
            size: theme.text_size.md,
            font_family: DEFAULT_FONT_FAMILY_ALIAS.to_string(),
            font_weight: FontWeight::Regular,
            underline: Stroke::NONE,
            strikethrough: Stroke::NONE,
        }
    }

    pub fn new_heading(theme: &Theme) -> Self {
        Self {
            selectable: true,
            truncate: false,
            color: theme.colors.fg.base,
            size: theme.text_size.lg,
            font_family: DEFAULT_FONT_FAMILY_ALIAS.to_string(),
            font_weight: FontWeight::Bold,
            underline: Stroke::NONE,
            strikethrough: Stroke::NONE,
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

    /// Draw a line under the text, e.g. `Stroke::new(1.0, color)`.
    pub fn underline(mut self, underline: impl Into<Stroke>) -> Self {
        self.underline = underline.into();
        self
    }

    /// Draw a line through the text, e.g. `Stroke::new(1.0, color)`.
    pub fn strikethrough(mut self, strikethrough: impl Into<Stroke>) -> Self {
        self.strikethrough = strikethrough.into();
        self
    }
}

/// The styled text used by [`render`], for components that lay out text themselves.
/// Lines keep their own stroke color, which `RichText` cannot express, so decorated
/// text is built as a `LayoutJob`.
pub(crate) fn rich_text(ui: &Ui, text: &str, style: &Style) -> WidgetText {
    let rich = RichText::new(text)
        .family(FONT_MANAGER.get_egui_font_family(
            ui.ctx(),
            &style.font_family,
            style.font_weight,
            false,
        ))
        .color(style.color)
        .size(style.size);
    if style.underline.is_empty() && style.strikethrough.is_empty() {
        return rich.into();
    }
    let job = WidgetText::from(rich).into_layout_job(
        ui.style(),
        FontSelection::Default,
        ui.text_valign(),
    );
    let mut job = Arc::unwrap_or_clone(job);
    for section in &mut job.sections {
        section.format.underline = style.underline;
        section.format.strikethrough = style.strikethrough;
    }
    WidgetText::LayoutJob(Arc::new(job))
}

pub fn render(ui: &mut Ui, text: &str, style: Style) -> Response {
    let mut label = Label::new(rich_text(ui, text, &style)).selectable(style.selectable);

    if style.truncate {
        label = label.truncate();
    }

    ui.add(label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::UiTheme;

    fn theme() -> Theme {
        UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
            .unwrap()
            .light
    }

    #[test]
    fn decorations_keep_their_stroke_and_do_not_change_layout() {
        let theme = theme();
        let red = theme.colors.red.l6;
        let gray = theme.colors.gray.l6;
        let plain = Style::new(&theme);
        let decorated = plain
            .clone()
            .underline(Stroke::new(2.0, red))
            .strikethrough(Stroke::new(1.0, gray));
        let ctx = egui::Context::default();
        let mut checks = None;
        ctx.run_ui(Default::default(), |ui| {
            // Undecorated text keeps the RichText path existing callers relied on.
            assert!(matches!(
                rich_text(ui, "Done", &plain),
                WidgetText::RichText(_)
            ));
            let galley = |style: &Style| {
                rich_text(ui, "Done", style).into_galley(
                    ui,
                    None,
                    f32::INFINITY,
                    egui::TextStyle::Body,
                )
            };
            let (plain_galley, decorated_galley) = (galley(&plain), galley(&decorated));
            let format = &decorated_galley.job.sections[0].format;
            assert_eq!(format.underline, Stroke::new(2.0, red));
            assert_eq!(format.strikethrough, Stroke::new(1.0, gray));
            assert_eq!(format.color, plain.color, "text color is unchanged");
            let vertices = |g: &egui::Galley| -> usize {
                g.rows.iter().map(|r| r.visuals.mesh.vertices.len()).sum()
            };
            assert!(
                vertices(&decorated_galley) > vertices(&plain_galley),
                "lines are drawn"
            );
            let plain_rect = render(ui, "Done", plain.clone()).rect;
            let decorated_rect = render(ui, "Done", decorated.clone()).rect;
            checks = Some((plain_rect.size(), decorated_rect.size()));
        })
        .drop_without_applying_deltas();
        let (plain_size, decorated_size) = checks.unwrap();
        assert_eq!(plain_size, decorated_size);
    }
}
