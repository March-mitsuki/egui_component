//! Forms can own drafts in egui's temporary storage or borrow application data.
//!
//! ```
//! use egui_component::form::Form;
//! fn editor(ui: &mut egui::Ui) {
//!     let result = Form::new("name_form").show::<String, _>(ui, |form, ui, name| {
//!         form.field(ui, "name", name,
//!             |ui, value| ui.text_edit_singleline(value),
//!             |value| if value.trim().is_empty() {
//!                 Err("Name is required".into())
//!             } else { Ok(()) });
//!         form.submit_button(ui, "Save");
//!         form.reset_button(ui, "Reset");
//!     });
//!     if result.submitted {
//!         let validated_name = result.data;
//!         // Commit the validated draft here.
//!     }
//! }
//! ```
//!
//! `show` manages both draft and interaction state, resetting the draft to
//! `Default` on reset. `show_with_data` borrows application data without type
//! bounds; `show_with_state` also borrows interaction state. Borrowed drafts
//! must be restored by the caller when `reset_requested` is true.
//!
//! Use stable instance and field keys. Cached values live within one egui
//! Context and are not saved across restarts. Use `remove::<D>` to end a managed
//! editing session. This replaces the global `FORM_CONTEXT` registry.

use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

use egui::{Id, Response, Ui};

/// Lightweight interaction state; contains no application data or async tasks.
#[derive(Clone, Debug, Default)]
pub struct FormState {
    touched: HashSet<Id>,
    submit_attempted: bool,
}

impl FormState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn submit_attempted(&self) -> bool {
        self.submit_attempted
    }
}

/// A form instance, scoped to the parent UI's ID.
pub struct Form {
    id_salt: Id,
}

impl Form {
    pub fn new(id_salt: impl Hash) -> Self {
        Self {
            id_salt: Id::new(id_salt),
        }
    }

    pub fn id(&self, ui: &Ui) -> Id {
        ui.make_persistent_id(self.id_salt)
    }

    /// Forget cached interaction state. Does not modify any draft.
    pub fn remove_state(&self, ui: &Ui) {
        let id = self.id(ui);
        ui.data_mut(|d| d.remove::<FormState>(id));
    }

    /// Forget a managed draft of type `D` and its cached interaction state.
    pub fn remove<D: 'static>(&self, ui: &Ui) {
        let id = self.id(ui);
        ui.data_mut(|d| {
            d.remove::<ManagedDraft<D>>(id);
            d.remove::<FormState>(id);
        });
    }

    /// Manage a draft and interaction state in egui, keyed by form ID and type.
    /// Returns a snapshot in `data`; reset restores `D::default()` automatically.
    /// No Context lock is held while running `content`.
    pub fn show<D, R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut FormUi<'_>, &mut Ui, &mut D) -> R,
    ) -> FormResponse<R, D>
    where
        D: Default + Clone + Send + Sync + 'static,
    {
        let id = self.id(ui);
        let mut data = ui
            .data(|d| d.get_temp::<ManagedDraft<D>>(id))
            .map(|draft| draft.0)
            .unwrap_or_default();
        let result = self.show_with_data(ui, &mut data, content);
        if result.reset_requested {
            data = D::default();
        }
        ui.data_mut(|d| d.insert_temp(id, ManagedDraft(data.clone())));
        result.with_data(data)
    }

    /// Borrow a draft without requiring `Default`, `Clone`, or `Send + Sync`.
    /// Only lightweight interaction state is cloned from/to egui's temporary map.
    pub fn show_with_data<D, R>(
        self,
        ui: &mut Ui,
        data: &mut D,
        content: impl FnOnce(&mut FormUi<'_>, &mut Ui, &mut D) -> R,
    ) -> FormResponse<R> {
        let id = self.id(ui);
        let mut state = ui.data(|d| d.get_temp::<FormState>(id).unwrap_or_default());
        let result = self.show_with_state(ui, data, &mut state, content);
        ui.data_mut(|d| d.insert_temp(id, state));
        result
    }

    /// Use application-owned interaction state instead of the egui cache.
    pub fn show_with_state<D, R>(
        self,
        ui: &mut Ui,
        data: &mut D,
        state: &mut FormState,
        content: impl FnOnce(&mut FormUi<'_>, &mut Ui, &mut D) -> R,
    ) -> FormResponse<R> {
        let id = self.id(ui);
        let mut form = FormUi {
            id,
            state,
            errors: HashMap::new(),
            changed: false,
            submit_requested: false,
            reset_requested: false,
        };
        // Scope auto-generated widget IDs as well as field-state IDs.
        let inner = ui.push_id(id, |ui| content(&mut form, ui, data)).inner;
        let valid = form.errors.is_empty();
        if form.reset_requested {
            form.state.reset();
            ui.ctx().request_repaint();
        } else if form.submit_requested {
            form.state.submit_attempted = true;
            // Fields rendered before the submit button show errors next pass.
            ui.ctx().request_repaint();
        }
        FormResponse {
            data: (),
            inner,
            changed: form.changed,
            valid,
            submitted: form.submit_requested && valid && !form.reset_requested,
            submit_requested: form.submit_requested,
            reset_requested: form.reset_requested,
            errors: form.errors,
        }
    }
}

// Wrapper prevents a managed `D = FormState` from colliding with interaction state.
#[derive(Clone)]
struct ManagedDraft<D>(D);

pub struct FormResponse<R, D = ()> {
    /// Snapshot of a managed draft after rendering/reset; `()` for borrowed drafts.
    pub data: D,
    pub inner: R,
    /// At least one registered field reported a change during this pass.
    pub changed: bool,
    /// All fields registered during this pass passed validation.
    pub valid: bool,
    pub submitted: bool,
    pub submit_requested: bool,
    /// Managed drafts reset automatically; borrowed drafts must be reset by the caller.
    pub reset_requested: bool,
    /// Current validation errors, including those not yet displayed.
    pub errors: HashMap<Id, String>,
}

impl<R, D> FormResponse<R, D> {
    fn with_data<T>(self, data: T) -> FormResponse<R, T> {
        FormResponse {
            data,
            inner: self.inner,
            changed: self.changed,
            valid: self.valid,
            submitted: self.submitted,
            submit_requested: self.submit_requested,
            reset_requested: self.reset_requested,
            errors: self.errors,
        }
    }
}

pub struct FormUi<'a> {
    id: Id,
    state: &'a mut FormState,
    errors: HashMap<Id, String>,
    changed: bool,
    submit_requested: bool,
    reset_requested: bool,
}

impl FormUi<'_> {
    pub fn field_id(&self, key: impl Hash) -> Id {
        self.id.with(key)
    }

    /// Render a field, then validate its updated value. Validation runs each pass;
    /// messages appear after focus loss or a submit attempt. Keys must be unique
    /// within a form. Hidden/unregistered fields do not participate in validation.
    pub fn field<V>(
        &mut self,
        ui: &mut Ui,
        key: impl Hash,
        value: &mut V,
        render: impl FnOnce(&mut Ui, &mut V) -> Response,
        validate: impl FnOnce(&V) -> Result<(), String>,
    ) -> Response {
        self.field_with_error(ui, key, value, render, validate, |ui, error| {
            ui.colored_label(ui.visuals().error_fg_color, error);
        })
    }

    /// Like `field`, with caller-provided error rendering for custom components.
    pub fn field_with_error<V>(
        &mut self,
        ui: &mut Ui,
        key: impl Hash,
        value: &mut V,
        render: impl FnOnce(&mut Ui, &mut V) -> Response,
        validate: impl FnOnce(&V) -> Result<(), String>,
        render_error: impl FnOnce(&mut Ui, &str),
    ) -> Response {
        let id = self.field_id(key);
        let response = ui.push_id(id, |ui| render(ui, value)).inner;
        self.changed |= response.changed();
        if response.lost_focus() {
            self.state.touched.insert(id);
        }
        if let Err(error) = validate(value) {
            if self.state.submit_attempted || self.state.touched.contains(&id) {
                render_error(ui, &error);
            }
            self.errors.insert(id, error);
        }
        response
    }

    /// Register a submit action from any styled button or keyboard shortcut.
    pub fn request_submit(&mut self) {
        self.submit_requested = true;
    }

    pub fn submit_button(&mut self, ui: &mut Ui, label: impl Into<egui::WidgetText>) -> Response {
        let response = ui.button(label);
        if response.clicked() {
            self.request_submit();
        }
        response
    }

    /// Clear interaction state and ask the caller to reset its draft.
    pub fn request_reset(&mut self) {
        self.reset_requested = true;
    }

    pub fn reset_button(&mut self, ui: &mut Ui, label: impl Into<egui::WidgetText>) -> Response {
        let response = ui.button(label);
        if response.clicked() {
            self.request_reset();
        }
        response
    }
}

#[derive(Debug, Default, Clone)]
pub struct Number<T> {
    raw: String,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Number<T>
where
    T: FromStr + Display,
{
    pub fn new(value: T) -> Self {
        Self {
            raw: value.to_string(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn value(&self) -> Result<T, <T as FromStr>::Err> {
        self.raw.parse::<T>()
    }

    pub fn raw_mut(&mut self) -> &mut String {
        &mut self.raw
    }

    pub fn set_raw(&mut self, value: T) {
        self.raw = value.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_draft_survives_frames_resets_and_can_be_removed() {
        let ctx = egui::Context::default();
        for stage in 0..4 {
            let _ = ctx.run_ui(Default::default(), |ui| {
                let result = Form::new("managed").show::<String, _>(ui, |form, _, draft| {
                    match stage {
                        0 => {
                            assert!(draft.is_empty());
                            draft.push_str("Alice");
                        }
                        1 => {
                            assert_eq!(draft, "Alice");
                            form.request_submit();
                        }
                        2 => {
                            assert_eq!(draft, "Alice");
                            form.request_reset();
                        }
                        _ => assert!(draft.is_empty()),
                    }
                    123
                });
                assert_eq!(result.inner, 123);
                assert_eq!(result.data, if stage < 2 { "Alice" } else { "" });
                assert_eq!(result.submitted, stage == 1);
                assert_eq!(result.reset_requested, stage == 2);
                let other = Form::new("other").show::<String, _>(ui, |_, _, draft| {
                    assert!(draft.is_empty());
                });
                assert!(other.data.is_empty());
                if stage == 3 {
                    let id = Form::new("managed").id(ui);
                    Form::new("managed").remove::<String>(ui);
                    assert!(
                        ui.data(|d| d.get_temp::<ManagedDraft<String>>(id))
                            .is_none()
                    );
                    assert!(ui.data(|d| d.get_temp::<FormState>(id)).is_none());
                }
            });
        }
    }

    #[test]
    fn borrowed_data_does_not_need_managed_storage_bounds() {
        // Rc<Cell<_>> is neither Send nor Sync.
        let mut data = std::rc::Rc::new(std::cell::Cell::new(7));
        let ctx = egui::Context::default();
        let _ = ctx.run_ui(Default::default(), |ui| {
            Form::new("borrowed").show_with_data(ui, &mut data, |_, _, value| {
                value.set(8);
            });
        });
        assert_eq!(data.get(), 8);
    }

    #[test]
    fn cached_state_is_scoped_to_form_and_context() {
        let ctx = egui::Context::default();
        let mut draft = String::new();
        let _ = ctx.run_ui(Default::default(), |ui| {
            egui::CentralPanel::default().show_inside(ui, |ui| {
                let a = Form::new("a").id(ui);
                let b = Form::new("b").id(ui);
                let result = Form::new("a").show_with_data(ui, &mut draft, |form, ui, draft| {
                    // Submission before field registration must still validate all fields.
                    form.request_submit();
                    form.field(
                        ui,
                        "name",
                        draft,
                        |ui, value| ui.text_edit_singleline(value),
                        |_| Err("required".into()),
                    );
                });
                assert!(result.submit_requested);
                assert!(!result.submitted);
                assert!(!result.valid);
                assert_eq!(result.errors.len(), 1);
                Form::new("b").show_with_data(ui, &mut draft, |_, _, _| {});
                assert!(ui.data(|d| d.get_temp::<FormState>(a).unwrap().submit_attempted()));
                assert!(!ui.data(|d| d.get_temp::<FormState>(b).unwrap().submit_attempted()));
                assert!(
                    egui::Context::default()
                        .data(|d| d.get_temp::<FormState>(a))
                        .is_none()
                );
                Form::new("a").remove_state(ui);
                assert!(ui.data(|d| d.get_temp::<FormState>(a)).is_none());
            });
        });
    }

    #[test]
    fn explicit_state_reset_wins_over_submit_without_modifying_draft() {
        let ctx = egui::Context::default();
        let mut state = FormState::default();
        let mut draft = 42;
        let _ = ctx.run_ui(Default::default(), |ui| {
            egui::CentralPanel::default().show_inside(ui, |ui| {
                let result = Form::new("explicit").show_with_state(
                    ui,
                    &mut draft,
                    &mut state,
                    |form, _, _| {
                        form.request_submit();
                    },
                );
                assert!(result.submitted);
                assert!(state.submit_attempted());
                let result = Form::new("explicit").show_with_state(
                    ui,
                    &mut draft,
                    &mut state,
                    |form, _, _| {
                        form.request_submit();
                        form.request_reset();
                    },
                );
                assert!(result.reset_requested);
                assert!(!result.submitted);
                assert!(!state.submit_attempted());
                assert_eq!(draft, 42);
                assert!(
                    ui.data(|d| d.get_temp::<FormState>(Form::new("explicit").id(ui)))
                        .is_none()
                );
            });
        });
    }
}
