//! Context-local toast notifications stored in egui temporary data.
//!
//! Pass the same Context to mutation functions and render with [`renderer`]
//! each frame. Background tasks may use a cloned Context handle.

use std::collections::HashMap;
use std::sync::Arc;
use web_time::{Duration, Instant};

use egui::{Align2, Area, Context, Frame, Id, Order, Ui, Vec2};

use crate::{
    components::{icon, text},
    icon::ICONS,
    theme::Theme,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    #[default]
    BottomRight,
    TopCenter,
    BottomCenter,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub id: u64,
    pub kind: ToastKind,
    pub message: String,
    pub created_at: Instant,
    pub duration: Duration,
}

/// 单个 Toast 堆叠位移的 Spring 状态
#[derive(Clone)]
struct SpringState {
    pos: f32,
    vel: f32,
}

type Content = Arc<dyn Fn(&mut Ui, &Theme) + Send + Sync>;

#[derive(Clone, Default)]
struct ToastManager {
    contents: HashMap<u64, Content>,
    toasts: Vec<Toast>,
    next_id: u64,
    position: ToastPosition,
    /// 每个 Toast 的堆叠 Y 轴 spring 状态，key 为 toast.id
    spring_states: HashMap<u64, SpringState>,
}

// All toast state belongs to this Context and is never serialized.
fn with_manager<R>(ctx: &Context, f: impl FnOnce(&mut ToastManager) -> R) -> R {
    ctx.data_mut(|data| {
        f(data.get_temp_mut_or_default::<ToastManager>(Id::new("egui_component/toast")))
    })
}

/// Set the toast position for this context.
pub fn set_position(ctx: &Context, position: ToastPosition) {
    with_manager(ctx, |manager| manager.position = position);
    ctx.request_repaint();
}

/// Create a toast in this context.
pub fn create(ctx: &Context, message: impl Into<String>, kind: ToastKind) {
    let message = message.into();
    with_manager(ctx, |manager| {
        let id = manager.next_id;
        manager.next_id += 1;
        manager.toasts.push(Toast {
            id,
            kind,
            message,
            created_at: Instant::now(),
            duration: Duration::from_secs(3),
        });
    });
    ctx.request_repaint();
}

/// Create a toast with arbitrary UI. `None` keeps it visible until [`dismiss`].
/// The returned ID is local to this context.
/// The callback runs without the context data lock and may create or dismiss toasts.
/// Capture shared state to update content between frames.
pub fn custom(
    ctx: &Context,
    kind: ToastKind,
    duration: Option<Duration>,
    content: impl Fn(&mut Ui, &Theme) + Send + Sync + 'static,
) -> u64 {
    let id = with_manager(ctx, |manager| {
        let id = manager.next_id;
        manager.next_id += 1;
        manager.toasts.push(Toast {
            id,
            kind,
            message: String::new(),
            created_at: Instant::now(),
            duration: duration.unwrap_or(Duration::MAX),
        });
        manager.contents.insert(id, Arc::new(content));
        id
    });
    ctx.request_repaint();
    id
}

/// Dismiss a toast in this context, allowing its exit animation to complete.
pub fn dismiss(ctx: &Context, id: u64) {
    with_manager(ctx, |manager| {
        if let Some(toast) = manager.toasts.iter_mut().find(|toast| toast.id == id) {
            toast.duration = Instant::now().duration_since(toast.created_at);
        }
    });
    ctx.request_repaint();
}

pub fn info(ctx: &Context, message: impl Into<String>) {
    create(ctx, message, ToastKind::Info);
}

pub fn success(ctx: &Context, message: impl Into<String>) {
    create(ctx, message, ToastKind::Success);
}

pub fn warning(ctx: &Context, message: impl Into<String>) {
    create(ctx, message, ToastKind::Warning);
}

pub fn error(ctx: &Context, message: impl Into<String>) {
    create(ctx, message, ToastKind::Error);
}

/// 渲染所有 Toast
pub fn renderer(ui: &mut Ui, theme: &Theme) {
    let ctx = ui.ctx();
    let now = Instant::now();
    let (toasts, pos_config) = with_manager(ctx, |manager| {
        // 1. 清理完全结束（包括退出动画）的 Toast，同时移除对应的 spring 状态
        manager.toasts.retain(|t| {
            let elapsed = now.duration_since(t.created_at).as_secs_f32();
            elapsed < t.duration.as_secs_f32() + 0.3
        });
        let active_ids: Vec<u64> = manager.toasts.iter().map(|t| t.id).collect();
        manager.contents.retain(|id, _| active_ids.contains(id));
        manager.spring_states.retain(|id, _| {
            active_ids
                .iter()
                .any(|&tid| tid == *id || (tid ^ 0xDEAD_BEEF) == *id)
        });

        (manager.toasts.clone(), manager.position)
    });

    if toasts.is_empty() {
        return;
    }

    // 限制 dt 最大值，避免卡帧时 spring 发生大幅跳变
    let dt = ctx.input(|i| i.unstable_dt).min(0.05);

    let is_top = matches!(
        pos_config,
        ToastPosition::TopLeft | ToastPosition::TopRight | ToastPosition::TopCenter
    );

    let (anchor, base_offset) = match pos_config {
        ToastPosition::TopLeft => (Align2::LEFT_TOP, Vec2::new(20.0, 20.0)),
        ToastPosition::TopRight => (Align2::RIGHT_TOP, Vec2::new(-20.0, 20.0)),
        ToastPosition::BottomLeft => (Align2::LEFT_BOTTOM, Vec2::new(20.0, -20.0)),
        ToastPosition::BottomRight => (Align2::RIGHT_BOTTOM, Vec2::new(-20.0, -20.0)),
        ToastPosition::TopCenter => (Align2::CENTER_TOP, Vec2::new(0.0, 20.0)),
        ToastPosition::BottomCenter => (Align2::CENTER_BOTTOM, Vec2::new(0.0, -20.0)),
    };

    let mut current_stack_y = 0.0_f32;
    let spacing = 12.0_f32;

    // Spring 参数：
    //   k = 200  —— 刚度，值越大弹得越快
    //   c = 22   —— 阻尼，临界阻尼 ≈ 2*sqrt(200) ≈ 28.3
    //              低于临界值时有回弹感；22 约为轻微欠阻尼，有一次明显弹跳
    //   调节建议：
    //     想要更弹  → 减小 c（如 16）
    //     想要无弹跳 → c 接近 28~30
    const SPRING_K: f32 = 200.0;
    const SPRING_C: f32 = 22.0;

    for toast in &toasts {
        let elapsed = now.duration_since(toast.created_at).as_secs_f32();
        let total_dur = toast.duration.as_secs_f32();
        let is_exiting = elapsed > total_dur;

        // ── 透明度动画（保持原有线性，够用） ──────────────────────────────
        let alpha = if elapsed < 0.3 {
            elapsed / 0.3
        } else if is_exiting {
            ((total_dur + 0.3) - elapsed).max(0.0) / 0.3
        } else {
            1.0
        };

        // ── 入场 Slide 动画（Spring 驱动） ────────────────────────────────
        // 用独立的 spring 状态模拟入场位移，初始从屏幕外推入
        // 目标始终是 0（完全就位），只在首帧初始化为偏移量
        let (final_offset, content) = with_manager(ctx, |manager| {
            let slide_spring = manager
                .spring_states
                .entry(toast.id ^ 0xDEAD_BEEF) // 与堆叠 spring 区分 key
                .or_insert_with(|| SpringState {
                    pos: if is_top { -50.0 } else { 50.0 }, // 初始在屏幕外
                    vel: 0.0,
                });

            // 入场结束后（elapsed > 0.6）不再需要继续积分，直接钳制为 0
            let slide_y = if elapsed < 0.6 {
                let slide_target = 0.0_f32;
                let force =
                    -SPRING_K * (slide_spring.pos - slide_target) - SPRING_C * slide_spring.vel;
                slide_spring.vel += force * dt;
                slide_spring.pos += slide_spring.vel * dt;
                slide_spring.pos
            } else {
                slide_spring.pos = 0.0;
                slide_spring.vel = 0.0;
                0.0
            };

            // ── 堆叠位移 Spring ───────────────────────────────────────────────
            // 目标 = 当前累计高度；spring 平滑地追踪它，新 Toast 插入时自然推开已有的
            let stack_target = current_stack_y;
            let stack_spring =
                manager
                    .spring_states
                    .entry(toast.id)
                    .or_insert_with(|| SpringState {
                        pos: stack_target, // 首帧直接落在目标位置，无需初始弹跳
                        vel: 0.0,
                    });

            let force = -SPRING_K * (stack_spring.pos - stack_target) - SPRING_C * stack_spring.vel;
            stack_spring.vel += force * dt;
            stack_spring.pos += stack_spring.vel * dt;

            let smooth_stack_y = stack_spring.pos;

            // ── 合并偏移量 ─────────────────────────────────────────────────────
            let final_y_offset = if is_top {
                base_offset.y + smooth_stack_y + slide_y
            } else {
                base_offset.y - smooth_stack_y + slide_y
            };
            let final_offset = Vec2::new(base_offset.x, final_y_offset);

            // ── 渲染 ──────────────────────────────────────────────────────────
            (final_offset, manager.contents.get(&toast.id).cloned())
        });
        let area_id = Id::new("toast_area").with(toast.id);
        let response = Area::new(area_id)
            .anchor(anchor, final_offset)
            .order(Order::Foreground)
            .interactable(true)
            .show(ctx, |ui| {
                ui.set_max_width(320.0);
                render_toast_ui(ui, theme, toast, alpha, content.as_ref());
            });

        // ── 堆叠高度累计 ──────────────────────────────────────────────────
        // 退出时通过 alpha 缩减占位高度，使下方 Toast 平滑上移
        let full_height = response.response.rect.height();
        if !is_exiting {
            current_stack_y += full_height + spacing;
        } else {
            current_stack_y += (full_height + spacing) * alpha;
        }
    }

    // 强制重绘以维持动画
    ctx.request_repaint();
}

fn render_toast_ui(
    ui: &mut Ui,
    theme: &Theme,
    toast: &Toast,
    alpha: f32,
    content: Option<&Content>,
) {
    let status_icon = match toast.kind {
        ToastKind::Info => None,
        ToastKind::Success => Some((ICONS.success.clone(), theme.colors.green.l4)),
        ToastKind::Warning => Some((ICONS.warning.clone(), theme.colors.orange.l4)),
        ToastKind::Error => Some((ICONS.error.clone(), theme.colors.red.l4)),
    };

    ui.scope(|ui| {
        ui.multiply_opacity(alpha);
        ui.visuals_mut().override_text_color = Some(theme.colors.fg.base);
        let frame = Frame::NONE
            .fill(theme.colors.bg.base)
            .stroke(egui::Stroke::new(1.0_f32, theme.colors.gray.border()))
            .corner_radius(egui::CornerRadius::same(theme.corner_radius.sm as u8))
            .inner_margin(egui::Margin::symmetric(16, 12));
        // Include padding and border in the shared outer width.
        let content_width = 320.0 - frame.total_margin().sum().x;
        frame.show(ui, |ui| {
            ui.set_width(content_width);
            let render_content = |ui: &mut Ui| {
                if let Some(content) = content {
                    content(ui, theme);
                } else {
                    text::render(
                        ui,
                        &toast.message,
                        text::Style::new(theme).size(theme.text_size.sm),
                    );
                }
            };
            if let Some((source, color)) = status_icon {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    icon::render(
                        ui,
                        source,
                        icon::Style::new(theme)
                            .box_size(20.0)
                            .icon_size(20.0)
                            .color(color)
                            .sense(egui::Sense::hover()),
                    );
                    ui.vertical(render_content);
                });
            } else {
                render_content(ui);
            }
        });
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn contexts_isolate_queue_position_and_dismissal() {
        let a = Context::default();
        let b = Context::default();
        set_position(&a, ToastPosition::TopLeft);
        let first = custom(&a, ToastKind::Info, None, |_, _| {});
        let second = custom(&b, ToastKind::Success, None, |_, _| {});
        assert_eq!(first, second); // IDs are local to each Context.
        dismiss(&a, first);
        with_manager(&a, |manager| {
            assert_eq!(manager.position, ToastPosition::TopLeft);
            assert!(manager.toasts[0].duration < Duration::from_secs(10));
        });
        with_manager(&b, |manager| {
            assert_eq!(manager.position, ToastPosition::BottomRight);
            assert_eq!(manager.toasts[0].duration, Duration::MAX);
            assert!(manager.spring_states.is_empty());
        });
        let cloned = a.clone();
        info(&cloned, "Shared context handle");
        with_manager(&a, |manager| assert_eq!(manager.toasts.len(), 2));
    }

    #[test]
    fn custom_content_can_modify_toasts_without_losing_updates() {
        let ctx = Context::default();
        let calls = Arc::new(AtomicUsize::new(0));
        let rendered = calls.clone();
        let id = custom(&ctx, ToastKind::Info, None, move |ui, _| {
            // This callback must run outside the context data lock.
            if rendered.fetch_add(1, Ordering::Relaxed) == 0 {
                set_position(ui.ctx(), ToastPosition::TopLeft);
                success(ui.ctx(), "Created inside callback");
                dismiss(ui.ctx(), 0);
            }
            ui.label("Custom content");
        });
        with_manager(&ctx, |manager| {
            assert_eq!(manager.toasts[0].duration, Duration::MAX);
        });
        let theme =
            crate::theme::UiTheme::from_json_bytes(include_bytes!("../examples/theme.json"))
                .unwrap();
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
            renderer(ui, &theme.light);
        });
        assert!(calls.load(Ordering::Relaxed) > 0);
        with_manager(&ctx, |manager| {
            assert_eq!(manager.position, ToastPosition::TopLeft);
            assert_eq!(manager.toasts.len(), 2);
            assert!(
                manager.toasts.iter().find(|t| t.id == id).unwrap().duration
                    < Duration::from_secs(10)
            );
            assert!(
                manager
                    .toasts
                    .iter()
                    .any(|t| t.message == "Created inside callback")
            );
            assert!(!manager.spring_states.is_empty());
        });
        // Expired entries release custom content and animation state together.
        with_manager(&ctx, |manager| {
            for toast in &mut manager.toasts {
                toast.created_at = Instant::now() - Duration::from_secs(10);
                toast.duration = Duration::ZERO;
            }
        });
        let _ = ctx.run_ui(egui::RawInput::default(), |ui| renderer(ui, &theme.light));
        with_manager(&ctx, |manager| {
            assert!(manager.toasts.is_empty());
            assert!(manager.contents.is_empty());
            assert!(manager.spring_states.is_empty());
        });
    }
}
