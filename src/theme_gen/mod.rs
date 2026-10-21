//! Deterministic sRGB theme generation using OKLCH, with role-specific light/dark scales.
use crate::theme::{Color, CornerRadius, SetColor, UiTheme};
use egui::Color32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NeutralTone {
    #[default]
    Neutral,
    Cool,
    Warm,
}
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub seed: Color32,
    pub neutral: NeutralTone,
    /// Medium radius; the other radius tokens scale proportionally.
    pub radius: f32,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            seed: Color32::from_rgb(107, 101, 245),
            neutral: NeutralTone::Neutral,
            radius: 8.0,
        }
    }
}

/// Generate colors and radii while retaining component metrics from `base`.
pub fn generate(base: &UiTheme, options: Options) -> UiTheme {
    let (_, chroma, hue) = to_oklch(options.seed);
    let mut result = base.clone();
    for (theme, dark) in [(&mut result.light, false), (&mut result.dark, true)] {
        theme.colors.primary = scale(hue, chroma.clamp(0.0, 0.24), dark);
        let (nh, nc) = match options.neutral {
            NeutralTone::Neutral => (0.0, 0.0),
            NeutralTone::Cool => (260.0, 0.012),
            NeutralTone::Warm => (75.0, 0.012),
        };
        theme.colors.gray = scale(nh, nc, dark);
        let levels = if dark {
            [0.16, 0.20, 0.24]
        } else {
            [0.995, 0.975, 0.95]
        };
        theme.colors.bg = SetColor {
            base: oklch(levels[0], nc, nh),
            subtle: oklch(levels[1], nc, nh),
            accent: oklch(levels[2], nc, nh),
        };
        let levels = if dark {
            [0.96, 0.79, 1.0]
        } else {
            [0.19, 0.37, 0.10]
        };
        theme.colors.fg = SetColor {
            base: oklch(levels[0], nc, nh),
            subtle: oklch(levels[1], nc, nh),
            accent: oklch(levels[2], nc, nh),
        };
        theme.colors.red = scale(25.0, 0.20, dark);
        theme.colors.green = scale(150.0, 0.16, dark);
        theme.colors.blue = scale(255.0, 0.19, dark);
        theme.colors.yellow = scale(95.0, 0.17, dark);
        theme.colors.orange = scale(55.0, 0.18, dark);
        // These tokens are also used for text on both base and subtle surfaces.
        let backgrounds = [
            theme.colors.bg.base,
            theme.colors.bg.subtle,
            theme.colors.bg.accent,
        ];
        for color in [
            &mut theme.colors.primary,
            &mut theme.colors.gray,
            &mut theme.colors.red,
            &mut theme.colors.green,
            &mut theme.colors.blue,
            &mut theme.colors.yellow,
            &mut theme.colors.orange,
        ] {
            correct_roles(color, &backgrounds, dark);
        }
        let r = if options.radius.is_finite() {
            options.radius.clamp(0.0, 32.0)
        } else {
            8.0
        };
        theme.corner_radius = CornerRadius {
            xxs: r * 0.25,
            xs: r * 0.5,
            sm: r * 0.75,
            md: r,
            lg: r * 1.5,
            xl: r * 2.0,
        };
    }
    result
}
fn scale(h: f32, c: f32, dark: bool) -> Color {
    let ls = if dark {
        [0.23, 0.28, 0.34, 0.42, 0.52, 0.82, 0.76, 0.69, 0.88, 0.95]
    } else {
        [0.97, 0.93, 0.86, 0.78, 0.68, 0.59, 0.53, 0.46, 0.37, 0.28]
    };
    let cs = [0.12, 0.22, 0.35, 0.55, 0.75, 0.90, 1.0, 0.90, 0.65, 0.35];
    let a: [Color32; 10] = std::array::from_fn(|i| oklch(ls[i], c * cs[i], h));
    Color {
        foreground: None,
        l0: a[0],
        l1: a[1],
        l2: a[2],
        l3: a[3],
        l4: a[4],
        l5: a[5],
        l6: a[6],
        l7: a[7],
        l8: a[8],
        l9: a[9],
    }
}
fn correct_roles(color: &mut Color, backgrounds: &[Color32], dark: bool) {
    // Move the solid trio together, preserving hover/active separation.
    let (mut l, c, h) = to_oklch(color.l6);
    for _ in 0..100 {
        color.l6 = oklch(l, c, h);
        color.l5 = oklch((l + 0.06).min(0.98), c * 0.9, h);
        color.l7 = oklch((l - 0.06).max(0.02), c * 0.9, h);
        let trio = [color.l5, color.l6, color.l7];
        let foreground = [Color32::BLACK, Color32::WHITE]
            .into_iter()
            .max_by(|a, b| min_contrast(*a, &trio).total_cmp(&min_contrast(*b, &trio)))
            .unwrap();
        color.foreground = Some(foreground);
        if min_contrast(color.l6, backgrounds) >= 4.5 && min_contrast(foreground, &trio) >= 4.5 {
            break;
        }
        l = (l + if dark { 0.005 } else { -0.005 }).clamp(0.10, 0.94);
    }
}
fn min_contrast(a: Color32, bs: &[Color32]) -> f32 {
    bs.iter()
        .map(|b| contrast_ratio(a, *b))
        .fold(f32::INFINITY, f32::min)
}
/// WCAG relative-luminance contrast for opaque sRGB colors.
pub fn contrast_ratio(a: Color32, b: Color32) -> f32 {
    fn luminance(c: Color32) -> f32 {
        0.2126 * linear(c.r()) + 0.7152 * linear(c.g()) + 0.0722 * linear(c.b())
    }
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
fn linear(v: u8) -> f32 {
    let v = v as f32 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
fn to_oklch(color: Color32) -> (f32, f32, f32) {
    let (r, g, b) = (linear(color.r()), linear(color.g()), linear(color.b()));
    let l = (0.41222146 * r + 0.53633255 * g + 0.051445995 * b).cbrt();
    let m = (0.2119035 * r + 0.6806995 * g + 0.10739696 * b).cbrt();
    let s = (0.08830246 * r + 0.28171885 * g + 0.6299787 * b).cbrt();
    let a = 1.9779985 * l - 2.4285922 * m + 0.4505937 * s;
    let b = 0.025904037 * l + 0.78277177 * m - 0.80867577 * s;
    let c = a.hypot(b);
    (
        0.21045426 * l + 0.7936178 * m - 0.004072047 * s,
        if c < 0.00001 { 0.0 } else { c },
        b.atan2(a).to_degrees(),
    )
}
fn rgb(l: f32, c: f32, h: f32) -> [f32; 3] {
    let (a, b) = (c * h.to_radians().cos(), c * h.to_radians().sin());
    let ll = (l + 0.39633778 * a + 0.21580376 * b).powi(3);
    let m = (l - 0.105561346 * a - 0.06385417 * b).powi(3);
    let s = (l - 0.08948418 * a - 1.2914855 * b).powi(3);
    [
        4.0767417 * ll - 3.3077116 * m + 0.23096994 * s,
        -1.268438 * ll + 2.6097574 * m - 0.3413194 * s,
        -0.0041960863 * ll - 0.7034186 * m + 1.7076147 * s,
    ]
}
fn oklch(l: f32, c: f32, h: f32) -> Color32 {
    let (mut lo, mut hi) = (0.0, c);
    let in_gamut = |a: [f32; 3]| a.into_iter().all(|x| (-0.000001..=1.000001).contains(&x));
    if !in_gamut(rgb(l, c, h)) {
        for _ in 0..24 {
            let mid = (lo + hi) * 0.5;
            if in_gamut(rgb(l, mid, h)) {
                lo = mid
            } else {
                hi = mid
            }
        }
    } else {
        lo = c;
    }
    let a = rgb(l, lo, h).map(|v| {
        let v = v.clamp(0.0, 1.0);
        let s = if v <= 0.0031308 {
            12.92 * v
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        };
        (s * 255.0).round() as u8
    });
    Color32::from_rgb(a[0], a[1], a[2])
}

/// Validate the structural/finite constraints required by gallery theme editing.
pub fn validate(theme: &UiTheme) -> Result<(), &'static str> {
    for t in [&theme.light, &theme.dark] {
        let value = serde_json::to_value(t).map_err(|_| "Invalid theme")?;
        fn check(v: &serde_json::Value) -> bool {
            match v {
                serde_json::Value::Number(n) => n
                    .as_f64()
                    .is_some_and(|n| n.is_finite() && (0.0..=4096.0).contains(&n)),
                serde_json::Value::Array(a) => a.iter().all(check),
                serde_json::Value::Object(o) => o.values().all(check),
                serde_json::Value::Null => false,
                _ => true,
            }
        }
        // Optional foreground may be absent, never null in normal serialized themes.
        if !check(&value) {
            return Err("Theme values must be finite and between 0 and 4096");
        }
        for size in [t.button.xs, t.button.sm, t.button.md, t.button.lg] {
            if size.size <= 0.0 || size.text_size <= 0.0 {
                return Err("Button and text sizes must be positive");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extreme_seeds_keep_roles_readable_and_round_trip() {
        let base = UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json")).unwrap();
        for seed in [
            Color32::WHITE,
            Color32::BLACK,
            Color32::GRAY,
            Color32::YELLOW,
            Color32::RED,
            Color32::BLUE,
            Color32::GREEN,
            Color32::from_rgb(107, 101, 245),
        ] {
            let theme = generate(
                &base,
                Options {
                    seed,
                    ..Default::default()
                },
            );
            validate(&theme).unwrap();
            UiTheme::from_json_bytes(theme.to_json().as_bytes()).unwrap();
            for t in [&theme.light, &theme.dark] {
                for c in [
                    t.colors.primary,
                    t.colors.gray,
                    t.colors.red,
                    t.colors.green,
                    t.colors.blue,
                    t.colors.yellow,
                    t.colors.orange,
                ] {
                    for bg in [t.colors.bg.base, t.colors.bg.subtle, t.colors.bg.accent] {
                        assert!(contrast_ratio(c.l6, bg) >= 4.5, "seed {seed:?}");
                    }
                    for bg in [c.l5, c.l6, c.l7] {
                        assert!(
                            contrast_ratio(c.foreground.unwrap(), bg) >= 4.5,
                            "seed {seed:?}, contrast {}",
                            contrast_ratio(c.foreground.unwrap(), bg)
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn contrast_endpoints() {
        assert!((contrast_ratio(Color32::WHITE, Color32::BLACK) - 21.0).abs() < 0.001);
    }
}
