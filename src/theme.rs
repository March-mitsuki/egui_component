use egui::{Color32, Stroke};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq)]
pub enum ButtonSize {
    Xs,
    Sm,
    Md,
    Lg,
}
#[derive(PartialEq, Eq)]
pub enum ButtonVariant {
    Solid,
    Surface,
    Outline,
    Ghost,
}
pub struct ButtonPalette {
    pub normal_bg: Color32,
    pub hover_bg: Color32,
    pub active_bg: Color32,
    pub text_color: Color32,
    pub stroke: Stroke,
}

#[derive(PartialEq, Eq)]
pub enum CardSize {
    Sm,
    Md,
    Lg,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct Color {
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

    pub fn to_button_palette(self, theme: &Theme, variant: ButtonVariant) -> ButtonPalette {
        match variant {
            ButtonVariant::Solid => ButtonPalette {
                normal_bg: self.normal(),
                text_color: theme.colors.fg.base,
                hover_bg: self.hover(),
                active_bg: self.active(),
                stroke: Stroke::NONE,
            },
            ButtonVariant::Surface => ButtonPalette {
                normal_bg: self.l0,
                text_color: self.normal(),
                hover_bg: self.l1,
                active_bg: self.l2,
                stroke: Stroke::new(1.0, self.l1),
            },
            ButtonVariant::Outline => ButtonPalette {
                normal_bg: Color32::TRANSPARENT,
                text_color: self.normal(),
                hover_bg: self.l0,
                active_bg: self.l1,
                stroke: Stroke::new(1.0, self.normal()),
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

#[derive(Deserialize, Serialize)]
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

#[derive(Deserialize, Serialize)]
pub struct Theme {
    pub colors: Colors,
    pub corner_radius: CornerRadius,
}

#[derive(Deserialize, Serialize)]
pub struct UiTheme {
    pub dark: Theme,
    pub light: Theme,
}

impl UiTheme {
    /// 只是用来开发的时候更新构造体之后输出新的占位types
    pub fn dummy() -> Self {
        let dummy_color = Color {
            l0: Color32::from_hex("#f4f7e9").unwrap(),
            l1: Color32::from_hex("#e2ebca").unwrap(),
            l2: Color32::from_hex("#cbde9b").unwrap(),
            l3: Color32::from_hex("#b3d171").unwrap(),
            l4: Color32::from_hex("#9ac44b").unwrap(),
            l5: Color32::from_hex("#81b828").unwrap(),
            l6: Color32::from_hex("#5f9119").unwrap(),
            l7: Color32::from_hex("#416b0d").unwrap(),
            l8: Color32::from_hex("#264505").unwrap(),
            l9: Color32::from_hex("#101f02").unwrap(),
        };
        let dummy_set_color = SetColor {
            base: Color32::from_hex("#f4f7e9").unwrap(),
            subtle: Color32::from_hex("#cbde9b").unwrap(),
            accent: Color32::from_hex("#cbde9b").unwrap(),
        };
        let dummy_corner_radius = CornerRadius {
            xxs: 2.0,
            xs: 4.0,
            sm: 6.0,
            md: 8.0,
            lg: 12.0,
            xl: 16.0,
        };

        // dark mode
        let dark_colors = Colors {
            bg: dummy_set_color,
            fg: dummy_set_color,
            primary: dummy_color,
            gray: dummy_color,
            red: dummy_color,
            green: dummy_color,
            blue: dummy_color,
            yellow: dummy_color,
            orange: dummy_color,
        };
        let dark = Theme {
            colors: dark_colors,
            corner_radius: dummy_corner_radius,
        };

        // light mode
        let light_colors = Colors {
            bg: dummy_set_color,
            fg: dummy_set_color,
            primary: dummy_color,
            gray: dummy_color,
            red: dummy_color,
            green: dummy_color,
            blue: dummy_color,
            yellow: dummy_color,
            orange: dummy_color,
        };
        let light = Theme {
            colors: light_colors,
            corner_radius: dummy_corner_radius,
        };
        UiTheme { dark, light }
    }

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
