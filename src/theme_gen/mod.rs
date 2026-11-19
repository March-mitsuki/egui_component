//! Deterministic sRGB theme generation using OKLCH, with role-specific light/dark scales.
use crate::theme::{Color, CornerRadius, FocusRing, SetColor, Theme, UiTheme};
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
    /// Light-mode focus ring color; `None` derives one from primary with
    /// [`auto_focus_color`].
    pub focus_ring_color_light: Option<Color32>,
    /// Dark-mode focus ring color; `None` derives one like the light color.
    pub focus_ring_color_dark: Option<Color32>,
    /// Focus ring width in points, shared by both modes.
    pub focus_ring_width: f32,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            seed: Color32::from_rgb(107, 101, 245),
            neutral: NeutralTone::Neutral,
            radius: 8.0,
            focus_ring_color_light: None,
            focus_ring_color_dark: None,
            focus_ring_width: 2.0,
        }
    }
}

/// Generate colors, radii and the focus ring while retaining component metrics
/// from `base`.
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
        let color = if dark {
            options.focus_ring_color_dark
        } else {
            options.focus_ring_color_light
        };
        apply_focus_ring(theme, color, options.focus_ring_width);
    }
    result
}

/// Set `theme.focus_ring`, deriving the color with [`auto_focus_color`] when
/// it is `None`. The width is clamped to 1..=8 points.
pub fn apply_focus_ring(theme: &mut Theme, color: Option<Color32>, width: f32) {
    theme.focus_ring = FocusRing {
        color: color.unwrap_or_else(|| auto_focus_color(theme)),
        width: if width.is_finite() {
            width.clamp(1.0, 8.0)
        } else {
            2.0
        },
    };
}

/// A focus ring color derived from primary: same hue and chroma, at the
/// lightness closest to primary that has 3:1 contrast with every background
/// (required, WCAG 1.4.11) and with primary's solid fill `l6`, so it stays
/// distinct on primary buttons and selected days. When both cannot be met
/// (typical in dark mode, where primary is light), it keeps the background
/// contrast, moves away from the background, and maximizes the fill contrast;
/// the ring's separator line keeps it visible on the fill either way.
pub fn auto_focus_color(theme: &Theme) -> Color32 {
    let primary = theme.colors.primary;
    let backgrounds = [
        theme.colors.bg.base,
        theme.colors.bg.subtle,
        theme.colors.bg.accent,
    ];
    let fills = [primary.l6];
    let (l, c, h) = to_oklch(primary.l6);
    // Keep away from pure black/white so the ring keeps primary's hue.
    let candidates = (40..=192).map(|i| i as f32 * 0.005).map(|lightness| {
        let color = oklch(lightness, c, h);
        let on_bg = min_contrast(color, &backgrounds);
        let on_fill = min_contrast(color, &fills);
        (lightness, color, on_bg, on_fill)
    });
    let meets = |&(_, _, on_bg, on_fill): &(f32, Color32, f32, f32)| on_bg >= 3.0 && on_fill >= 3.0;
    // Otherwise move away from the background (lighter in dark mode): a pale
    // or deep tint reads clearly against the background, which the ring
    // borders, while the separator line sets it off from the fill.
    let away = |lightness: f32| {
        let bg_l = to_oklch(theme.colors.bg.base).0;
        (lightness - l) * (l - bg_l) >= 0.0
    };
    candidates
        .clone()
        .filter(meets)
        .min_by(|a, b| (a.0 - l).abs().total_cmp(&(b.0 - l).abs()))
        .or_else(|| {
            candidates
                .filter(|&(lightness, _, on_bg, _)| on_bg >= 3.0 && away(lightness))
                .max_by(|a, b| a.3.total_cmp(&b.3))
        })
        .map_or(primary.l6, |(_, color, _, _)| color)
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
        // Chosen by `correct_roles` once the solid trio is final.
        foreground: Color32::WHITE,
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
        color.foreground = foreground;
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

/// How much a failed [`ContrastCheck`] matters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    /// Fails a WCAG AA requirement for how components use the pair.
    Error,
    /// Fails a pairing that only some components or states use, or that
    /// another cue (such as the focus ring's separator line) backs up.
    Warning,
}

/// One color pair that components draw together, with its WCAG contrast.
#[derive(Clone, Debug, PartialEq)]
pub struct ContrastCheck {
    /// Token path of the drawn color, for example `"primary.l6"`.
    pub foreground: String,
    /// Token path of the color behind it, for example `"bg.base"`.
    pub background: String,
    pub ratio: f32,
    /// 4.5 for text, 3.0 for the focus ring on backgrounds, 9.0 between the
    /// ring and its separator.
    pub required: f32,
    pub severity: Severity,
}
impl ContrastCheck {
    pub fn passes(&self) -> bool {
        self.ratio >= self.required
    }
}

/// Every color pair that components rely on, passing or not. Translucent
/// colors are composited over the color behind them first.
///
/// Errors: `fg.*` text on every background; each scale's `l6` on every
/// background (outline and ghost button text); each scale's `foreground` on
/// its solid fills `l5`/`l6`/`l7`; the focus ring on every background.
/// Warnings: each scale's `l6` on its tint `l0` (surface button text), and
/// the focus ring on its separator line below 9:1. The ring and the black or
/// white separator form a two-color indicator (WCAG technique C40): at 9:1
/// apart, one of them has 3:1 against any fill, such as a light primary
/// button in dark mode where no single color can contrast with both the
/// background and the fill.
pub fn contrast_checks(theme: &Theme) -> Vec<ContrastCheck> {
    let c = &theme.colors;
    let backgrounds = [
        ("bg.base", c.bg.base),
        ("bg.subtle", c.bg.subtle),
        ("bg.accent", c.bg.accent),
    ];
    let scales = [
        ("primary", c.primary),
        ("gray", c.gray),
        ("red", c.red),
        ("green", c.green),
        ("blue", c.blue),
        ("yellow", c.yellow),
        ("orange", c.orange),
    ];
    let mut checks = Vec::new();
    let mut check =
        |foreground: &str, fg: Color32, background: &str, bg: Color32, required, severity| {
            let bg = over(bg, c.bg.base);
            checks.push(ContrastCheck {
                foreground: foreground.to_owned(),
                background: background.to_owned(),
                ratio: contrast_ratio(over(fg, bg), bg),
                required,
                severity,
            });
        };
    use Severity::*;
    for (name, color) in [
        ("fg.base", c.fg.base),
        ("fg.subtle", c.fg.subtle),
        ("fg.accent", c.fg.accent),
    ] {
        for (bg_name, bg) in backgrounds {
            check(name, color, bg_name, bg, 4.5, Error);
        }
    }
    for (scale, color) in scales {
        let token = |level: &str| format!("{scale}.{level}");
        let l6 = token("l6");
        for (bg_name, bg) in backgrounds {
            check(&l6, color.l6, bg_name, bg, 4.5, Error);
        }
        let foreground = token("foreground");
        for (level, fill) in [("l5", color.l5), ("l6", color.l6), ("l7", color.l7)] {
            check(
                &foreground,
                color.foreground,
                &token(level),
                fill,
                4.5,
                Error,
            );
        }
        check(&l6, color.l6, &token("l0"), color.l0, 4.5, Warning);
    }
    for (bg_name, bg) in backgrounds {
        check(
            "focus_ring",
            theme.focus_ring.color,
            bg_name,
            bg,
            3.0,
            Error,
        );
    }
    let ring = theme.focus_ring;
    check(
        "focus_ring",
        ring.color,
        "focus_ring.separator",
        ring.separator_color(),
        9.0,
        Warning,
    );
    checks
}

/// The failing [`contrast_checks`], errors first. Empty means every pair passes.
pub fn audit(theme: &Theme) -> Vec<ContrastCheck> {
    let mut issues: Vec<_> = contrast_checks(theme)
        .into_iter()
        .filter(|check| !check.passes())
        .collect();
    issues.sort_by_key(|issue| issue.severity == Severity::Warning);
    issues
}

/// Source-over compositing in sRGB, as browsers and egui blend.
fn over(fg: Color32, bg: Color32) -> Color32 {
    let [r, g, b, a] = fg.to_srgba_unmultiplied();
    if a == 255 {
        return fg;
    }
    let a = a as f32 / 255.0;
    let mix = |f: u8, b: u8| (f as f32 * a + b as f32 * (1.0 - a)).round() as u8;
    Color32::from_rgb(mix(r, bg.r()), mix(g, bg.g()), mix(b, bg.b()))
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
            for (t, dark) in [(&theme.light, false), (&theme.dark, true)] {
                let ring = t.focus_ring.color;
                for bg in [t.colors.bg.base, t.colors.bg.subtle, t.colors.bg.accent] {
                    assert!(contrast_ratio(ring, bg) >= 3.0, "seed {seed:?}");
                }
                // Two-color indicator (C40): visible on any fill.
                assert!(
                    contrast_ratio(ring, t.focus_ring.separator_color()) >= 9.0,
                    "seed {seed:?}"
                );
                // A light primary on a dark background leaves no room for both.
                if !dark {
                    assert!(
                        contrast_ratio(ring, t.colors.primary.l6) >= 3.0,
                        "seed {seed:?}"
                    );
                }
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
                            contrast_ratio(c.foreground, bg) >= 4.5,
                            "seed {seed:?}, contrast {}",
                            contrast_ratio(c.foreground, bg)
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn gallery_theme_uses_the_derived_focus_color() {
        let base = UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json")).unwrap();
        for t in [&base.light, &base.dark] {
            assert_eq!(t.focus_ring.color, auto_focus_color(t));
        }
    }
    #[test]
    fn focus_ring_options_override_the_derived_color() {
        let base = UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json")).unwrap();
        let theme = generate(
            &base,
            Options {
                focus_ring_color_dark: Some(Color32::RED),
                focus_ring_width: 3.0,
                ..Default::default()
            },
        );
        assert_eq!(theme.dark.focus_ring.color, Color32::RED);
        // Each mode is independent: light stays derived.
        assert_eq!(theme.light.focus_ring.color, auto_focus_color(&theme.light));
        for t in [&theme.light, &theme.dark] {
            assert_eq!(t.focus_ring.width, 3.0);
        }
    }
    #[test]
    fn contrast_endpoints() {
        assert!((contrast_ratio(Color32::WHITE, Color32::BLACK) - 21.0).abs() < 0.001);
    }

    #[test]
    fn generated_themes_pass_the_audit() {
        let base = UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json")).unwrap();
        for seed in [
            Color32::from_rgb(107, 101, 245),
            Color32::from_rgb(255, 230, 0),
            Color32::from_rgb(0, 200, 255),
            Color32::from_rgb(230, 20, 20),
            Color32::from_rgb(20, 20, 20),
            Color32::from_rgb(240, 240, 240),
        ] {
            let generated = generate(
                &base,
                Options {
                    seed,
                    ..Default::default()
                },
            );
            for theme in [&generated.light, &generated.dark] {
                assert!(
                    audit(theme).is_empty(),
                    "seed {seed:?}: {:#?}",
                    audit(theme)
                );
            }
        }
    }

    #[test]
    fn gallery_theme_json_passes_every_contrast_check() {
        let theme = UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json")).unwrap();
        for theme in [&theme.light, &theme.dark] {
            assert!(audit(theme).is_empty(), "{:#?}", audit(theme));
        }
    }

    #[test]
    fn audit_reports_failing_pairs_errors_first() {
        let mut theme = UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
            .unwrap()
            .light;
        theme.colors.fg.base = theme.colors.bg.base;
        // Passes 3:1 on the white background but is too close to its separator.
        theme.focus_ring.color = Color32::from_gray(0x77);
        let issues = audit(&theme);
        assert_eq!(issues[0].foreground, "fg.base");
        assert_eq!(issues[0].background, "bg.base");
        assert!((issues[0].ratio - 1.0).abs() < 0.001);
        assert_eq!(issues[0].severity, Severity::Error);
        let last = issues.last().unwrap();
        assert_eq!(
            (last.foreground.as_str(), last.background.as_str()),
            ("focus_ring", "focus_ring.separator")
        );
        assert_eq!(last.severity, Severity::Warning);
    }

    #[test]
    fn translucent_colors_are_composited_before_measuring() {
        let half_black = Color32::from_rgba_unmultiplied(0, 0, 0, 128);
        assert_eq!(
            over(half_black, Color32::WHITE),
            Color32::from_rgb(127, 127, 127)
        );
    }
}
