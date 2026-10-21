//! Typed navigation with in-memory native history and browser pathname history.
//!
//! On Web, construct one router per application. Routes own path parsing,
//! including unknown paths. The router does not depend on egui or a UI theme.

use std::{cell::RefCell, rc::Rc};

#[cfg(target_arch = "wasm32")]
mod web;

mod params;
pub use params::{PathParams, PathPattern, SearchParams};

/// Application-defined route. Paths must be same-origin absolute pathnames (no query or hash) and round-trip through
/// `from_path`. Use an application route variant to represent unknown paths.
pub trait Route: Clone + PartialEq + 'static {
    fn from_path(path: &str) -> Self;
    fn to_path(&self) -> String;
}

/// A browser API failure. Navigation leaves the current route unchanged on error.
#[derive(Debug)]
pub struct NavigationError(pub(crate) String);

impl std::fmt::Display for NavigationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for NavigationError {}

fn validate_path(path: &str) -> Result<(), NavigationError> {
    if !path.starts_with('/')
        || path.starts_with("//")
        || path.contains(['\\', '?', '#'])
        || path.chars().any(char::is_control)
    {
        return Err(NavigationError(
            "Routes must use same-origin absolute pathnames without a query or hash".into(),
        ));
    }
    if path.split('/').any(|segment| {
        percent_encoding::percent_decode_str(segment)
            .decode_utf8()
            .is_ok_and(|value| value == "." || value == "..")
    }) {
        return Err(NavigationError(
            "Dot path segments are not supported".into(),
        ));
    }
    Ok(())
}

/// One complete navigation entry shared by native and Web.
#[derive(Clone, Debug, PartialEq)]
pub struct Location<R> {
    pub route: R,
    pub search: SearchParams,
}
impl<R: Route> Location<R> {
    pub fn new(route: R, search: SearchParams) -> Self {
        Self { route, search }
    }
    pub fn to_url(&self) -> String {
        let path = self.route.to_path();
        if self.search.is_empty() {
            path
        } else {
            format!("{path}?{}", self.search)
        }
    }
}

/// Whether a search-only update adds a history entry or replaces the current one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryMode {
    Push,
    Replace,
}

pub struct Router<R: Route> {
    current: Rc<RefCell<Location<R>>>,
    #[cfg(not(target_arch = "wasm32"))]
    history: Vec<Location<R>>,
    #[cfg(not(target_arch = "wasm32"))]
    index: usize,
    #[cfg(target_arch = "wasm32")]
    browser: web::Browser,
}

impl<R: Route> Router<R> {
    /// Native starts at `initial` with no search params; Web reads pathname and search.
    /// Pass an egui repaint callback for browser history events.
    pub fn new(initial: R, on_change: impl Fn() + 'static) -> Result<Self, NavigationError> {
        Self::with_location(Location::new(initial, SearchParams::new()), on_change)
    }

    /// Like `new`, with initial native search params. Web always reads the URL.
    pub fn with_location(
        initial: Location<R>,
        on_change: impl Fn() + 'static,
    ) -> Result<Self, NavigationError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            validate_path(&initial.route.to_path())?;
            let _ = on_change;
            Ok(Self {
                current: Rc::new(RefCell::new(initial.clone())),
                history: vec![initial],
                index: 0,
            })
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = initial;
            let current = Rc::new(RefCell::new(web::location::<R>()?));
            let target = current.clone();
            let browser = web::Browser::new(move || {
                if let Ok(location) = web::location::<R>() {
                    *target.borrow_mut() = location;
                    on_change();
                }
            })?;
            Ok(Self { current, browser })
        }
    }
    pub fn current(&self) -> R {
        self.current.borrow().route.clone()
    }
    pub fn location(&self) -> Location<R> {
        self.current.borrow().clone()
    }
    pub fn search_params(&self) -> SearchParams {
        self.current.borrow().search.clone()
    }

    /// Push a route with empty search params. An identical location is a no-op.
    pub fn navigate(&mut self, route: R) -> Result<(), NavigationError> {
        self.navigate_with_search(route, SearchParams::new())
    }
    pub fn navigate_with_search(
        &mut self,
        route: R,
        search: SearchParams,
    ) -> Result<(), NavigationError> {
        self.commit(Location::new(route, search), HistoryMode::Push)
    }
    /// Replace the current entry, clearing search params.
    pub fn replace(&mut self, route: R) -> Result<(), NavigationError> {
        self.replace_with_search(route, SearchParams::new())
    }
    pub fn replace_with_search(
        &mut self,
        route: R,
        search: SearchParams,
    ) -> Result<(), NavigationError> {
        self.commit(Location::new(route, search), HistoryMode::Replace)
    }
    /// Update search params on the current route. Read a copy with `search_params`,
    /// mutate it, then pass it here. Both path and search are restored by history.
    pub fn set_search_params(
        &mut self,
        search: SearchParams,
        mode: HistoryMode,
    ) -> Result<(), NavigationError> {
        self.commit(Location::new(self.current(), search), mode)
    }
    fn commit(&mut self, location: Location<R>, mode: HistoryMode) -> Result<(), NavigationError> {
        validate_path(&location.route.to_path())?;
        if mode == HistoryMode::Push && self.location() == location {
            return Ok(());
        }
        #[cfg(target_arch = "wasm32")]
        self.browser
            .write(&location.to_url(), mode == HistoryMode::Replace)?;
        #[cfg(not(target_arch = "wasm32"))]
        match mode {
            HistoryMode::Push => {
                self.history.truncate(self.index + 1);
                self.history.push(location.clone());
                self.index += 1;
            }
            HistoryMode::Replace => self.history[self.index] = location.clone(),
        }
        *self.current.borrow_mut() = location;
        Ok(())
    }
    /// Web traverses browser history asynchronously and may leave the app.
    /// Native does nothing at the beginning of its history.
    pub fn back(&mut self) -> Result<(), NavigationError> {
        #[cfg(target_arch = "wasm32")]
        self.browser.back()?;
        #[cfg(not(target_arch = "wasm32"))]
        if self.index > 0 {
            self.index -= 1;
            *self.current.borrow_mut() = self.history[self.index].clone();
        }
        Ok(())
    }
    /// Web traverses browser history asynchronously; native stops at the end.
    pub fn forward(&mut self) -> Result<(), NavigationError> {
        #[cfg(target_arch = "wasm32")]
        self.browser.forward()?;
        #[cfg(not(target_arch = "wasm32"))]
        if self.index + 1 < self.history.len() {
            self.index += 1;
            *self.current.borrow_mut() = self.history[self.index].clone();
        }
        Ok(())
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    #[derive(Clone, PartialEq, Debug)]
    struct Page(String);
    impl Route for Page {
        fn from_path(path: &str) -> Self {
            Self(path.into())
        }
        fn to_path(&self) -> String {
            self.0.clone()
        }
    }
    fn page(path: &str) -> Page {
        Page::from_path(path)
    }

    #[test]
    fn traversal_and_boundaries() {
        let mut r = Router::new(page("/"), || {}).unwrap();
        r.back().unwrap();
        r.navigate(page("/docs")).unwrap();
        r.navigate(page("/docs")).unwrap();
        r.navigate(page("/component")).unwrap();
        r.back().unwrap();
        assert_eq!(r.current(), page("/docs"));
        r.back().unwrap();
        assert_eq!(r.current(), page("/"));
        r.forward().unwrap();
        r.forward().unwrap();
        r.forward().unwrap();
        assert_eq!(r.current(), page("/component"));
    }

    #[test]
    fn invalid_navigation_preserves_route_and_history() {
        let mut r = Router::new(page("/"), || {}).unwrap();
        for path in [
            "docs",
            "//example.com/docs",
            "/docs#section",
            "/docs?q=1",
            "/\\other",
            "/docs/..",
            "/docs/%2e%2e",
        ] {
            assert!(r.navigate(page(path)).is_err());
            assert!(r.replace(page(path)).is_err());
            assert_eq!(r.current(), page("/"));
        }
        r.forward().unwrap();
        assert_eq!(r.current(), page("/"));
    }

    #[test]
    fn initial_search_and_full_location_replacement() {
        let mut r = Router::with_location(
            Location::new(page("/docs"), SearchParams::parse("q=hello")),
            || {},
        )
        .unwrap();
        assert_eq!(r.search_params().get("q"), Some("hello"));
        r.navigate_with_search(
            page("/component/button"),
            SearchParams::parse("tab=preview"),
        )
        .unwrap();
        r.replace_with_search(page("/component/card"), SearchParams::parse("tab=code"))
            .unwrap();
        r.back().unwrap();
        assert_eq!(r.location().to_url(), "/docs?q=hello");
        r.forward().unwrap();
        assert_eq!(r.location().to_url(), "/component/card?tab=code");
        r.replace(page("/docs")).unwrap();
        assert!(r.search_params().is_empty());
    }

    #[test]
    fn search_history_push_replace_and_route_navigation() {
        let mut r = Router::new(page("/"), || {}).unwrap();
        r.navigate_with_search(
            page("/component/button"),
            SearchParams::parse("tab=preview&tag=a&tag=b"),
        )
        .unwrap();
        r.set_search_params(SearchParams::parse("tab=code"), HistoryMode::Push)
            .unwrap();
        r.set_search_params(
            SearchParams::parse("tab=code&q=hello+world"),
            HistoryMode::Replace,
        )
        .unwrap();
        assert_eq!(
            r.location().to_url(),
            "/component/button?tab=code&q=hello+world"
        );
        r.back().unwrap();
        assert_eq!(
            r.search_params().get_all("tag").collect::<Vec<_>>(),
            ["a", "b"]
        );
        assert_eq!(r.search_params().get("tab"), Some("preview"));
        r.forward().unwrap();
        assert_eq!(r.search_params().get("q"), Some("hello world"));
        r.navigate(page("/docs")).unwrap();
        assert!(r.search_params().is_empty());
        r.back().unwrap();
        assert_eq!(r.search_params().get("tab"), Some("code"));
        r.back().unwrap();
        r.set_search_params(SearchParams::parse("tab=new"), HistoryMode::Push)
            .unwrap();
        r.forward().unwrap();
        assert_eq!(r.search_params().get("tab"), Some("new"));
    }

    #[test]
    fn replacement_and_branching() {
        let mut r = Router::new(page("/"), || {}).unwrap();
        r.navigate(page("/docs")).unwrap();
        r.replace(page("/component")).unwrap();
        r.back().unwrap();
        assert_eq!(r.current(), page("/"));
        r.navigate(page("/new")).unwrap();
        r.forward().unwrap();
        assert_eq!(r.current(), page("/new"));
        r.back().unwrap();
        r.forward().unwrap();
        assert_eq!(r.current(), page("/new"));
    }
}
