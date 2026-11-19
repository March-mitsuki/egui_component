# ECML todo app

A TodoMVC-style list built with ECML: add todos with Enter, edit them, complete
and delete them, filter by All / Active / Completed, complete all, and clear
completed ones. It is the acceptance example for ECML lists, conditionals and text
input events.

```sh
cargo run --features ecml --example ecml_todo_app
```

Edit [todos.ecml](todos.ecml) while the app runs; the todos you entered survive the
reload. [main.rs](main.rs) holds the state and every registration:

- `.list("todos", read, find, key, rows)` exposes the todos to
  `e-for="todo in todos"`. `read` applies the current filter, so the template has
  no filtering logic. Rows are identified by `id`: editing one title keeps its
  focus when other rows are added, removed or completed.
- Inside the loop, `todo.done` and `todo.draft` are row models, `todo.title` and
  `todo.editing` are row bindings, and `todo.edit`, `todo.save`, `todo.cancel` and
  `todo.remove` are row handlers that receive the row's id.
- The entry field has `autofocus="true"` and `@submit="add"`: it is focused on
  start and keeps focus after Enter. Edit swaps the title for a field with
  `autofocus`, `@submit` and `@blur` saving (an empty title deletes the todo) and
  `@cancel` restoring it on Escape. TodoMVC enters editing on double-click; ECML
  text has no events yet, so an Edit button starts it.
- `<fragment e-if="has_todos">` hides the list and footer while there are no
  todos, and `e-if="any_done"` shows "Clear completed" only when something is
  done. The active filter is still shown as a disabled button, since ECML has no
  dynamic styling yet. The window scrolls because the host wraps `View::show` in an egui
  `ScrollArea`; ECML layouts do not scroll by themselves.

`cargo test --features ecml --example ecml_todo_app` drives the template through
adding, filtering and clearing todos. For integration, hot reload and
troubleshooting, see the [ECML user guide](../ecml_hello_world/README.md); for the
syntax, see the gallery's ECML pages.
