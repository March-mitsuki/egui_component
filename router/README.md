# egui_component_router

Minimal typed SPA navigation for native and Web egui applications. This crate
shares its release version with egui_component and egui_component_style_macros,
and has no dependency on egui_component or egui.

Implement `Route` with `from_path` and `to_path`, then create a `Router` with an
initial native route and a repaint callback. Use `current`, `navigate`, `replace`,
`back`, and `forward`. Routes should map to distinct paths starting with `/`.

Native navigation uses in-memory history. Web reads `location.pathname` and uses
`pushState` / `replaceState`, with `popstate` handling browser back/forward.
Pathnames match routes; search params are stored separately in `Location`.
Hash routing is not supported. Route output must be an encoded same-origin absolute
pathname without a query or fragment. Browser fragments are ignored.

The Web initial route argument is ignored. Unknown paths are handled by your route
type (the example displays a 404 page while preserving the requested URL).
Listeners are removed when the router is dropped. Use one router per Web
application. Browser history traversal is asynchronous and may leave the app.

## Path parameters

Use an exact segment pattern. Captures are decoded as UTF-8; encoded `/` remains
inside a single parameter. `+` in a path is a literal plus. Matching malformed
percent escapes, invalid UTF-8, empty parameters or dot segments fails. Names must
be unique; trailing slashes are significant. There are no wildcard/optional segments.

```rust
use egui_component_router::PathPattern;
let pattern = PathPattern::new("/component/:name").unwrap();
let params = pattern.match_path("/component/text%20edit").unwrap();
assert_eq!(params.get("name"), Some("text edit"));
assert_eq!(pattern.build(&[("name", "a/b")]).unwrap(), "/component/a%2Fb");
```

Call the matcher in `Route::from_path` to construct an application's typed variant
(e.g. `ComponentDetail { name: String }`), and the builder in `Route::to_path`.
Applications choose pattern precedence and the fallback variant. `to_path` is
infallible, so application route values must satisfy the pattern builder's constraints.

## Search parameters and navigation

`SearchParams` follows form URL encoding: spaces serialize as `+`, literal plus
as `%2B`. It preserves pair order, duplicate keys and empty values. Invalid UTF-8
is decoded with replacement characters, matching the form parser's behavior.
`get` returns the first value; `get_all` reads duplicates; `set` replaces all values
for a key; `append` adds a pair; `remove` removes all matching pairs.

```rust
use egui_component_router::SearchParams;
let mut search = SearchParams::parse("?tab=preview&tag=a&tag=b");
search.set("tab", "code");
assert_eq!(search.get_all("tag").collect::<Vec<_>>(), ["a", "b"]);
```

- `router.current()` returns the typed route.
- `router.location()` returns route + search; `location.to_url()` generates both.
- `navigate_with_search(route, search)` pushes a complete location.
- `replace_with_search(route, search)` replaces the complete current entry.
- `search_params()` returns a copy. Modify it and call
  `set_search_params(search, HistoryMode::Push)` or `HistoryMode::Replace`.
- `navigate(route)` and `replace(route)` clear previous search params.
- `Router::with_location(location, callback)` supplies initial native params;
  Web initialization always reads pathname and search from the browser.

Both native history and browser history restore route + search together. Pushing
an identical location is a no-op; search-only changes create history entries too.
The original URL's query spelling may be normalized when written (e.g. `%20` to `+`).

## Hosting and 404 fallback

Direct requests such as `/component` must serve the SPA's `index.html`, preserving
the URL. The router then displays the matched page or the application's 404 page.
The example explicitly enables Trunk's SPA fallback (`no_spa = false`) and uses
root-relative asset URLs so nested unknown paths can also load the application.

Production hosting must configure the same fallback independently of this crate.
For example, with nginx serving the built directory:

```nginx
location / {
    try_files $uri $uri/ /index.html;
}
```

This static SPA fallback normally returns HTTP 200, even when the application
renders its 404 page. Returning a real HTTP 404 status requires server-side route
knowledge or an appropriate hosting error-page configuration; client-side Rust
cannot change the HTTP status of the document already loaded.

Deployment base paths, scroll restoration, and route macros are deferred.

## Example

From the repository root:

```sh
cargo run --example router
cd examples/router
trunk serve
```

Open http://127.0.0.1:8081/ (or `/docs`, `/component/button?tab=code`). The example uses the
library's own buttons, text and separators. The existing gallery is unchanged.
