use crate::theme::Theme;
use egui::{Context, Ui};
use egui_component_style_macros::FrameModifier;

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub struct WindowState {
    pub pos: egui::Pos2,
    pub is_collapsed: bool,
    pub is_open: bool,
    pub min_size: egui::Vec2,
    pub header_title_width: f32,
}

impl WindowState {
    pub fn load(ctx: &Context, id: egui::Id) -> Option<Self> {
        ctx.data_mut(|d| d.get_persisted(id))
    }

    pub fn store(&self, ctx: &Context, id: egui::Id) {
        ctx.data_mut(|d| d.insert_persisted(id, *self));
    }
}

#[derive(FrameModifier, Clone)]
pub struct Style {
    pub frame: egui::Frame,
    pub init_pos: egui::Pos2,
    /// resize 允许的最小宽度
    /// 如果没有那么是自动计算的 header 的宽度
    pub min_width: Option<f32>,
    /// resize 允许的最小高度
    /// 如果没有那么是自动计算的 header + content 的高度
    pub min_height: Option<f32>,
    /// 如果有，固定宽度为这个值，并设置横向不可 resize
    pub width: Option<f32>,
    /// 如果有，固定高度为这个值，并设置纵向不可 resize
    pub height: Option<f32>,
    /// Header bar height
    pub header_height: f32,
}

impl Style {
    pub fn new(theme: &Theme) -> Self {
        let card_frame = super::utils::CardFrame::new(theme, crate::theme::CardSize::Md);
        Self {
            frame: card_frame.frame,
            init_pos: egui::pos2(150.0, 150.0),
            min_width: None,
            min_height: None,
            width: None,
            height: None,
            header_height: 36.0,
        }
    }

    pub fn init_pos(mut self, init_pos: egui::Pos2) -> Self {
        self.init_pos = init_pos;
        self
    }

    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = Some(min_width);
        self
    }

    pub fn min_height(mut self, min_height: f32) -> Self {
        self.min_height = Some(min_height);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn header_height(mut self, header_height: f32) -> Self {
        self.header_height = header_height;
        self
    }
}

pub fn render(
    ctx: &Context,
    id: egui::Id,
    mut open: Option<&mut bool>,
    header: impl FnOnce(&mut Ui, &mut WindowState),
    content: impl FnOnce(&mut Ui, &mut WindowState),
    style: Style,
) {
    // 1. Load or initialize window state
    let mut state = WindowState::load(ctx, id).unwrap_or_else(|| {
        let is_open_init = open.as_ref().map_or(true, |o| **o);
        WindowState {
            pos: style.init_pos,
            is_collapsed: false,
            is_open: is_open_init,
            min_size: egui::vec2(
                style.min_width.unwrap_or(240.0),
                style.min_height.unwrap_or(150.0),
            ),
            header_title_width: 150.0,
        }
    });

    // Sync is_open: if the caller explicitly set open=false, respect it
    if let Some(ref o) = open {
        state.is_open = **o;
    }

    // If the window is closed, do not render the egui::Window at all.
    // Since we don't use egui's built-in open/close (title_bar is false),
    // there is no fade-out animation to worry about.
    if !state.is_open {
        state.store(ctx, id);
        return;
    }

    // 2. Prepare Window Builder
    // NOTE: We intentionally do NOT call window.open() here.
    // egui's Window wraps content in a CollapsingState that is marked .closable(),
    // which means ui.close() from within the content would close the Collapsible
    // (not the Window), leaving a residual empty frame on screen.
    // Instead, we manage close state entirely through WindowState.is_open,
    // which the header closure can set to false directly.
    let mut window = egui::Window::new("")
        .id(id)
        .title_bar(false)
        .movable(false)
        .fade_out(false)
        .current_pos(state.pos)
        .frame(style.frame);

    // 3. Apply constraints
    let (min_w, max_w) = match (style.width, style.min_width) {
        (Some(w), _) => (w, w),
        (None, Some(min_w)) => (min_w, f32::INFINITY),
        (None, None) => (state.min_size.x, f32::INFINITY),
    };

    let (min_h, max_h) = if state.is_collapsed {
        (style.header_height, style.header_height)
    } else {
        match (style.height, style.min_height) {
            (Some(h), _) => (h, h),
            (None, Some(min_h)) => (min_h, f32::INFINITY),
            (None, None) => (state.min_size.y, f32::INFINITY),
        }
    };

    let resizable_x = style.width.is_none();
    let resizable_y = style.height.is_none() && !state.is_collapsed;

    if state.is_collapsed {
        window = window
            .resizable([resizable_x, false])
            .min_size([min_w, min_h])
            .max_size([max_w, max_h]);
    } else {
        window = window
            .resizable([resizable_x, resizable_y])
            .default_size(state.min_size)
            .min_size([min_w, min_h])
            .max_size([max_w, max_h]);
    }

    window.show(ctx, |ui| {
        // 1. Allocate the header area first
        let (header_rect, _) = ui.allocate_at_least(
            egui::vec2(ui.available_width(), style.header_height),
            egui::Sense::hover(),
        );

        // 2. Register the drag detector FIRST
        let drag_response = ui.interact(header_rect, id.with("header_drag"), egui::Sense::drag());

        if drag_response.dragged() {
            state.pos += drag_response.drag_delta();
        }

        // 3. Draw the background and buttons ON TOP of the drag detector
        let mut header_ui = ui.new_child(egui::UiBuilder::new().max_rect(header_rect));

        header(&mut header_ui, &mut state);

        let mut next_min_w = state.header_title_width + 80.0;
        let mut next_min_h = style.header_height;

        // Render Window Body only if not collapsed
        if !state.is_collapsed {
            ui.separator();
            next_min_h += 8.0; // height of separator and spacing

            let scroll_output = egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    content(ui, &mut state);
                });

            // Height should fit the header + separator + body content
            next_min_h += scroll_output.content_size.y;
        }

        // Apply style constraints on computed minimum size
        if let Some(min_w_limit) = style.min_width {
            next_min_w = next_min_w.max(min_w_limit);
        }
        if let Some(min_h_limit) = style.min_height {
            next_min_h = next_min_h.max(min_h_limit);
        }

        // Save the calculated inner minimum size for the next frame
        state.min_size = egui::vec2(next_min_w, next_min_h);
    });

    // Sync state.is_open back to caller's open boolean.
    // The header closure may have set state.is_open = false.
    if let Some(ref mut o) = open {
        **o = state.is_open;
    }

    state.store(ctx, id);
}
