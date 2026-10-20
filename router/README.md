# egui_component_router

Minimal typed SPA navigation for native and Web egui applications. This crate
is an unpublished workspace member and has no dependency on egui_component or egui.

Implement `Route` with `from_path` and `to_path`, then create a `Router` with an
initial native route and a repaint callback. Use `current`, `navigate`, `replace`,
`back`, and `forward`. Routes should map to distinct paths starting with `/`.

Native navigation uses in-memory history. Web reads `location.pathname` and uses
`pushState` / `replaceState`, with `popstate` handling browser back/forward.
Only pathname routing is supported; hash routing is not supported. Query strings
and fragments are not part of route matching. Route output must be a same-origin
absolute pathname without a query or fragment.

The Web initial route argument is ignored. Unknown paths are handled by your route
type (the example displays a 404 page while preserving the requested URL).
Listeners are removed when the router is dropped. Use one router per Web
application. Browser history traversal is asynchronous and may leave the app.

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

Deployment base paths, query parsing, scroll restoration, and route macros are deferred.

## Example

From the repository root:

```sh
cargo run --example router
cd examples/router
trunk serve
```

Open http://127.0.0.1:8081/ (or `/docs`, `/component`). The example uses the
library's own buttons, text and separators. The existing gallery is unchanged.
