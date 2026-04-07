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

## 📄 LICENSE

This project and all its included packages are released under the **MIT License**.
