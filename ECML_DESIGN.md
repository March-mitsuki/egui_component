# Contributing to ECML

[English](ECML_DESIGN.md) | [中文](ECML_DESIGN.zh.md)

This guide defines the goals, architectural boundaries, and behavioral contracts
for contributors to the optional `ecml` feature. Read it before changing the
language, binding API, renderer, or reload pipeline. For application integration
and supported syntax, see the [ECML user guide](examples/ecml/README.md). General
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
- `layout` is a reserved built-in construct mapped directly to egui. Do not add
  public Row/Column components solely to support markup, or accept `row`/`column`
  as aliases.
- Preserve egui field names and snake_case. Structured properties use dotted
  paths such as `item_spacing.x`, not `gap_x` or `item_spacing_x`.
- The parser retains the complete attribute name. The schema explicitly defines
  each supported path, value type, default, and mapping. Dotted UI properties do
  not imply nested application-state binding or automatic Rust reflection.
- Use `e-` for directives, `:` for dynamic properties and `@` for events. Current
  expressions are registered names, text interpolation and bool negation for
  `:disabled`. Computations belong in Rust getters.
- Reject unknown tags/attributes, duplicates, invalid values, unresolved names
  and type/capability mismatches. Never silently ignore unsupported syntax or
  perform lossy conversions.

The custom grammar requires one root and quoted attribute values. It supports
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
text_edit and bool for switch. Getters are evaluated on demand without dependency
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

## Extending ECML

For a new component/property, define its name, type, defaults, supported dynamic
bindings/events, scope, and identity implications. Add schema validation and a
renderer mapping to an existing component or native layout operation. Include a
positive test and a meaningful failure/behavior regression, then update the user
guide and both language versions of this design guide when contracts change.

Current adapters are layout, text, button, text_edit and switch. Conditionals,
keyed loops, nested state paths, custom Rust slots, more events/types/components,
derive helpers, lazy models and browser hot reload are future work, not accepted
syntax. Loops require a collection/row-scope/event/key design. A derive helper
must remain optional and generate the same explicit registration model.

## Validation before submitting

Run from the repository root:

```sh
cargo fmt --all --check
cargo test --features ecml --lib
cargo check --features ecml --example ecml
cargo check --release --features ecml --example ecml
cargo check --features ecml --lib --target wasm32-unknown-unknown
```

Install the WASM target if needed. For UI changes, also run the example and check
that controls fit its default window, not just that responses exist. Preserve
regressions for multipass input/clicks, focus/cursor retention, isolated instances,
spacing inheritance and scope, invalid-template recovery, and idle file wakeup.
Update English and Chinese design documents together; keep implementation details
here and application setup/reference material in the user guide.
