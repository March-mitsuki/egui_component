//! Focus management for modal surfaces (dialog, drawer), per the WAI-ARIA APG
//! dialog pattern: focus moves into the modal when it opens, Tab and Shift+Tab
//! wrap inside it, and focus returns to the opener when it closes.
//!
//! egui's `Modal` already keeps focus off the background, but it leaves focus
//! empty on open, lets Tab fall off the last control, and forgets the opener.
use std::collections::HashMap;

use egui::{Context, FocusDirection, Id};

#[derive(Clone, Copy)]
struct Record {
    /// Widget focused when the modal opened, restored on close.
    opener: Option<Id>,
    /// Still waiting for focus to land inside the modal.
    opening: bool,
    /// Tab pressed on the last pass: `Some(shift)`.
    tab: Option<bool>,
    last_pass: u64,
}

type Records = HashMap<Id, Record>;

fn key() -> Id {
    Id::unique("egui_component_modal_focus")
}

/// Call every pass while the modal is open, before its content.
pub(super) fn begin(ctx: &Context, id: Id) {
    ctx.add_plugin(RestorePlugin);
    let focused = ctx.memory(|m| m.focused());
    let mut records = ctx.data(|d| d.get_temp::<Records>(key()).unwrap_or_default());
    let record = records.entry(id).or_insert(Record {
        opener: focused,
        opening: true,
        tab: None,
        last_pass: 0,
    });
    if focused.is_some() && focused != record.opener {
        record.opening = false;
    }
    // With nothing focused, egui's Next/Previous pick the first/last control
    // the modal allows, which also wraps Tab at either end.
    if focused.is_none() && (record.opening || record.tab.is_some()) {
        let direction = if record.tab == Some(true) {
            FocusDirection::Previous
        } else {
            FocusDirection::Next
        };
        ctx.memory_mut(|m| m.move_focus(direction));
        ctx.request_repaint();
    }
    ctx.data_mut(|d| d.insert_temp(key(), records));
}

/// Call every pass while the modal is open, after its content.
pub(super) fn end(ctx: &Context, id: Id) {
    let pass = ctx.cumulative_pass_nr();
    let tab = ctx.input(|i| i.key_pressed(egui::Key::Tab).then_some(i.modifiers.shift));
    ctx.data_mut(|d| {
        let records = d.get_temp_mut_or_default::<Records>(key());
        if let Some(record) = records.get_mut(&id) {
            record.tab = tab;
            record.last_pass = pass;
        }
    });
}

/// Restore focus to the opener of every modal not shown during this pass.
struct RestorePlugin;

impl egui::Plugin for RestorePlugin {
    fn debug_name(&self) -> &'static str {
        "egui_component::modal_focus"
    }

    fn on_end_pass(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let pass = ctx.cumulative_pass_nr();
        let closed: Vec<Option<Id>> = ctx.data_mut(|d| {
            let records = d.get_temp_mut_or_default::<Records>(key());
            let closed = records
                .values()
                .filter(|r| r.last_pass < pass)
                .map(|r| r.opener)
                .collect();
            records.retain(|_, r| r.last_pass >= pass);
            closed
        });
        for opener in closed.into_iter().flatten() {
            ctx.memory_mut(|m| m.request_focus(opener));
            ctx.request_repaint();
        }
    }
}

fn dialog_key(ctx: &Context) -> Id {
    Id::unique("egui_component_current_dialog").with(ctx.viewport_id())
}

/// Exposes `ui` as a modal dialog (`role="dialog"` with `aria-modal`) and
/// makes it the target of [`label_dialog`] while `add_contents` runs.
pub(super) fn dialog_scope<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let ctx = ui.ctx().clone();
    let node = ui.unique_id();
    ctx.accesskit_node_builder(node, |builder| {
        builder.set_role(egui::accesskit::Role::Dialog);
        builder.set_modal();
    });
    let key = dialog_key(&ctx);
    let previous = ctx.data(|data| data.get_temp::<Id>(key));
    ctx.data_mut(|data| data.insert_temp(key, node));
    let inner = add_contents(ui);
    ctx.data_mut(|data| match previous {
        Some(previous) => {
            data.insert_temp(key, previous);
        }
        None => data.remove::<Id>(key),
    });
    inner
}

/// Names the enclosing dialog after `title`, like `aria-labelledby`.
/// Does nothing outside [`dialog_scope`].
pub(super) fn label_dialog(ctx: &Context, title: Id) {
    let key = dialog_key(ctx);
    if let Some(node) = ctx.data(|data| data.get_temp::<Id>(key)) {
        ctx.accesskit_node_builder(node, |builder| {
            builder.push_labelled_by(title.accesskit_id());
        });
    }
}
