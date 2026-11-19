//! Keyboard navigation for popup menus and select lists (WAI-ARIA APG menu
//! button and select-only combobox patterns), on top of `egui::Popup`:
//!
//! - ArrowDown/ArrowUp on a focused trigger open the popup; opening it with the
//!   keyboard (Enter, Space or an arrow) focuses the preferred item (the
//!   selected option) or the first one.
//! - Inside, ArrowDown/ArrowUp move between items (wrapping), Home/End jump to
//!   the ends, and Tab closes the popup back to the trigger.
//! - Activating an item closes the popup; whenever it closes while focus was in
//!   it, focus returns to the trigger.
//!
//! Items are this crate's buttons and select items rendered inside the popup;
//! they register themselves via [`register`]. Other widgets remain reachable
//! with Tab but are skipped by the arrow keys.
use egui::{Context, Id, Key, Modifiers, Response, Ui};

#[derive(Clone, Default)]
struct State {
    keyboard_open: bool,
    was_open: bool,
    items: Vec<Id>,
}

#[derive(Clone, Default)]
struct Scope {
    items: Vec<Id>,
    preferred: Option<Id>,
    activated: bool,
}

fn state_id(popup_id: Id) -> Id {
    popup_id.with("egui_component_menu_nav")
}

fn scope_key() -> Id {
    Id::unique("egui_component_menu_nav_scope")
}

fn load(ctx: &Context, popup_id: Id) -> State {
    ctx.data(|d| d.get_temp::<State>(state_id(popup_id)).unwrap_or_default())
}

fn save(ctx: &Context, popup_id: Id, state: State) {
    ctx.data_mut(|d| d.insert_temp(state_id(popup_id), state));
}

/// Call after rendering the trigger and before showing the popup.
pub(super) fn trigger(ui: &Ui, trigger: &Response, popup_id: Id) {
    let ctx = ui.ctx();
    if !trigger.has_focus() {
        return;
    }
    let open = egui::Popup::is_id_open(ctx, popup_id);
    let keyboard_click = trigger.clicked() && !trigger.clicked_by(egui::PointerButton::Primary);
    let arrow = !open
        && ui.input_mut(|i| {
            i.consume_key(Modifiers::NONE, Key::ArrowDown)
                | i.consume_key(Modifiers::NONE, Key::ArrowUp)
        });
    if arrow {
        egui::Popup::open_id(ctx, popup_id);
    }
    if arrow || (keyboard_click && !open) {
        let mut state = load(ctx, popup_id);
        state.keyboard_open = true;
        save(ctx, popup_id, state);
        ctx.request_repaint();
    }
}

/// Wrap the popup's content.
pub(super) fn content<R>(
    ui: &mut Ui,
    trigger: &Response,
    popup_id: Id,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let ctx = ui.ctx().clone();
    let outer = ctx.data_mut(|d| d.remove_temp::<Scope>(scope_key()));
    ctx.data_mut(|d| d.insert_temp(scope_key(), Scope::default()));
    let inner = add_contents(ui);
    let scope = ctx
        .data_mut(|d| d.remove_temp::<Scope>(scope_key()))
        .unwrap_or_default();
    if let Some(outer) = outer {
        ctx.data_mut(|d| d.insert_temp(scope_key(), outer));
    }

    let mut state = load(&ctx, popup_id);
    let items = &scope.items;
    let focused = ctx.memory(|m| m.focused());
    let index = focused.and_then(|f| items.iter().position(|id| *id == f));
    let mut target = None;
    if state.keyboard_open && !items.is_empty() {
        state.keyboard_open = false;
        target = Some(scope.preferred.unwrap_or(items[0]));
    } else if let Some(index) = index {
        let last = items.len() - 1;
        ui.input_mut(|i| {
            if i.consume_key(Modifiers::NONE, Key::ArrowDown) {
                target = Some(items[if index == last { 0 } else { index + 1 }]);
            } else if i.consume_key(Modifiers::NONE, Key::ArrowUp) {
                target = Some(items[if index == 0 { last } else { index - 1 }]);
            } else if i.consume_key(Modifiers::NONE, Key::Home) {
                target = Some(items[0]);
            } else if i.consume_key(Modifiers::NONE, Key::End) {
                target = Some(items[last]);
            }
        });
        let tab = ui.input(|i| i.key_pressed(Key::Tab));
        if tab {
            // Cancel egui's own Tab move, which would land on a vanishing item.
            ctx.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
            egui::Popup::close_id(&ctx, popup_id);
            trigger.request_focus();
        }
    }
    if let Some(target) = target {
        // egui also moves focus with the arrow keys; cancel that move.
        ctx.memory_mut(|m| {
            m.move_focus(egui::FocusDirection::None);
            m.request_focus(target);
        });
    }
    if scope.activated {
        egui::Popup::close_id(&ctx, popup_id);
    }
    state.items = scope.items;
    save(&ctx, popup_id, state);
    inner
}

/// Call after showing the popup: returns focus to the trigger once it closes.
pub(super) fn after(ctx: &Context, trigger: &Response, popup_id: Id) {
    let mut state = load(ctx, popup_id);
    let open = egui::Popup::is_id_open(ctx, popup_id);
    if state.was_open && !open {
        let focused = ctx.memory(|m| m.focused());
        if focused.is_none_or(|f| state.items.contains(&f)) {
            trigger.request_focus();
        }
        state.items.clear();
        state.keyboard_open = false;
    }
    state.was_open = open;
    save(ctx, popup_id, state);
}

/// Register a focusable item rendered inside the current popup, if any.
/// `preferred` marks the item to focus when the popup opens (e.g. the
/// selected option).
pub(super) fn register(response: &Response, preferred: bool) {
    if !response.sense.is_focusable() || !response.enabled() {
        return;
    }
    let clicked = response.clicked();
    let id = response.id;
    response.ctx.data_mut(|d| {
        if let Some(mut scope) = d.get_temp::<Scope>(scope_key()) {
            scope.items.push(id);
            if preferred {
                scope.preferred = Some(id);
            }
            scope.activated |= clicked;
            d.insert_temp(scope_key(), scope);
        }
    });
}
