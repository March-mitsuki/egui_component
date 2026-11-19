# Contributing to ECML

[English](ECML_DESIGN.md) | [中文](ECML_DESIGN.zh.md)

This guide defines the goals, architectural boundaries, and behavioral contracts
for contributors to the optional `ecml` feature. Read it before changing the
language, binding API, renderer, or reload pipeline. For application integration
see the [ECML user guide](examples/ecml_hello_world/README.md); for supported
syntax, see the gallery's ECML page (`examples/gallery/ecml_docs/`). General
repository workflow is documented in [Contributing](CONTRIBUTE.md).

## Goals and boundaries

ECML (egui Component Markup Language) lets developers change supported UI
structure, text, and styling while an application runs, without recompiling Rust
or losing application state. Templates describe UI; Rust owns state and business
behavior. Changes to Rust handlers, registrations, components, or language support
still require compilation.

The runtime must remain host-independent. It accepts `egui::Ui` and this library's
`Theme`, reuses existing components, and leaves windows, input, rendering, and
repaint scheduling to the host. Do not introduce eframe, winit, or a renderer as
ECML runtime dependencies. eframe is only an example launcher. A host such as
bevy_egui must use a compatible egui version and honor repaint requests; the
current callbacks have no Send/Sync bounds, so integration must account for
main-thread storage. A runnable Bevy integration is not currently provided.

Keep ECML optional, with any future parser/watcher dependencies feature-gated and
platform-specific where necessary. Plain Rust component users must not need to
adopt ECML or an additional public layout API.

ECML is a Vue style custom markup language, not XML/HTML/Vue compatibility, CSS layout,
Rust code generation, reflection, or a scripting environment. Do not expose
arbitrary fields or callable Rust functions through strings.

## Module responsibilities

| File | Responsibility |
| --- | --- |
| `src/ecml/mod.rs` | Module declarations and re-exports only; no implementation |
| `src/ecml/runtime.rs` | Public value/binding types, builder, View lifecycle, rendering, model writes and event dispatch |
| `src/ecml/parser.rs` | Markup parsing, source offsets, grammar and input limits; no application-state access |
| `src/ecml/template.rs` | Registration/schema validation and compilation into typed renderable nodes |
| `src/ecml/source.rs` | Source selection, caller-side macro and native background file polling |
| `src/ecml/tests.rs` | Grammar, binding, layout, interaction, identity and reload regression tests |

The pipeline is source → parsed elements → schema/binding validation → renderable
nodes → existing component calls. Parse on load or replacement, not each frame.
Resolve binding and handler names during validation so drawing uses indices.
Keep the public `egui_component::ecml::*` paths stable when reorganizing internals.

## Language and naming conventions

- Component tags use the exact public component module name, such as `text_edit`.
  Do not add `input` or other aliases. A module's existence does not automatically
  make it a supported ECML component; an explicit adapter is required.
- `layout` and `fragment` are reserved built-in constructs: `layout` maps directly
  to egui, and `fragment` groups elements without a layout. Do not add
  public Row/Column components solely to support markup, or accept `row`/`column`
  as aliases.
- Preserve egui field names and snake_case. Structured properties use dotted
  paths such as `item_spacing.x`, not `gap_x` or `item_spacing_x`.
- The parser retains the complete attribute name. The schema explicitly defines
  each supported path, value type, default, and mapping. Dotted UI properties do
  not imply nested application-state binding or automatic Rust reflection.
- Use `e-` for directives, `:` for dynamic properties and `@` for events. Current
  expressions are registered names, text interpolation, bool negation for
  conditions, and `cond ? value : value` in styling attributes (see Dynamic
  styling). Computations belong in Rust getters.
- Reject unknown tags/attributes, duplicates, invalid values, unresolved names
  and type/capability mismatches. Never silently ignore unsupported syntax or
  perform lossy conversions.

The custom grammar requires one root and quoted attribute values; the bare
`e-else` is the only attribute without a value. It supports
self-closing elements, text, comments inside elements, and five named entities
(`amp`, `lt`, `gt`, `quot`, `apos`). Names use ASCII segments starting with a letter
or underscore, followed by letters, digits, underscores or hyphens; attribute
paths may join segments with dots, with optional `:`/`@` prefixes. Templates are
limited to 256 KiB and 64 element levels. Errors carry file, line and column;
text interpolation errors currently point to the containing element.

## Layout contract

Map layout settings to egui rather than inventing another layout engine. Specify
both child placement and space allocation when adding a property.

| Setting | Current behavior |
| --- | --- |
| `direction` | `top_down` by default; also supports `left_to_right` |
| `cross_align` | `min` by default; also `center` and `max`, mapped to egui alignment |
| `item_spacing.x`, `item_spacing.y` | Finite nonnegative logical points; omitted axes inherit independently from the parent Ui |
| Scope | Spacing changes affect the layout and descendants, never siblings or the surrounding Rust Ui |
| Horizontal allocation | Start at `ui.spacing().interact_size.y` height and grow with content, as `Ui::horizontal` does |
| Vertical allocation | Use available width and report the used region to the parent; do not implicitly stretch all children |

Axes remain physical x/y when direction changes. Main/cross justification and
wrapping remain false. Layout adds no scrolling, clipping, or content shrinking;
egui overflow and the enclosing clip apply. A text_edit without an explicit width
still uses available width. Do not accidentally give a horizontal row all the
remaining height: this can move subsequent controls outside the viewport.

Reverse directions, main alignment, justification, wrap and layout-container
width/height are not supported yet. Before adding them, define whether a size is
requested, minimum, maximum or fixed. Scroll areas, grids and panels need their
own contracts rather than implicit behavior in `layout`.

## State, interaction and identity

State belongs to the caller. View stores accessors and handlers, not long-lived
mutable field references or a JSON copy of application state. `.model()` registers
a borrowed getter and owned-value setter; `.bind()` registers a read-only getter;
`.on()` registers an explicit handler. Validate types without evaluating getters.
Current values are String, bool, i32 and i64; editable controls accept String for
text_edit and bool for switch and checkbox. Getters are evaluated on demand without dependency
tracking or caching and must not have business side effects. Setters should only
synchronously update state. External effects belong in explicit handlers, which
must not block rendering with long work.

A View represents one mount in one Context/viewport, shown once per pass. Use
separate Views for multiple mounts. Preserve this update contract:

1. At the first show of a new frame, adopt a valid pending template.
2. Render components and write model edits immediately.
3. Collect clicks and run handlers in tree order after traversal; handlers see
   model updates from that traversal.
4. Request repaint after edits or handlers. Later passes show the resulting state
   without resubmitting model writes or handlers. text_edit is read-only on those
   passes to prevent replaying edits.

This is at-most-once submission, not rollback for discarded egui passes. Template
replacement must not change the tree between passes of the same frame.

Explicit IDs must be nonempty and unique within a View. Identity combines an
instance namespace, viewport, explicit ID (or automatic structural path), and tag.
Compatible explicitly identified controls retain focus/editing state across
moves; changing the tag changes identity. Automatic IDs are not stable across
structural edits. Removing and reinserting a node does not guarantee clearing
egui memory. Future numeric editors must preserve intermediate text such as `-`
and `1.` separately from business values. Application-level drafts, save and
cancel remain Rust responsibilities.

## Sources, hot reload and diagnostics

`ecml::source!("path")` resolves against the caller's `CARGO_MANIFEST_DIR` and
selects its branch in the caller's compilation context:

| Caller configuration | Source |
| --- | --- |
| Native with debug assertions | File with hot reload |
| Native without debug assertions | Compile-time `include_str!` embedding |
| WASM, with or without debug assertions | Compile-time embedding |

`Source::inline` and native `Source::file` provide explicit overrides. The macro
packages resources, not semantic validation: `.load()` still parses/validates at
runtime. Initial errors return `Error`; no startup error placeholder exists.

The native worker polls every 150 ms, compares content and parses changes. It
holds only the latest candidate and never accesses State or handlers. The first
show supplies a Context/viewport for wakeup. The UI thread validates a candidate
at the next frame boundary and replaces the tree only on success. Invalid edits
or read failures keep the previous tree and expose an error window and
`last_error()`; a valid replacement clears the error. State and registered
handlers survive. View drop stops and joins the worker. `View::replace(text)`
also stages a replacement; parsing that explicit call is synchronous.

Retain bounded work, idle wakeup, atomic-save recovery and latest-candidate
behavior if replacing polling with an OS watcher. File and embedded sources must
share the same validation/rendering path. Browser hot reload would need a host
transport, not native filesystem polling.

## Loops (e-for)

This section defines the `e-for` contract. The syntax reference and an editable
demo are on the gallery's ECML page; changes to the behavior below must update
this document first.

### Template syntax

```html
<layout e-for="todo in todos" id="todo_row" direction="left_to_right" item_spacing.x="8">
    <checkbox e-model="todo.done" />
    <text>{{ todo.title }}</text>
    <button variant="ghost" @click="todo.remove">Delete</button>
</layout>
```

- `e-for="<alias> in <list>"` repeats the element itself and is allowed on any
  element except the root. Alias and list names follow the name-segment rules.
- Keys are not declared in templates. Rust provides them when registering the
  list, which keeps the expression language small and prevents position-as-key.
- `alias.name` resolves only to row bindings or row handlers explicitly
  registered for that list; it is not field reflection or a nested state path.
  Unprefixed names still resolve to root registrations.
- Row events are written `@click="todo.remove"`; call syntax such as
  `remove(todo)` is not supported. The alias determines which row a handler gets.
- No index is exposed. Ordinals and other derived values belong in Rust row getters.

### Rust registration

```rust
.list(
    "todos",
    |s| Box::new(s.todos.iter().filter(|t| s.filter.matches(t))), // read: Rust filters and sorts
    |s, id| s.todos.iter_mut().find(|t| t.id == *id),             // write: locate by key
    |t| t.id,                                                     // key: Clone + Eq + Hash + Debug + 'static
    |row| row
        .model("done", |t| &t.done, |t, v| t.done = v)
        .bind("title", |_s, t| t.title.clone())
        .bind("selected", |s, t| s.selected == Some(t.id))
        .on("remove", |s, id| s.todos.retain(|t| t.id != *id)),
)
```

- The read accessor returns an iterator borrowing `&S`
  (`Box<dyn Iterator<Item = &T>>`), so Rust can filter and sort and storage need
  not be a Vec. Reads must keep returning an iterator. Keys need `Debug` so that
  duplicate keys can be reported.
- Writes and handlers locate rows only by key: model writes call the lookup
  accessor (`for<'a> Fn(&'a mut S, &K) -> Option<&'a mut T>`); `None` drops the write and
  requests a repaint. Row handlers are `FnMut(&mut S, &K)`. Never locate by
  position, since earlier handlers may have added or removed rows.
- Row `bind` getters receive `(&S, &T)` for values derived from global state; row
  `model` getters return a value borrowed from `&T`, and setters receive only
  `&mut T` and write synchronously.
- Row registration names are unique within their list; list names share the
  namespace of root bindings and handlers and must not collide with them.

### Runtime semantics

- The iterator borrows `&S` during traversal. All model writes inside a loop body
  (row and root models) are buffered and committed in tree order after that loop
  finishes: row writes through the lookup accessor, root writes through root setters.
- Within the same pass, nodes in the same loop after a write therefore still read
  the old value. The write requests a repaint and the next pass shows the new
  value; this is an accepted trade-off that avoids mutable borrow conflicts and
  per-row re-lookup.
- Clicks still run after the whole tree traversal in component-tree order; row
  events carry a cloned key. The at-most-once-per-frame commit and read-only
  later passes are unchanged.
- Duplicate keys cannot be found at compile time. At runtime only the first row
  with a key renders; later duplicates are skipped and reported with the list
  name and key through the error window and `last_error()`, never silently. This
  runtime error is recomputed on every pass and clears once the keys are unique;
  a template error takes precedence in `last_error()`. Rows are type-erased
  internally (`dyn Any`), and keys are cloned into an `Rc` for buffered writes and
  row events.
- No virtualization; scrolling long lists belongs to a separately specified `scroll_area`.

### Identity

- Row identity = loop node identity (explicit `id` or automatic structural path)
  + hash of the row key; nodes inside a row continue with explicit IDs or
  structural paths within the row template.
- Adding, removing, sorting or inserting rows therefore does not affect focus or
  edit state of other rows.
- Prefer an explicit `id` on the `e-for` element; otherwise hot reload that
  restructures outer nodes changes loop identity and loses focus inside rows.
- Explicit IDs remain unique within the View; an ID in a row template counts once
  and is combined with the row key at runtime. In v1, `View::response(id)` does
  not return nodes inside loops; a list-and-key lookup API may be added later.

### Compile-time errors (v1)

Report with location: unknown list; malformed `e-for` value; alias colliding with
a root registration or outer alias; alias used outside its loop; unregistered
`alias.name` or `alias.list`; `e-model` on a read-only row binding; value type
mismatch; `e-for` on the root; duplicate row registration names; a list name that
collides with a binding or handler at the same level; `e-for` together with a
conditional directive (see below).

### Nested loops

Nesting uses the same explicit registration, not reflection: a row builder
registers a child list, and `e-for="item in group.items"` resolves `group.items`
to that registration exactly as `group.name` resolves to a row binding.

```html
<layout e-for="group in groups" id="group">
    <text size="heading">{{ group.name }}</text>
    <layout e-for="item in group.items" id="item" direction="left_to_right">
        <text>{{ item.label }}</text>
        <button @click="item.remove">Delete</button>
    </layout>
</layout>
```

```rust
.list("groups", |s| Box::new(s.groups.iter()),
      |s, name| s.groups.iter_mut().find(|g| g.name == *name), |g| g.name.clone(),
      |group| group
          .bind("name", |_, g| g.name.clone())
          .list("items",
                |_s, g| Box::new(g.items.iter()),          // read: (&S, &parent row)
                |g, id| g.items.iter_mut().find(|i| i.id == *id), // find: in &mut parent row
                |i| i.id,
                |item| item
                    .bind("label", |_, i| i.label.clone())
                    .on("remove", |s, (group, id)| { /* key path */ })))
```

- A child list's read accessor receives `(&S, &T)` for its parent row `T`; its
  lookup receives `&mut T` and the child key, so writes locate rows level by
  level from the root. The rows of a child list have the key path
  `(ParentPath, ChildKey)` as their key type: row handlers receive
  `(&mut S, &(K1, K2))` at depth two and `&((K1, K2), K3)` at depth three. The
  key function still returns only the child's own key.
- Templates name values explicitly: `{{ item.label }}`, not `{{ item }}`.
- `alias.list` may name any alias in scope, not only the innermost. Inner loops
  keep access to outer aliases (`group.name` inside the item loop); an inner
  alias must not repeat an outer one.
- Writes anywhere inside the outermost loop are buffered until that loop
  finishes. Duplicate keys are checked among the rows of one parent row; the
  same child key may appear under different parents. Row identity chains: loop
  identity + outer key hash + inner loop identity + inner key hash.

## Conditionals (e-if) and fragments

```html
<layout direction="left_to_right" item_spacing.x="8">
    <checkbox e-model="todo.done" />
    <fragment e-if="todo.editing">
        <text_edit id="edit" e-model="todo.draft" />
        <button @click="todo.save">Save</button>
    </fragment>
    <text e-else-if="todo.done">{{ todo.title }} (done)</text>
    <text e-else>{{ todo.title }}</text>
</layout>
```

- `e-if="name"` and `e-else-if="name"` take one registered bool, root or
  `alias.name`, optionally negated with `!`, exactly like `:disabled`. Computations
  belong in Rust `bind` getters. `e-else` is the only bare attribute in the
  grammar and takes no value.
- `e-else-if` and `e-else` must directly follow an `e-if` or `e-else-if` sibling
  element; comments may sit between them. A chain renders at most its first true
  branch, and `e-else` ends a chain.
- Conditions are evaluated once per pass when the chain is drawn. Outside loops
  earlier writes are immediate, so a later condition sees them in the same pass;
  inside loops writes are buffered, so conditions see them on the next pass. The
  tree's structure stays fixed between passes; only branch selection changes.
- A hidden branch draws nothing and allocates no space; `View::response(id)`
  returns `None` for its nodes. Each branch is its own template element with its
  own structural path, so switching branches never changes siblings' automatic
  IDs. Hiding does not guarantee clearing egui memory (like removal), and a
  focused control loses focus when hidden. Explicit IDs remain unique within the
  View, across branches too.
- `e-if` is allowed on the root (a false root draws nothing); `e-else` and
  `e-else-if` are not, since the root has no siblings.
- An element carries at most one of `e-if`, `e-else-if` and `e-else`. A
  conditional directive and `e-for` on the same element are rejected; nest them
  instead, with a fragment when no layout is wanted. Filtering rows belongs in the
  list's read accessor.

`fragment` is a reserved built-in like `layout`: it groups elements without
creating a layout. Its children are drawn directly into the parent Ui and inherit
its direction and spacing, so a fragment inside a horizontal layout keeps its
children in the row. It accepts only `id` and the directives `e-if`,
`e-else-if`, `e-else` and `e-for`, and no text content. An `id` identifies the
fragment for loops (row identity = fragment identity + key); its children keep
their own explicit IDs or structural paths. A fragment may be the root, so a
template can draw several top-level elements into the host Ui. There is no `<>`
shorthand.

Compile-time errors, with location: `e-else`/`e-else-if` without a preceding
conditional sibling, or on the root; a value on `e-else`; an empty, unknown or
non-bool condition; several conditional directives on one element; a
conditional directive together with `e-for`; any other attribute or text on a
fragment.

## Dynamic styling

```html
<text :color="todo.done ? $theme.colors.gray.l6 : $theme.colors.fg.base"
      :strikethrough="todo.done">{{ todo.title }}</text>
<button :variant="showing_all ? 'outline' : 'ghost'" color="$theme.colors.red">All</button>
<button :variant="!enabled ? 'ghost' : at_limit ? 'surface' : 'solid'">Chained</button>
```

- A styling attribute is a constant or, with a `:` prefix, an expression. A
  constant is a bare word (`variant="ghost"`) or a theme path
  (`color="$theme.colors.gray.l6"`). An element cannot carry both `name` and
  `:name`.
- The expression grammar is deliberately small:
  `expr := value | cond "?" value ":" expr`, where `value` is a quoted word
  (`'ghost'`, single quotes because the attribute uses double quotes) or a theme
  path, and `cond` is a registered bool, root or `alias.name`, optionally negated
  with `!`, as in `e-if`. The value chosen when a condition holds is a single value;
  chaining happens in the else position and the first true condition wins. Bool
  attributes (`underline`, `strikethrough`, `disabled`) take `true`/`false` or,
  as `:name`, a condition directly. There is no arithmetic, comparison or call;
  computations belong in Rust `bind` getters.
- `$theme` is reserved. Its paths mirror the Rust `Theme` fields:
  `$theme.colors.<scale>.<foreground|l0..l9>` and
  `$theme.colors.<bg|fg>.<base|subtle|accent>` are colors, and
  `$theme.colors.<scale>` is a whole scale (button `color`). Scales are primary,
  gray, red, green, blue, yellow and orange. Paths are matched against a fixed
  table when the template compiles (no reflection) and looked up in the Theme
  passed to `View::show`, so a light/dark switch restyles without recompiling.
  Other `Theme` fields are not exposed yet.
- Every value is type-checked at compile time against its attribute: words must
  be one of the attribute's choices, colors must be color paths and scales must
  be scale paths. Rust bindings cannot return colors or variants yet.
- Supported now: text `size`, `color`, `underline`, `strikethrough`; button
  `size`, `variant`, `color`; text_edit `size`, `variant`; switch and checkbox
  `size`. Text lines are 1 point wide in the text color, mapping to
  `text::Style::underline`/`strikethrough`. Layout attributes and `width` stay
  constant.
- Expressions are evaluated each pass when the element is drawn, with the same
  read rules as `:disabled`: inside loops, buffered writes show on the next pass.
  Styling never changes identity.

## Text input events and focus

```html
<text_edit id="new" e-model="draft" autofocus="true" @submit="add" />
<text_edit e-if="todo.editing" id="edit" e-model="todo.draft" autofocus="true"
           @submit="todo.save" @blur="todo.save" @cancel="todo.cancel" />
```

| Event | Fires when the focused text_edit loses focus because of |
| --- | --- |
| `@submit` | Enter (single-line only) |
| `@cancel` | Escape |
| `@blur` | anything else, such as a click elsewhere or Tab |

- Every focus loss fires exactly one handler: the specific one when it is
  registered, otherwise `@blur`. So `@submit="save" @blur="save"` saves once, and
  Escape never runs a `@blur` save when `@cancel` is registered. In egui, Enter
  and Escape end editing, unlike in a browser. Detection follows egui's idiom
  (`lost_focus()` plus the key pressed this frame), as in `Form`; Enter that
  confirms an IME composition does not submit. egui keeps `lost_focus()` true for
  two frames, so with one pass per frame (as on the web) a View skips a loss it
  already reported on the previous frame unless the field was focused again in
  between; keep the single-pass regression test.
- Handlers take no arguments, like `@click`; root and row handlers
  (`todo.save`) are allowed. They run with clicks after the traversal, in tree
  order, on the first pass of a frame only; edits typed in the same frame (and
  buffered loop writes) are committed first. Disabled and read-only fields fire
  nothing.
- A text_edit with `@submit` keeps focus after Enter (focus is requested again),
  so an entry field can add several items in a row. If the handler hides the
  field, it is gone on the next pass and focus ends there. Without `@submit`,
  egui's behavior is unchanged.
- `autofocus="true"` requests focus when the control appears: on the View's
  first show, when an `e-if` branch switches to it, or when its loop row is
  added. A View remembers which autofocus controls it drew on the previous frame;
  hot reload that keeps their identity does not refocus. If several appear in
  one frame, the last one drawn gets focus. Disabled fields are not focused. The
  request is repeated once on the next frame, before the field is drawn: a field
  first drawn unfocused can lose a late request on the web, where the text agent
  only takes keyboard input for a field that draws focused.
- Multiline input is not supported in ECML yet. When it is added to `text_edit`
  (mapped to `render_multiline`), Enter inserts a newline and `@submit` is
  rejected on multiline fields; `@cancel` and `@blur` keep their meaning.
  Single-line-only and multiline-only attributes are validated per mode.

## Extending ECML

For a new component/property, define its name, type, defaults, supported dynamic
bindings/events, scope, and identity implications. Add schema validation and a
renderer mapping to an existing component or native layout operation. Include a
positive test and a meaningful failure/behavior regression. Document new syntax
on the gallery's ECML page with an editable demo (a template in
`examples/gallery/ecml_docs/templates/` plus en/zh/ja copy), and update both
language versions of this design guide when contracts change. New syntax is
designed here first and moves to the gallery once implemented; runtime contracts
and invariants stay here. New ECML examples are only for complete application
scenarios and are named `ecml_*`.

Current adapters are layout, fragment, text, button, text_edit, switch and
checkbox, plus the `e-for`, `e-if`, `e-else-if` and `e-else` directives. Events
are `@click` and the text_edit events above. Other events (such as double-click
on text), multiline input, Rust-computed style values, other `$theme` fields, nested state paths, custom Rust slots, more
events/types/components, derive helpers, lazy models and browser hot reload are
future work, not accepted syntax. A derive helper must remain optional and
generate the same explicit registration model.

## Validation before submitting

Run from the repository root:

```sh
cargo fmt --all --check
cargo test --features ecml --lib
cargo check --features ecml --example ecml_hello_world
cargo check --release --features ecml --example ecml_hello_world
cargo test --features ecml --example ecml_todo_app
cargo check --features ecml --lib --target wasm32-unknown-unknown
```

Install the WASM target if needed. For UI changes, also run the example and check
that controls fit its default window, not just that responses exist. Preserve
regressions for multipass input/clicks, focus/cursor retention, isolated instances,
spacing inheritance and scope, invalid-template recovery, and idle file wakeup.
Also run `cargo test --example gallery`, which checks every gallery ECML demo.
Update English and Chinese design documents together; keep contracts and
implementation details here, syntax reference and demos in the gallery, and
application setup in the user guide.
