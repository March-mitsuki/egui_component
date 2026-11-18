# Using ECML

ECML lets you describe an egui_component interface in a `.ecml` file. In native
development builds, saving that file updates the running interface while keeping
your Rust application state. ECML is experimental; the supported syntax is listed
below. To work on the feature itself, read the contributor design guide in
[English](../../ECML_DESIGN.md) or [中文](../../ECML_DESIGN.zh.md).

## Try the example

From the repository root, run:

```sh
cargo run --features ecml --example ecml
```

Enter a name, increment the counter, then edit [counter.ecml](counter.ecml) and
save. Change text, button variants or layout spacing: the interface reloads and
the name/count stay intact. The switch enables incrementing; the counter stops at
10 and Reset clears it. See [main.rs](main.rs) for the example application.

If you save an invalid template, the last valid interface remains visible and an
error window shows the file, line, column and reason. Fix the template and save
again to recover. The initial template must be valid for the application to start.

Template-only changes can reload. Changes to Rust code, registered fields/handlers
or ECML support require rebuilding and restarting the application.

## Add ECML to your application

Enable the `ecml` feature on a checkout containing ECML. For example, if this
repository is next to your application directory:

```toml
[dependencies]
egui_component = { path = "../egui_component", features = ["ecml"] }
```

Adjust the path to your checkout. Use the same compatible egui version as your
host integration (this checkout uses egui 0.37).

Create `ui/counter.ecml` relative to your application's `Cargo.toml`:

```html
<layout item_spacing.y="12">
    <text size="heading">Counter</text>
    <text_edit id="name" e-model="name" placeholder="Your name" width="280" />
    <text>Hello, {{ name }}</text>
    <text>Count: {{ count }}</text>
    <layout direction="left_to_right" item_spacing.x="8">
        <button :disabled="!can_increment" @click="increment">Increment</button>
        <button variant="outline" @click="reset">Reset</button>
    </layout>
</layout>
```

Register the values and actions the template is allowed to use. Keep the View and
state in your application rather than recreating them each frame:

```rust
use egui_component::{ecml, theme::Theme};

#[derive(Default)]
struct State {
    name: String,
    count: i32,
}

struct Counter {
    state: State,
    view: ecml::View<State>,
}

impl Counter {
    fn new() -> Result<Self, ecml::Error> {
        let view = ecml::View::<State>::builder()
            .model("name", |s| &s.name, |s, value| s.name = value)
            .bind("count", |s| s.count)
            .bind("can_increment", |s| s.count < 10)
            .on("increment", |s| s.count += 1)
            .on("reset", |s| s.count = 0)
            .load(ecml::source!("ui/counter.ecml"))?;
        Ok(Self { state: State::default(), view })
    }

    fn ui(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        self.view.show(ui, theme, &mut self.state);
    }
}
```

Call `Counter::ui` from your existing host's egui UI pass and supply a `Theme`,
for example the light/dark theme your other components already use. ECML does not
create windows or depend on eframe. The repository example uses eframe only to
launch a window; the same rendering entry point accepts a Ui from another host.

For bevy_egui, align egui versions and keep the View in main-thread/non-send
storage: its callbacks do not require Send/Sync. The host must process egui's
repaint requests for idle hot reload. A Bevy-specific runnable example is not yet
provided.

## Bind state and handle events

| API or syntax | Use |
| --- | --- |
| `.model(name, getter, setter)` | Expose readable/writable state; getter returns a reference, setter receives an owned value |
| `.bind(name, getter)` | Expose a read-only value or calculation; getter returns an owned value |
| `.on(name, handler)` | Register a synchronous Rust function or closure receiving `&mut State` |
| `e-model="name"` | Edit a registered model |
| `{{ name }}` | Display a registered model or binding in text |
| `:disabled="saving"` / `:disabled="!ready"` | Read a registered bool, optionally negated |
| `@click="save"` | Invoke a registered button handler |

Registered value types are String, bool, i32 and i64. text_edit requires a String
model; switch requires a bool model. Read-only bindings cannot be used with
e-model. Registration names use ASCII letters, digits and underscores, starting
with a letter or underscore. Nested state paths and general expressions are not
supported. Put calculations in `.bind()` getters instead of template expressions.

Keep getters free of business side effects and setters limited to synchronous
state updates. Model edits write back during drawing; click handlers run after
the View's traversal and can read those edits. Multiple egui passes do not submit
the same frame's writes/handlers again. Discarding a pass does not roll back
submitted changes. Offload long operations through application-owned queues;
ECML does not provide an async executor.

## Tags and attributes

Component tags match their Rust module names: use `text_edit`, not `input`.
`layout` is built into ECML and does not add a Rust layout component. Only the
following tags and attributes are supported. Every tag also accepts `id`.

| Tag | Attributes |
| --- | --- |
| `layout` | `direction="top_down\|left_to_right"`, `cross_align="min\|center\|max"`, `item_spacing.x`, `item_spacing.y` |
| `text` | `size="md\|heading"` |
| `button` | `size="xs\|sm\|md\|lg"`, `variant="solid\|surface\|outline\|ghost"`, `width`, `disabled`, `:disabled`, `@click` |
| `text_edit` | `size="xs\|sm\|md\|lg"`, `variant="subtle\|outline\|flushed"`, `width`, `placeholder`, `disabled`, `:disabled`, `e-model` |
| `switch` | `size="sm\|md\|lg"`, `disabled`, `:disabled`, `e-model` |

Size defaults to md, button variant to solid, text_edit variant to outline, and
interaction to enabled. `disabled="true"` or `"false"` sets a literal value;
do not combine it with `:disabled`. Widths and spacing are finite nonnegative
logical points. Button width follows the existing component's behavior.

Only layout accepts child elements. text/button accept text and interpolation;
text_edit placeholder also supports interpolation. Outer text whitespace is
trimmed; internal whitespace is preserved. Whitespace between layout children
is ignored.

Use one root, quoted attribute values, and closing tags or `/>`. Comments are
supported inside elements. Escape special characters with `&amp;`, `&lt;`,
`&gt;`, `&quot;`, or `&apos;`. ECML is custom markup, not a full XML/HTML parser.

## Arrange the interface

A layout defaults to `direction="top_down"` and `cross_align="min"`.
`left_to_right` arranges children horizontally. Cross-axis alignment is horizontal
for top_down and vertical for left_to_right.

```html
<layout item_spacing.y="12">
    <text>Actions</text>
    <layout direction="left_to_right" cross_align="center" item_spacing.x="8">
        <switch e-model="enabled" />
        <text>Enabled</text>
    </layout>
</layout>
```

Register `enabled` as a bool model before using that example.

`item_spacing.x` controls horizontal spacing and `item_spacing.y` vertical
spacing. Unspecified axes inherit their parent Ui's settings, including settings
from Rust. Overrides affect only the layout and descendants. They do not leak
into siblings or outside Rust UI. Axes do not swap when direction changes.

Horizontal layouts start at one standard control's height and grow with content.
Vertical layouts use available width and report their used area. Layout does not
automatically stretch every child, scroll, clip or shrink overflowing content.
A text_edit without width consumes available width; specify component widths
when placing multiple editors side by side.

Structured property names use dots: `item_spacing.x` is the only supported
spelling, not `item_spacing_x` or `gap_x`. `gap`, `align`, `row` and `column` are
not aliases. Layout-container sizes, reverse directions, main alignment,
justification and wrapping are not yet available and produce errors.

## Choose a template source

`ecml::source!("ui/counter.ecml")` resolves relative to the **calling crate's**
Cargo manifest directory, independent of the application's working directory.

| Build | Default source |
| --- | --- |
| Native with debug assertions (normally Debug) | File, reloaded after saves |
| Native without debug assertions (normally Release) | Embedded text, no external template required at runtime |
| WASM, including Debug | Embedded text; no browser hot reload |

The selection follows `debug_assertions`, not the profile's name. Embedded files
must exist at compile time and change only after rebuilding. Their syntax and
bindings are still validated when `.load()` runs.

Override the default when needed:

```rust,ignore
// Provided text on any platform:
let source = ecml::Source::inline("counter.ecml", template_text);
// Explicit file reload on native platforms (including Release):
let source = ecml::Source::file("ui/counter.ecml");
```

Relative paths passed to `Source::file` use the process working directory; unlike
the macro, this constructor does not prepend the caller's manifest directory.
Native file changes are checked about every 150 ms. UI replacement happens at a
new frame boundary. You can also call `view.replace(new_text)` to queue a template
replacement, including for inline sources.

## Preserve editing state and inspect errors

Give editable controls explicit IDs, such as `id="name"`. IDs must be nonempty
and unique within a View. Compatible controls with explicit IDs retain identity
when moved in a template; automatic IDs may change when structure changes.
Changing a tag changes identity. Removal/reinsertion does not guarantee clearing
all egui editing memory. Business values remain in your Rust state.

Use one persistent View per mounted location and call it once per pass in a
single Context/viewport. Create separate Views for multiple mounts, even if they
use the same template and local IDs. Sharing application state between them is
your application's choice.

- `.load()` returns `Result<View<State>, Error>`; handle initial errors yourself.
- `view.last_error()` exposes the most recent replacement error. A successful
  replacement clears it; a failed one keeps the old UI.
- `view.response("name")` returns the most recent response for an explicitly
  identified leaf component, useful for focus or interaction inspection.

If changes do not appear, check that the source is file-backed, the View is
retained and shown, and the host honors repaint requests. If an attribute is
rejected, consult the table above: arbitrary component fields are not exposed.
A read-only e-model or a mismatched model type must be corrected in registration.

Templates are limited to 256 KiB and 64 element levels. Conditionals, loops,
nested state bindings, custom Rust slots, lazy models, browser hot reload and
additional component adapters are not yet supported. Render other Rust UI
alongside the View when needed.
