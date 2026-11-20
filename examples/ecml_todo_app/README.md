# ECML todo app

A TodoMVC-style list built with ECML: add todos with Enter, edit them, complete
and delete them, filter by All / Active / Completed, mark all as complete, and clear
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
- "Mark all as complete" is a checkbox on a computed model: `all_done` reads
  whether every todo is done, and its setter sets every todo to the new value.
- The entry field has `autofocus="true"` and `@submit="add"`: it is focused on
  start and keeps focus after Enter. Edit swaps the title for a field with
  `autofocus`, `@submit` and `@blur` saving (an empty title deletes the todo) and
  `@cancel` restoring it on Escape. As in TodoMVC, double-clicking a title starts
  editing (`@double_click="todo.edit"`); an Edit button does the same from the
  keyboard. While a todo is edited, only its field is shown.
- Delete shows only while the pointer is over its row, as in TodoMVC:
  `:visible="$contains_pointer.todo"` reads the row layout (`id="todo"`), and
  `visible` keeps the button's space so the row does not shift. Keyboard access
  to a hidden Delete is not handled yet.
- `<fragment e-if="has_todos">` hides the list and footer while there are no
  todos, and `e-if="any_done"` shows "Clear completed" only when something is
  done.
- Styling follows state with `:` expressions: a done todo is gray and struck
  through (`:color="todo.done ? $theme.colors.gray.l6 : $theme.colors.fg.base"`
  and `:strikethrough="todo.done"`), and the active filter is an outline button
  (`:variant="showing_all ? 'outline' : 'ghost'"`).
- The window scrolls because the host wraps `View::show` in an egui `ScrollArea`;
  ECML layouts do not scroll by themselves.

`cargo test --features ecml --example ecml_todo_app` drives the template through
adding, filtering and clearing todos. For integration, hot reload and
troubleshooting, see the [ECML user guide](../ecml_hello_world/README.md); for the
syntax, see the gallery's ECML pages.
