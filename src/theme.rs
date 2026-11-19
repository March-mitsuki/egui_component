use egui::{Color32, Stroke};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AvatarSize {
    Sm,
    Md,
    Lg,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Xs,
    Sm,
    Md,
    Lg,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Solid,
    Surface,
    Outline,
    Ghost,
}
#[derive(Clone)]
pub struct ButtonPalette {
    pub normal_bg: Color32,
    pub hover_bg: Color32,
    pub active_bg: Color32,
    pub text_color: Color32,
    pub stroke: Stroke,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextEditSize {
    Xs,
    Sm,
    Md,
    Lg,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextEditVariant {
    Subtle,
    Outline,
    Flushed,
}
#[derive(Clone)]
pub struct TextEditPalette {
    pub normal_bg: Color32,
    pub hover_bg: Color32,
    pub focus_bg: Color32,
    pub text_color: Color32,
    pub placeholder_color: Color32,
    pub border_stroke: Stroke,
    pub focus_stroke: Stroke,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CardSize {
    Sm,
    Md,
    Lg,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct Color {
    /// Text/icon color on solid surfaces (`l5`/`l6`/`l7`), e.g. solid button
    /// text and the switch knob. Keep at least 4.5:1 contrast with them.
    pub foreground: Color32,
    pub l0: Color32,
    pub l1: Color32,
    pub l2: Color32,
    pub l3: Color32,
    pub l4: Color32,
    pub l5: Color32,
    pub l6: Color32,
    pub l7: Color32,
    pub l8: Color32,
    pub l9: Color32,
}
impl Color {
    pub fn normal(&self) -> Color32 {
        self.l6
    }

    pub fn hover(&self) -> Color32 {
        self.l5
    }

    pub fn active(&self) -> Color32 {
        self.l7
    }

    pub fn border(&self) -> Color32 {
        self.l2
    }

    pub fn to_button_palette(self, variant: ButtonVariant) -> ButtonPalette {
        match variant {
            ButtonVariant::Solid => ButtonPalette {
                normal_bg: self.normal(),
                text_color: self.foreground,
                hover_bg: self.hover(),
                active_bg: self.active(),
                stroke: Stroke::NONE,
            },
            ButtonVariant::Surface => ButtonPalette {
                normal_bg: self.l0,
                text_color: self.normal(),
                hover_bg: self.l1,
                active_bg: self.l2,
                stroke: Stroke::new(1.0_f32, self.l1),
            },
            ButtonVariant::Outline => ButtonPalette {
                normal_bg: Color32::TRANSPARENT,
                text_color: self.normal(),
                hover_bg: self.l0,
                active_bg: self.l1,
                stroke: Stroke::new(1.0_f32, self.normal()),
            },
            ButtonVariant::Ghost => ButtonPalette {
                normal_bg: Color32::TRANSPARENT,
                text_color: self.normal(),
                hover_bg: self.l0,
                active_bg: self.l1,
                stroke: Stroke::NONE,
            },
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct SetColor {
    pub base: Color32,
    /// 一般来说比 base 深一点
    pub subtle: Color32,
    /// 一般来说比 base 浅一点
    pub accent: Color32,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Colors {
    pub bg: SetColor,
    pub fg: SetColor,
    pub primary: Color,
    pub gray: Color,
    pub red: Color,
    pub green: Color,
    pub blue: Color,
    pub yellow: Color,
    pub orange: Color,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct CornerRadius {
    pub xxs: f32,
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct TextSize {
    pub xxs: f32,
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct ButtonStyle {
    /// 在 button 中是 height
    /// 在 icon_button 中是默认宽度和高度
    pub size: f32,
    /// 在 button 中是 padding_x
    /// 在 icon_button 中是 padding
    pub padding: f32,

    pub text_size: f32,
    pub icon_size: f32,
    pub gap: f32,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct ButtonStyles {
    pub xs: ButtonStyle,
    pub sm: ButtonStyle,
    pub md: ButtonStyle,
    pub lg: ButtonStyle,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct AvatarStyle {
    pub size: f32,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct AvatarStyles {
    pub sm: AvatarStyle,
    pub md: AvatarStyle,
    pub lg: AvatarStyle,
}

/// The keyboard focus indicator shared by all components.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct FocusRing {
    /// Keep at least 3:1 contrast with the surfaces it is drawn on (WCAG 1.4.11).
    pub color: Color32,
    /// Stroke width in points; WCAG 2.4.13 suggests at least 2.
    pub width: f32,
}

impl FocusRing {
    pub fn stroke(self) -> Stroke {
        Stroke::new(self.width, self.color)
    }

    /// Black or white, whichever contrasts more with the ring. Drawn between
    /// the ring and the control's fill so the ring is visible on any fill.
    pub fn separator_color(self) -> Color32 {
        let contrast = |c| crate::theme_gen::contrast_ratio(self.color, c);
        if contrast(Color32::WHITE) >= contrast(Color32::BLACK) {
            Color32::WHITE
        } else {
            Color32::BLACK
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Theme {
    pub colors: Colors,
    pub focus_ring: FocusRing,
    pub corner_radius: CornerRadius,
    pub text_size: TextSize,
    pub avatar: AvatarStyles,
    pub button: ButtonStyles,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct UiTheme {
    pub dark: Theme,
    pub light: Theme,
}

impl UiTheme {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_json(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json).map_err(anyhow::Error::from)
    }

    pub fn from_json_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        serde_json::from_slice(bytes).map_err(anyhow::Error::from)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn brand_theme() -> UiTheme {
        UiTheme::from_json_bytes(include_bytes!("../examples/theme.json")).unwrap()
    }
    #[test]
    fn brand_colors_and_solid_foregrounds_follow_each_mode() {
        let ui = brand_theme();
        for (theme, primary, bg, fg, on_primary) in [
            (&ui.light, "#584EDC", "#FDFDFD", "#141414", "#FFFFFF"),
            (&ui.dark, "#A1A8FF", "#0D0D0D", "#F2F2F2", "#000000"),
        ] {
            assert_eq!(
                theme.colors.primary.normal(),
                Color32::from_hex(primary).unwrap()
            );
            assert_eq!(theme.colors.bg.base, Color32::from_hex(bg).unwrap());
            assert_eq!(theme.colors.fg.base, Color32::from_hex(fg).unwrap());
            let palette = theme.colors.primary.to_button_palette(ButtonVariant::Solid);
            assert_eq!(palette.text_color, Color32::from_hex(on_primary).unwrap());
            assert_eq!(
                crate::components::switch::Style::new(theme).knob_color,
                palette.text_color
            );
        }
    }
    #[test]
    fn theme_json_requires_foreground_and_focus_ring() {
        let theme = serde_json::to_value(brand_theme().light).unwrap();
        for pointer in ["/colors/primary/foreground", "/focus_ring"] {
            let mut json = theme.clone();
            let (parent, key) = pointer.rsplit_once('/').unwrap();
            json.pointer_mut(parent)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(key);
            assert!(
                serde_json::from_value::<Theme>(json).is_err(),
                "{pointer} is required"
            );
        }
    }
}
