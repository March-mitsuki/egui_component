//! Animated modal drawers. Call [`render`] every frame, including while closed,
//! so the caller-owned `open` state can drive the exit animation.
use crate::theme::Theme;
use egui::{Context, Id, Ui};
use egui_component_style_macros::FrameModifier;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    Left,
    #[default]
    Right,
}

#[derive(FrameModifier, Clone)]
pub struct Style {
    pub frame: egui::Frame,
    pub side: Side,
    /// Total outer width, including padding.
    pub width: f32,
    pub backdrop_color: egui::Color32,
    /// Duration in seconds. Zero disables animation.
    pub animation_duration: f32,
    pub close_on_escape: bool,
    pub close_on_backdrop: bool,
}
impl Style {
    pub fn new(theme: &Theme) -> Self {
        Self {
            frame: egui::Frame::new()
                .fill(theme.colors.bg.base)
                .inner_margin(24),
            side: Side::Right,
            width: 320.0,
            backdrop_color: egui::Color32::from_black_alpha(100),
            animation_duration: 0.2,
            close_on_escape: true,
            close_on_backdrop: true,
        }
    }
    pub fn side(mut self, value: Side) -> Self {
        self.side = value;
        self
    }
    pub fn width(mut self, value: f32) -> Self {
        self.width = value;
        self
    }
    pub fn backdrop_color(mut self, value: egui::Color32) -> Self {
        self.backdrop_color = value;
        self
    }
    pub fn animation_duration(mut self, value: f32) -> Self {
        self.animation_duration = value;
        self
    }
    pub fn close_on_escape(mut self, value: bool) -> Self {
        self.close_on_escape = value;
        self
    }
    pub fn close_on_backdrop(mut self, value: bool) -> Self {
        self.close_on_backdrop = value;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseReason {
    Escape,
    Backdrop,
    CloseButton,
    /// Content called `ui.close()`, for example a Cancel button.
    Content,
}

pub struct DrawerResponse<R> {
    /// Absent when fully closed. The builder closure is only called while visible.
    pub inner: Option<R>,
    pub response: Option<egui::Response>,
    close_reason: Option<CloseReason>,
}
impl<R> DrawerResponse<R> {
    pub fn should_close(&self) -> bool {
        self.close_reason.is_some()
    }
    pub fn close_reason(&self) -> Option<CloseReason> {
        self.close_reason
    }
}

type Slot<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;
/// Optional slots, rendered in a fixed layout regardless of registration order.
/// Each slot may be registered once; registering again replaces it.
#[derive(Default)]
pub struct Drawer<'a> {
    header: Option<Slot<'a>>,
    body: Option<Slot<'a>>,
    footer: Option<Slot<'a>>,
}
impl<'a> Drawer<'a> {
    pub fn header(&mut self, render: impl FnOnce(&mut Ui) + 'a) {
        self.header = Some(Box::new(render));
    }
    pub fn body(&mut self, render: impl FnOnce(&mut Ui) + 'a) {
        self.body = Some(Box::new(render));
    }
    pub fn footer(&mut self, render: impl FnOnce(&mut Ui) + 'a) {
        self.footer = Some(Box::new(render));
    }
}

/// Standard title and close button. Returns whether the close button was clicked.
/// Call `ui.close()` (as this helper does) to request closing the drawer.
pub fn render_header(ui: &mut Ui, theme: &Theme, title: &str) -> bool {
    let mut clicked = false;
    ui.horizontal(|ui| {
        super::text::render(ui, title, super::text::Style::new_heading(theme));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            clicked = super::icon_button::render(
                ui,
                crate::icon::ICONS.close.clone(),
                super::icon_button::Style::new_ghost_sm(theme),
            )
            .clicked();
            if clicked {
                ui.close();
            }
        });
    });
    if clicked {
        ui.data_mut(|data| data.insert_temp(ui.id().with("drawer-close-button"), true));
    }
    clicked
}

/// `id` must be stable and unique. Continue calling when `open` becomes false.
/// Close requests never change `open`; the caller can accept or veto them.
pub fn render<'a, R>(
    ctx: &Context,
    id: impl std::hash::Hash,
    open: bool,
    renderer: impl FnOnce(&mut Drawer<'a>) -> R,
    style: Style,
) -> DrawerResponse<R> {
    let id = Id::new(id);
    let animation_id = id.with("animation");
    let initialized = ctx.data_mut(|data| {
        let initialized = data.get_temp::<bool>(animation_id).unwrap_or(false);
        data.insert_temp(animation_id, true);
        initialized
    });
    if !initialized {
        ctx.animate_bool_with_time(animation_id, false, 0.0);
    }
    let progress =
        ctx.animate_bool_with_time(animation_id, open, style.animation_duration.max(0.0));
    if !open && progress <= 0.0 {
        return DrawerResponse {
            inner: None,
            response: None,
            close_reason: None,
        };
    }
    let eased = egui::emath::easing::cubic_out(progress);
    let screen = ctx.content_rect();
    let width = style.width.max(0.0).min(screen.width());
    let x = match style.side {
        Side::Right => screen.right() - width * eased,
        Side::Left => screen.left() - width * (1.0 - eased),
    };
    let area = egui::Area::new(id)
        .kind(egui::UiKind::Modal)
        .order(egui::Order::Foreground)
        .sense(egui::Sense::hover())
        .fixed_pos(egui::pos2(x, screen.top()))
        .constrain(false)
        .fade_in(false);
    let mut slots = Drawer::default();
    let inner = renderer(&mut slots);
    let mut header_closed = false;
    let modal = egui::Modal::new(id)
        .area(area)
        .frame(egui::Frame::NONE)
        .backdrop_color(style.backdrop_color.gamma_multiply(eased))
        .show(ctx, |ui| {
            ui.set_clip_rect(screen);
            ui.set_width(width);
            ui.set_height(screen.height());
            let outer = egui::Rect::from_min_size(
                egui::pos2(x, screen.top()),
                egui::vec2(width, screen.height()),
            );
            let rect = outer - style.frame.total_margin();
            ui.painter().add(style.frame.paint(rect));
            let mut content =
                ui.new_child(egui::UiBuilder::new().id_salt("content").max_rect(rect));
            content.set_clip_rect(rect.intersect(screen));
            // Measure footer from the bottom, reserving its actual height for the body.
            let mut bottom = rect.bottom();
            if let Some(footer) = slots.footer {
                let mut footer_ui = content.new_child(
                    egui::UiBuilder::new()
                        .id_salt("footer")
                        .max_rect(rect)
                        .layout(egui::Layout::bottom_up(egui::Align::Min)),
                );
                // Give horizontal action layouts a content-sized row instead of
                // the entire panel height (which would center them vertically).
                let height_id = id.with("footer-height");
                let height = ctx
                    .data_mut(|data| data.get_temp::<f32>(height_id))
                    .unwrap_or(footer_ui.spacing().interact_size.y);
                let row = footer_ui.allocate_ui_with_layout(
                    egui::vec2(rect.width(), height),
                    egui::Layout::top_down(egui::Align::Min),
                    footer,
                );
                let measured = row.response.rect.height();
                if (measured - height).abs() > 0.5 {
                    ctx.data_mut(|data| data.insert_temp(height_id, measured));
                    ctx.request_discard("drawer footer height changed");
                }
                bottom = footer_ui.min_rect().top() - content.spacing().item_spacing.y;
            }
            if let Some(header) = slots.header {
                content.scope_builder(egui::UiBuilder::new().id_salt("header"), |ui| {
                    header(ui);
                    header_closed = ui
                        .data_mut(|data| {
                            data.remove_temp::<bool>(ui.id().with("drawer-close-button"))
                        })
                        .unwrap_or(false);
                });
            }
            if let Some(body) = slots.body {
                let body_rect = egui::Rect::from_min_max(
                    content.cursor().min,
                    egui::pos2(rect.right(), bottom.max(content.cursor().top())),
                );
                content.scope_builder(
                    egui::UiBuilder::new().id_salt("body").max_rect(body_rect),
                    |ui| {
                        ui.set_clip_rect(body_rect.intersect(screen));
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .max_height(body_rect.height())
                            .show(ui, body);
                    },
                );
            }
        });
    let close_reason = if !open || !modal.is_top_modal {
        None
    } else if header_closed {
        Some(CloseReason::CloseButton)
    } else if modal.response.should_close() {
        Some(CloseReason::Content)
    } else if style.close_on_backdrop && modal.backdrop_response.clicked() {
        Some(CloseReason::Backdrop)
    } else if style.close_on_escape
        && !modal.any_popup_open
        && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
    {
        Some(CloseReason::Escape)
    } else {
        None
    };
    DrawerResponse {
        inner: Some(inner),
        response: Some(modal.response),
        close_reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn frame(ctx: &Context, time: f64, mut run: impl FnMut(&Context)) {
        let _ = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 600.0),
                )),
                time: Some(time),
                ..Default::default()
            },
            |ui| run(ui.ctx()),
        );
    }

    #[test]
    fn slots_reserve_footer_and_body_scrolls_inside_panel() {
        let ctx = Context::default();
        let theme =
            crate::theme::UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
                .unwrap()
                .light;
        let header = Cell::new(egui::Rect::NOTHING);
        let body = Cell::new(egui::Rect::NOTHING);
        let footer = Cell::new(egui::Rect::NOTHING);
        for n in 0..3 {
            frame(&ctx, n as f64 / 60.0, |ctx| {
                render(
                    ctx,
                    "layout",
                    true,
                    |drawer| {
                        drawer.header(|ui| {
                            header.set(ui.label("Title").rect);
                        });
                        drawer.body(|ui| {
                            body.set(ui.clip_rect());
                            for _ in 0..100 {
                                ui.label("Long content");
                            }
                        });
                        drawer.footer(|ui| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    footer.set(
                                        super::super::button::render(
                                            ui,
                                            "Save",
                                            super::super::button::Style::new_solid_md(&theme),
                                        )
                                        .rect,
                                    );
                                    ui.button("Cancel").on_hover_text("Cancel");
                                },
                            );
                        });
                    },
                    Style::new(&theme).animation_duration(0.0),
                );
            });
        }
        assert!(header.get().left() >= 480.0);
        assert!(header.get().bottom() <= body.get().top());
        assert!(body.get().bottom() <= footer.get().top());
        assert!(footer.get().bottom() <= 576.0);
        assert!(footer.get().top() > 540.0);
    }

    #[test]
    fn exit_stays_visible_then_stops_and_can_reverse() {
        let ctx = Context::default();
        let theme =
            crate::theme::UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
                .unwrap()
                .light;
        let visible = Cell::new(false);
        let x = Cell::new(0.0);
        for n in 0..20 {
            frame(&ctx, n as f64 / 60.0, |ctx| {
                render(
                    ctx,
                    "animation",
                    true,
                    |drawer| {
                        drawer.body(|ui| {
                            x.set(ui.max_rect().left());
                        });
                    },
                    Style::new(&theme),
                );
            });
        }
        let open_x = x.get();
        frame(&ctx, 20.0 / 60.0, |ctx| {
            let r = render(ctx, "animation", false, |_| (), Style::new(&theme));
            visible.set(r.response.is_some());
        });
        assert!(visible.get(), "exit should still be drawn");
        frame(&ctx, 21.0 / 60.0, |ctx| {
            render(
                ctx,
                "animation",
                true,
                |drawer| {
                    drawer.body(|ui| {
                        x.set(ui.max_rect().left());
                    });
                },
                Style::new(&theme),
            );
        });
        assert!((x.get() - open_x).abs() < 2.0);
        for n in 22..50 {
            frame(&ctx, n as f64 / 60.0, |ctx| {
                let r = render(ctx, "animation", false, |_| (), Style::new(&theme));
                visible.set(r.response.is_some());
            });
        }
        assert!(!visible.get());
    }

    #[test]
    fn content_can_request_close_without_mutating_open_state() {
        let ctx = Context::default();
        let theme =
            crate::theme::UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
                .unwrap()
                .light;
        let reason = Cell::new(None);
        for n in 0..3 {
            frame(&ctx, n as f64 / 60.0, |ctx| {
                let r = render(
                    ctx,
                    "close",
                    true,
                    |drawer| {
                        drawer.footer(|ui| {
                            ui.close();
                        });
                    },
                    Style::new(&theme).animation_duration(0.0),
                );
                reason.set(r.close_reason());
                assert!(r.response.is_some());
            });
        }
        assert_eq!(reason.get(), Some(CloseReason::Content));
    }
    #[test]
    fn escape_and_backdrop_respect_close_options() {
        let theme =
            crate::theme::UiTheme::from_json_bytes(include_bytes!("../../examples/theme.json"))
                .unwrap()
                .light;
        for enabled in [false, true] {
            let ctx = Context::default();
            let reason = Cell::new(None);
            for n in 0..5 {
                let events = match n {
                    2 => vec![egui::Event::Key {
                        key: egui::Key::Escape,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    }],
                    3 | 4 => vec![
                        egui::Event::PointerMoved(egui::pos2(100.0, 100.0)),
                        egui::Event::PointerButton {
                            pos: egui::pos2(100.0, 100.0),
                            button: egui::PointerButton::Primary,
                            pressed: n == 3,
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                    _ => vec![],
                };
                let _ = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(800.0, 600.0),
                        )),
                        time: Some(n as f64 / 60.0),
                        events,
                        ..Default::default()
                    },
                    |ui| {
                        let r = render(
                            ui.ctx(),
                            "input",
                            true,
                            |_| (),
                            Style::new(&theme)
                                .animation_duration(0.0)
                                .close_on_escape(enabled)
                                .close_on_backdrop(enabled),
                        );
                        reason.set(r.close_reason());
                    },
                );
                if n == 2 {
                    assert_eq!(reason.get(), enabled.then_some(CloseReason::Escape));
                }
            }
            assert_eq!(reason.get(), enabled.then_some(CloseReason::Backdrop));
        }
    }
}
