use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use egui::{Align2, Area, Frame, Id, Order, RichText, Ui, Vec2};
use parking_lot::Mutex;

use crate::theme::Theme;

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
struct SpringState {
    pos: f32,
    vel: f32,
}

struct ToastManager {
    toasts: Vec<Toast>,
    next_id: u64,
    position: ToastPosition,
    /// 每个 Toast 的堆叠 Y 轴 spring 状态，key 为 toast.id
    spring_states: HashMap<u64, SpringState>,
}

static MANAGER: OnceLock<Mutex<ToastManager>> = OnceLock::new();

fn get_manager() -> &'static Mutex<ToastManager> {
    MANAGER.get_or_init(|| {
        Mutex::new(ToastManager {
            toasts: Vec::new(),
            next_id: 0,
            position: ToastPosition::default(),
            spring_states: HashMap::new(),
        })
    })
}

/// 设置所有 Toast 的显示位置
pub fn set_position(position: ToastPosition) {
    let mut manager = get_manager().lock();
    manager.position = position;
}

/// 创建一个 Toast
pub fn create(message: impl Into<String>, kind: ToastKind) {
    let mut manager = get_manager().lock();
    let id = manager.next_id;
    manager.next_id += 1;
    manager.toasts.push(Toast {
        id,
        kind,
        message: message.into(),
        created_at: Instant::now(),
        duration: Duration::from_secs(3),
    });
}

pub fn info(message: impl Into<String>) {
    create(message, ToastKind::Info);
}

pub fn success(message: impl Into<String>) {
    create(message, ToastKind::Success);
}

pub fn warning(message: impl Into<String>) {
    create(message, ToastKind::Warning);
}

pub fn error(message: impl Into<String>) {
    create(message, ToastKind::Error);
}

/// 渲染所有 Toast
pub fn renderer(ui: &mut Ui, theme: &Theme) {
    let mut manager = get_manager().lock();
    let now = Instant::now();

    // 1. 清理完全结束（包括退出动画）的 Toast，同时移除对应的 spring 状态
    manager.toasts.retain(|t| {
        let elapsed = now.duration_since(t.created_at).as_secs_f32();
        elapsed < t.duration.as_secs_f32() + 0.3
    });
    let active_ids: Vec<u64> = manager.toasts.iter().map(|t| t.id).collect();
    manager.spring_states.retain(|id, _| {
        active_ids
            .iter()
            .any(|&tid| tid == *id || (tid ^ 0xDEAD_BEEF) == *id)
    });

    if manager.toasts.is_empty() {
        return;
    }

    let ctx = ui.ctx();

    // 限制 dt 最大值，避免卡帧时 spring 发生大幅跳变
    let dt = ctx.input(|i| i.unstable_dt).min(0.05);

    let pos_config = manager.position;
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

    let toasts = manager.toasts.clone();
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
            let force = -SPRING_K * (slide_spring.pos - slide_target) - SPRING_C * slide_spring.vel;
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
        let stack_spring = manager
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
        let area_id = Id::new("toast_area").with(toast.id);
        let response = Area::new(area_id)
            .anchor(anchor, final_offset)
            .order(Order::Foreground)
            .interactable(true)
            .show(ctx, |ui| {
                ui.set_max_width(320.0);
                render_toast_ui(ui, theme, toast, alpha);
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

fn render_toast_ui(ui: &mut Ui, theme: &Theme, toast: &Toast, alpha: f32) {
    let color = match toast.kind {
        ToastKind::Info => theme.colors.blue.l4,
        ToastKind::Success => theme.colors.green.l4,
        ToastKind::Warning => theme.colors.orange.l4,
        ToastKind::Error => theme.colors.red.l4,
    };

    let bg_color = color.gamma_multiply(alpha);
    let text_color = egui::Color32::WHITE.gamma_multiply(alpha);

    Frame::NONE
        .fill(bg_color)
        .corner_radius(egui::CornerRadius::same(theme.corner_radius.sm as u8))
        .inner_margin(egui::Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.label(
                RichText::new(&toast.message)
                    .color(text_color)
                    .size(theme.text_size.sm),
            );
        });
}
