//! Typed navigation with in-memory native history and browser pathname history.
//!
//! On Web, construct one router per application. Routes own path parsing,
//! including unknown paths. The router does not depend on egui or a UI theme.

use std::{cell::RefCell, rc::Rc};

#[cfg(target_arch = "wasm32")]
mod web;

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
    Ok(())
}

pub struct Router<R: Route> {
    current: Rc<RefCell<R>>,
    #[cfg(not(target_arch = "wasm32"))]
    history: Vec<R>,
    #[cfg(not(target_arch = "wasm32"))]
    index: usize,
    #[cfg(target_arch = "wasm32")]
    browser: web::Browser,
}

impl<R: Route> Router<R> {
    /// Native starts at `initial`; Web reads the URL pathname.
    /// `on_change` should request an egui repaint when browser navigation occurs.
    /// Native UI-driven navigation already runs within the application's frame.
    pub fn new(initial: R, on_change: impl Fn() + 'static) -> Result<Self, NavigationError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
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
            let current = Rc::new(RefCell::new(R::from_path(&web::path()?)));
            let target = current.clone();
            let browser = web::Browser::new(move || {
                if let Ok(path) = web::path() {
                    *target.borrow_mut() = R::from_path(&path);
                    on_change();
                }
            })?;
            Ok(Self { current, browser })
        }
    }

    pub fn current(&self) -> R {
        self.current.borrow().clone()
    }

    /// Push a route; navigating to the current route is a no-op.
    pub fn navigate(&mut self, route: R) -> Result<(), NavigationError> {
        validate_path(&route.to_path())?;
        if self.current() == route {
            return Ok(());
        }
        #[cfg(target_arch = "wasm32")]
        self.browser.write(&route.to_path(), false)?;
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.history.truncate(self.index + 1);
            self.history.push(route.clone());
            self.index += 1;
        }
        *self.current.borrow_mut() = route;
        Ok(())
    }

    /// Replace the current entry without adding a back-button step.
    pub fn replace(&mut self, route: R) -> Result<(), NavigationError> {
        validate_path(&route.to_path())?;
        #[cfg(target_arch = "wasm32")]
        self.browser.write(&route.to_path(), true)?;
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.history[self.index] = route.clone();
        }
        *self.current.borrow_mut() = route;
        Ok(())
    }

    /// Web traverses browser history asynchronously, potentially leaving the app.
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
        ] {
            assert!(r.navigate(page(path)).is_err());
            assert!(r.replace(page(path)).is_err());
            assert_eq!(r.current(), page("/"));
        }
        r.forward().unwrap();
        assert_eq!(r.current(), page("/"));
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
