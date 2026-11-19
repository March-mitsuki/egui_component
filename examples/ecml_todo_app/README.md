# ECML todo app

A TodoMVC-style list built with ECML: add, edit, complete and delete todos, filter
by All / Active / Completed, complete all, and clear completed ones. It is the
acceptance example for lists (`e-for`).

```sh
cargo run --features ecml --example ecml_todo_app
```

Edit [todos.ecml](todos.ecml) while the app runs; the todos you entered survive the
reload. [main.rs](main.rs) holds the state and every registration:

- `.list("todos", read, find, key, rows)` exposes the todos to
  `e-for="todo in todos"`. `read` applies the current filter, so the template has
  no filtering logic. Rows are identified by `id`: editing one title keeps its
  focus when other rows are added, removed or completed.
- Inside the loop, `todo.done` and `todo.title` are row models and `todo.remove`
  is a row handler that receives the row's id.
- The active filter is shown as a disabled button, since ECML has no conditionals
  yet. The window scrolls because the host wraps `View::show` in an egui
  `ScrollArea`; ECML layouts do not scroll by themselves.

`cargo test --features ecml --example ecml_todo_app` drives the template through
adding, filtering and clearing todos. For integration, hot reload and
troubleshooting, see the [ECML user guide](../ecml_hello_world/README.md); for the
syntax, see the gallery's ECML pages.
