# Using ECML

ECML lets you describe an egui_component interface in a `.ecml` file. In native
development builds, saving that file updates the running interface while keeping
your Rust application state. ECML is experimental. This guide covers running the
example and integrating ECML into an application. The syntax reference, with an
editable live demo for every tag, is the **ECML** page of the gallery
([online](https://march-mitsuki.github.io/egui_component/en/ecml/set-up-a-view), or
`cargo run --example gallery` → ECML). To work on the feature itself, read
the contributor design guide in [English](../../ECML_DESIGN.md) or
[中文](../../ECML_DESIGN.zh.md).

## Try the example

From the repository root, run:

```sh
cargo run --features ecml --example ecml_hello_world
```

Enter a name, increment the counter, then edit [counter.ecml](counter.ecml) and
save. Change text, button variants or layout spacing: the interface reloads and
the name/count stay intact. The switch enables incrementing; the counter stops at
10 and Reset clears it. The last button switches between the light and dark themes
(see [Switch between light and dark themes](#switch-between-light-and-dark-themes)).
See [main.rs](main.rs) for the example application.

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
            .model("name", |s| s.name.clone(), |s, value| s.name = value)
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

## Switch between light and dark themes

A template cannot replace the theme itself: `View::show` takes the `Theme` from
the host on every call, and `$theme` paths read whichever one it receives. Keep a
bool in the state, let the template change it, and pick the theme from it in the
host:

```rust
let view = ecml::View::<State>::builder()
    // A button with a handler...
    .on("toggle_theme", |s| s.dark = !s.dark)
    // ...or a switch editing the bool directly: <switch e-model="dark" aria_label="Dark mode" />
    .model("dark", |s| s.dark, |s, value| s.dark = value)
    // ...
```

```html
<button variant="ghost" @click="toggle_theme">{{ theme_label }}</button>
```

```rust
fn ui(&mut self, ui: &mut egui::Ui) {
    let (theme, mut visuals) = if self.state.dark {
        (&self.ui_theme.dark, egui::Visuals::dark())
    } else {
        (&self.ui_theme.light, egui::Visuals::light())
    };
    // egui's own visuals (panel fill, scrollbars, text selection) follow separately.
    visuals.panel_fill = theme.colors.bg.base;
    ui.ctx().set_visuals(visuals);
    self.view.show(ui, theme, &mut self.state);
}
```

The handler runs after the View is drawn, so the next frame shows the new theme;
the click already requests that repaint. Keep the `UiTheme` outside the state you
pass to `show`: `view.show(ui, &self.state.ui_theme.dark, &mut self.state)` borrows
the state twice and does not compile. This example uses a button with a handler;
the [todo app](../ecml_todo_app/README.md) uses a switch bound to the bool.

## Syntax reference

Bindings (`.model`, `.bind`, `.on`), tags, attributes, layout rules and syntax
rules are documented with live demos on the gallery's ECML page. Edit any demo
there to try a change before putting it in your own template.

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

## Troubleshooting

- `.load()` returns `Result<View<State>, Error>`; handle initial errors yourself.
  Later invalid replacements keep the last valid UI and are reported through
  `view.last_error()` and an error window (disable it with
  `.error_window(false)` to show errors in your own UI).
- If changes do not appear, check that the source is file-backed, the View is
  retained and shown, and the host honors repaint requests.
- If an attribute or name is rejected, check the gallery's ECML page and your
  registrations: arbitrary component fields and Rust functions are not exposed.

Use one persistent View per mounted location and call it once per pass. Render
other Rust UI alongside the View when ECML does not support something yet.
