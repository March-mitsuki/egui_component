# Changelog

Breaking changes and migration instructions are recorded here.

## 0.4.0

### Breaking changes

- Starting with `egui_component` 0.4.0, the supported egui version is 0.37. Update your direct `egui`, `eframe`, and `egui_extras` dependencies to 0.37 if you use them.
- Form and scroll area ID parameters now require `egui::AsIdSalt`, and `drawer::render` requires `Hash + Debug` for its ID. If your custom ID type previously implemented only `Hash`, add `#[derive(Debug)]` or implement `Debug` manually.
- `color_picker::Style` replaces `size: f32` with `width: f32` and `height: f32`. To keep a square swatch, set both to the previous size. Full struct literals also need `border_stroke` and `corner_radius`. Use `Style::new(theme).width(...).height(...)` to keep the default border and radius, or override them with `.border_stroke(...)` and `.corner_radius(...)`. The default swatch is still 32 × 32, and its transparency checkerboard keeps square tiles inside the rounded outline.
- `icon_button::Style` replaces the public `size: f32` field with `width: f32` and `height: f32`. Replace direct reads of `style.size` with the dimension you need. In struct literals, replace `size` with both fields. The `.size(value)` builder still sets both dimensions, while `.width(value)` and `.height(value)` set them independently.
- `button::Style` adds the public `content_align: egui::Align` field. Add `content_align: egui::Align::Center` to struct literals that list every field to preserve centered alignment. Include the field or use `..` in exhaustive destructuring patterns. Constructors default to `Center`. Use `.content_align(...)` with `Min`, `Center`, or `Max` to align the entire content group to the left, center, or right.
- The `Style` structs in `accordion`, `avatar`, `button`, `icon`, `icon_button`, `navigation`, `select`, `switch` add the public field `focus_color: egui::Color32`. In struct literals that list every field, add `focus_color: theme.colors.primary.normal()` to match the constructor default. In exhaustive destructuring patterns, include `focus_color` or use `..`.

## 0.3.0

### Breaking changes

- Toast state is now stored in egui's temporary Context data instead of a process-wide global manager. Separate Contexts have independent toast queues, positions, IDs, custom content, and animation state. Cloned handles of the same Context share state. Toasts are not serialized.
- `toast::create`, `custom`, `dismiss`, `set_position`, `info`, `success`, `warning`, and `error` now require `&egui::Context` as their first argument. Use the same Context for creation, dismissal, and rendering; a toast ID is only meaningful within its originating Context. `toast::renderer(ui, theme)` is unchanged and uses `ui.ctx()`.

```rust
// Before
toast::success("Saved");
toast::dismiss(id);

// After
toast::success(ui.ctx(), "Saved");
toast::dismiss(ui.ctx(), id);
```

For background tasks, clone the application's Context and pass a reference to it. Toast mutations request a repaint automatically. Custom UI callbacks may still create or dismiss toasts.
