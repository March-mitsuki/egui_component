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
- `text_edit::Style` adds the public fields `disabled: bool` and `read_only: bool`. Add both fields with a value of `false` to full struct literals to preserve editable behavior. In exhaustive destructuring patterns, include both fields or use `..`. Constructors default both to `false`, and the `.disabled(...)` and `.read_only(...)` builders apply to both single-line and multiline inputs.
- `accordion::Style` adds the public field `show_body_guide: bool`. Add `show_body_guide: false` to full struct literals. In exhaustive destructuring patterns, include the field or use `..`. The guide defaults to off. Enable it with `.show_body_guide(true)`.
- `accordion::Style` replaces `padding: i8` with `header_padding_y: i8` for the padding above and below the title and `body_indent: i8` for the left indent when the guide is enabled. Both default to `12` and clamp negative values to zero. Replace `padding` with the two new fields in full struct literals and exhaustive destructuring patterns. Accordion titles now accept any `Into<egui::WidgetText>`. Pass `egui::RichText` to customize the title, and its explicit color also applies to the hover underline. The default header is transparent and borderless, with a title underline on hover. If you customize `hover_background`, a transparent value now inherits `background`.
- `egui_component` no longer enables `egui_extras`' `all_loaders` feature; it now enables only `svg`, which the built-in icons need. Applications that load other images, such as `avatar` URLs over `http(s)://` or `file://`, PNG/JPEG, GIF, or WebP, must enable the matching loaders on their own `egui_extras` dependency, for example `egui_extras = { version = "0.37", features = ["all_loaders"] }`, and keep calling `egui_extras::install_image_loaders(ctx)`. Without them, `avatar` falls back to the name initial.
- `field::render` now requires `input_content` to return the input control's `egui::Response`, and returns `InnerResponse<Response>` instead of `InnerResponse<R>`. Clicking the label focuses that control, and toggle controls such as `switch` also toggle. In most cases, remove the trailing `;` in the closure so it returns the widget's response; `form.field` and `form.field_with_error` already return it. To pass other data out of the closure, assign it to a variable declared outside. Field labels are no longer selectable, regardless of `label_style.selectable`, and are not Tab stops.

### Color picker behavior changes

- The color picker popup now uses a saturation/value plane, hue and optional alpha bars, and a ghost button to switch between HEX and RGB(A) inputs. Alpha input uses `0–1` instead of the previous `0–255` slider scale; RGB inputs remain integers in `0–255`. Update UI automation and user instructions that depend on the old popup layout or alpha scale.
- HEX input accepts an optional `#` and normalizes valid edits on Enter or focus loss. With alpha disabled, it displays `#rrggbb`; with alpha enabled, it displays `#rrggbbaa`. Valid edits still update the color immediately; invalid drafts preserve the last valid color. Validation outlines appear on Enter or focus loss. Error text is opt-in through `render_with_options` and `Options::error_message`, which maps `ColorInputError` to an application-localized `String`.
- This redesign introduces no source-breaking API changes: `render(ui, id, &mut Color32, Style)` and the existing `Style` fields/builders remain unchanged. The new options API and lossless `hex_to_rgba` / `rgba_to_hex` helpers are additive; existing callers need no code migration.

### Form behavior changes

- Pressing Enter in a single-line input registered with `form.field` or `form.field_with_error` now requests submission, so `submitted` becomes `true` when the form is valid. The input keeps focus after submitting. Multiline inputs still insert a newline, and Enter used to confirm an IME composition does not submit. To keep the previous behavior, use `Form::new(id).submit_on_enter(false)`.

### Text edit behavior changes

- In password inputs, Tab now moves from the input to the show/hide toggle, then to the next control; previously the toggle came first. Clicking the toggle with a pointer keeps focus in the input, while toggling with the keyboard keeps focus on the toggle.

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
