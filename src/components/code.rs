//! Read-only syntax-highlighted code. Language names and extensions (e.g. `Rust`
//! or `rs`) are supported; unknown languages are displayed as plain text.

use egui::{Label, Response, Ui};
use egui_extras::syntax_highlighting::{self, CodeTheme};

use crate::theme::Theme;

#[derive(Clone)]
pub struct Style {
    pub size: f32,
    pub selectable: bool,
    /// Override the syntax palette. By default it follows the UI's color mode.
    pub syntax_theme: Option<CodeTheme>,
}

impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            size: theme.text_size.md,
            selectable: true,
            syntax_theme: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    pub fn syntax_theme(mut self, syntax_theme: CodeTheme) -> Self {
        self.syntax_theme = Some(syntax_theme);
        self
    }
}

/// Render code without wrapping. Place inside a horizontal scroll area when
/// long lines should scroll. Highlighting results are cached by `egui_extras`.
pub fn render(ui: &mut Ui, code: &str, language: &str, style: Style) -> Response {
    let syntax_theme = style
        .syntax_theme
        .unwrap_or_else(|| CodeTheme::from_style(ui.style()))
        .with_font_size(style.size);
    let job = syntax_highlighting::highlight(ui.ctx(), ui.style(), &syntax_theme, code, language);
    ui.add(Label::new(job).selectable(style.selectable).extend())
}
