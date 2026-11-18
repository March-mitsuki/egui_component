//! Label-to-control activation, e.g. clicking a field label toggles its switch.
//!
//! The label is rendered before its control, so the control's ID is only known
//! afterwards. The request is stored and consumed by the control on the next pass.
use egui::{Context, Id};

fn key(ctx: &Context) -> Id {
    Id::unique("egui_component_label_activation").with(ctx.viewport_id())
}

/// Ask the control `target` to perform its primary action on the next pass.
pub(super) fn request(ctx: &Context, target: Id) {
    let (key, pass) = (key(ctx), ctx.cumulative_pass_nr());
    // Compute everything before locking: Context methods cannot be called inside.
    ctx.data_mut(|data| data.insert_temp(key, (target, pass)));
    ctx.request_repaint();
}

/// Consume a pending activation for `id`. Stale requests (e.g. the control was
/// hidden on the next pass) are ignored so they cannot fire later.
pub(super) fn take(ctx: &Context, id: Id) -> bool {
    let key = key(ctx);
    let Some((target, pass)) = ctx.data(|data| data.get_temp::<(Id, u64)>(key)) else {
        return false;
    };
    if target != id {
        return false;
    }
    ctx.data_mut(|data| data.remove::<(Id, u64)>(key));
    ctx.cumulative_pass_nr().saturating_sub(pass) <= 2
}
