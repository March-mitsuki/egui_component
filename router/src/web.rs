use super::NavigationError;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

fn error(value: JsValue) -> NavigationError {
    NavigationError(format!("Browser navigation failed: {value:?}"))
}
fn window() -> Result<web_sys::Window, NavigationError> {
    web_sys::window().ok_or_else(|| NavigationError("Browser window is unavailable".into()))
}
pub(super) fn path() -> Result<String, NavigationError> {
    window()?.location().pathname().map_err(error)
}

pub(super) struct Browser {
    window: web_sys::Window,
    listener: Closure<dyn FnMut(web_sys::Event)>,
}
impl Browser {
    pub(super) fn new(on_change: impl Fn() + 'static) -> Result<Self, NavigationError> {
        let window = window()?;
        let listener = Closure::wrap(
            Box::new(move |_: web_sys::Event| on_change()) as Box<dyn FnMut(web_sys::Event)>
        );
        // History traversal dispatches popstate; pushes update the router directly.
        window
            .add_event_listener_with_callback("popstate", listener.as_ref().unchecked_ref())
            .map_err(error)?;
        Ok(Self { window, listener })
    }
    pub(super) fn write(&self, path: &str, replace: bool) -> Result<(), NavigationError> {
        let history = self.window.history().map_err(error)?;
        // pushState/replaceState do not dispatch popstate.
        if replace {
            history.replace_state_with_url(&JsValue::NULL, "", Some(path))
        } else {
            history.push_state_with_url(&JsValue::NULL, "", Some(path))
        }
        .map_err(error)
    }
    pub(super) fn back(&self) -> Result<(), NavigationError> {
        self.window.history().map_err(error)?.back().map_err(error)
    }
    pub(super) fn forward(&self) -> Result<(), NavigationError> {
        self.window
            .history()
            .map_err(error)?
            .forward()
            .map_err(error)
    }
}
impl Drop for Browser {
    fn drop(&mut self) {
        let _ = self.window.remove_event_listener_with_callback(
            "popstate",
            self.listener.as_ref().unchecked_ref(),
        );
    }
}
