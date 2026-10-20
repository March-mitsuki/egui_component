# egui_component

[English](README.md) | [中文](README.zh.md)

[![Crates.io](https://img.shields.io/crates/v/egui_component.svg)](https://crates.io/crates/egui_component)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

![egui_component dark screenshot](images/key_visual.dark.png)

![egui_component light screenshot](images/key_visual.light.png)

## 🚀 Introduction

**egui_component** is a modern, non-intrusive component library built specifically for [egui](https://github.com/emilk/egui). It provides a set of preset component styles and a comprehensive styling system, offering out-of-the-box utility while giving developers full control over component appearance.

Thanks to its non-intrusive design, you can seamlessly use it in any `eframe` or custom `egui` framework without modifying your existing UI architecture.

## ✨ Features

- **🧩 Non-Intrusive**: Does not force a takeover of global `Visuals`; can coexist perfectly with native egui controls.
- **✨ Modern Styling**: Out-of-the-box modern component styles to enhance your application's visual quality.
- **🎨 Flexible Styling System**: Highly decoupled; every visual attribute of each component is fully customizable.
- **🌓 Color Mode Friendly**: Native support for dark/light mode switching.

## 📦 Installation

Add the following dependency to your `Cargo.toml` file:

```toml
[dependencies]
egui_component = "0.1.0-alpha.2"
```

## 📝 Quick Start

`egui_component` follows egui's Immediate Mode design philosophy. If you are familiar with egui, getting started will feel very natural.

```rust
use egui_component::components::button;
use egui_component::theme::{UiTheme, ColorMode};

fn my_page(ui: &mut egui::Ui) {
    // 1. Load theme configuration (Recommended to load once during app initialization and store in global or persistent state)
    // Example code assumes you have a valid theme.json
    let ui_theme = UiTheme::from_json_bytes(include_bytes!("theme.json")).unwrap();

    // 2. Select the current color mode (e.g., using the dark theme)
    let theme = &ui_theme.dark;

    ui.horizontal(|ui| {
        // Use built-in presets for quick rendering and interaction handling
        if button::render(ui, "Solid", button::Style::new_solid_md(theme)).clicked() {
            println!("Solid button clicked!");
        }

        button::render(ui, "Surface", button::Style::new_surface_md(theme));
        button::render(ui, "Outline", button::Style::new_outline_md(theme));
        button::render(ui, "Ghost", button::Style::new_ghost_md(theme));
    });
}
```

## 📖 Examples

See `examples/` directory for more examples.

### Gallery in a browser

Install the WASM target and [Trunk](https://github.com/trunk-rs/trunk), then run from the repository root:

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk serve
```

Open <http://127.0.0.1:8080>. `trunk build --release` produces a static website in `dist/`.
The Web gallery uses WebGL and embeds its CJK font at compile time; it does not need access to local fonts at runtime. The bundled font adds approximately 19 MB before compression.
Native gallery: `cargo run --example gallery`.

The gallery is also the library's homepage: Home, Docs, and Components share a
header and use `egui_component_router` on both platforms. Desktop documentation
pages have a sidebar, scrollable content, and a page outline. Smaller windows
collapse the sidebar into Browse pages and move the outline above the content.
Try `/docs/getting-started`, `/components/button`, or
`/components/button?section=usage`. Unknown paths show an application 404 page.
For production hosting, serve assets normally and fall back to `index.html` for
page requests; Trunk's development SPA fallback is already enabled.


For your own app, use egui's default fonts or call `FONT_MANAGER.load_from_bytes` with bundled font bytes before `FONT_MANAGER.set_egui_fonts(&cc.egui_ctx)`. File and system font loading are native-only; theme JSON can be loaded on both platforms with `UiTheme::from_json_bytes`.

### Dynamic fonts

Call `FONT_MANAGER.apply(ui.ctx())` before rendering each frame. Unchanged versions are skipped independently for each context. `set_egui_fonts` remains an alias for applying updates. egui activates updates on the next pass; family lookups use the context's active fonts, so pending fonts cannot cause missing-family errors.

```rust,ignore
use egui_component::consts::{DEFAULT_FONT_FAMILY_ALIAS, FONT_MANAGER};
use egui_component::font::weight::FontWeight;

// After downloading or reading font bytes (application-owned I/O):
FONT_MANAGER.replace_family(DEFAULT_FONT_FAMILY_ALIAS, &[
    (regular_bytes.as_slice(), 0),
    (bold_bytes.as_slice(), 0),
])?;
FONT_MANAGER.load_from_bytes("cjk", &cjk_bytes, 0)?;
FONT_MANAGER.set_fallback_chain(&["cjk"]);
ctx.request_repaint(); // Wake an idle application after asynchronous loading.

// During the UI pass, before rendering components:
FONT_MANAGER.apply(ctx);
let family = FONT_MANAGER.get_egui_font_family(ctx, DEFAULT_FONT_FAMILY_ALIAS, FontWeight::Bold);
```

`load_from_bytes` replaces a matching family/weight/italic slot. `replace_family` validates the entire batch before replacing it; malformed data, empty batches and duplicate slots leave the previous family intact. `remove_face` and `remove_family` queue removals for the next `apply`. Fallback uses the closest weight in each configured family, then the base proportional fonts. Regular and italic faces have separate slots; `get_egui_font_family_with_style` selects the style.

Family lookup methods now take `&egui::Context` as their first argument and must be called during an egui pass. A standalone `FontManager` can serve multiple contexts with its own lookup methods. Components use the global `FONT_MANAGER`. Each context should have one font manager owner; to preserve application-defined fonts, initialize it with `FontManager::with_base_fonts(definitions)` or call `set_base_fonts`. Do not independently overwrite the context's fonts afterwards. Managed fonts do not change the base proportional/monospace chains.

Open the Gallery's **Text** page to switch fonts, toggle CJK fallback, remove the primary family, and try an invalid font at runtime.

The same page accepts a direct HTTP(S) font URL and downloads it asynchronously with `ehttp`. It includes Google Fonts Black Ops One as the default URL: paste a font URL and click **Load font**. The Gallery converts WOFF2 to OpenType with `wuff` before calling `replace_family`; these dependencies are confined to the example. TTF/OTF/TTC bytes are passed directly. Failed downloads or invalid fonts preserve the current font, and selecting another font or canceling discards late results. Web downloads require the font server to allow CORS. Google Fonts URLs may contain only a character subset; missing characters still use the configured fallback chain.

## 📄 LICENSE

This repository and all its included packages are released under the **MIT License**.
