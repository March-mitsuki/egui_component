//! Native and Web galleries start with default fonts and download CJK in the background.
use eframe::egui;
use egui_component::{components::progress, consts::FONT_MANAGER, toast};
use std::{
    ops::ControlFlow,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

const FONT_URL: &str = "https://raw.githubusercontent.com/March-mitsuki/egui_component/main/examples/assets/NotoSansCJK-Regular.ttc";
const MAX_BYTES: usize = 50 * 1024 * 1024;

#[derive(Default)]
struct Download {
    bytes: Vec<u8>,
    total: Option<usize>,
    error: Option<String>,
    done: bool,
}

pub fn start(ctx: &egui::Context) {
    let state = Arc::new(Mutex::new(Download::default()));
    let content_state = state.clone();
    let id = Arc::new(AtomicU64::new(u64::MAX));
    let content_id = id.clone();
    let toast_id = toast::custom(toast::ToastKind::Info, None, move |ui, theme| {
        let (received, total, error, done) = {
            let state = content_state.lock().unwrap();
            (
                state.bytes.len(),
                state.total,
                state.error.clone(),
                state.done,
            )
        };
        ui.set_width(260.0_f32.min(ui.available_width()));
        // English remains readable before the multilingual font is installed.
        if done {
            ui.label("Multilingual font ready");
        } else if let Some(error) = error {
            ui.label("Multilingual font download failed");
            ui.label(error);
            if ui.button("Retry").clicked() {
                toast::dismiss(content_id.load(Ordering::Relaxed));
                start(ui.ctx());
            }
        } else {
            ui.label("Downloading multilingual font…");
            let value = total
                .filter(|total| *total > 0)
                .map(|total| received as f32 / total as f32);
            let mut style = progress::Style::new(theme).width(260.0);
            style.fill_color = egui::Color32::WHITE;
            style.track_color = egui::Color32::from_white_alpha(60);
            progress::render(ui, value, style);
            ui.label(match total {
                Some(total) => format!(
                    "{:.1} / {:.1} MiB",
                    received as f32 / 1048576.0,
                    total as f32 / 1048576.0
                ),
                None => format!("{:.1} MiB downloaded", received as f32 / 1048576.0),
            });
        }
    });
    id.store(toast_id, Ordering::Relaxed);
    let ctx = ctx.clone();
    ehttp::streaming::fetch(ehttp::Request::get(FONT_URL), move |part| {
        let mut state = state.lock().unwrap();
        let outcome = match part {
            Err(error) => Err(error),
            Ok(ehttp::streaming::Part::Response(response)) => {
                if !response.ok {
                    Err(format!("HTTP {}", response.status))
                } else {
                    state.total = response
                        .headers
                        .get("content-length")
                        .and_then(|value| value.parse().ok());
                    if state.total.is_some_and(|size| size > MAX_BYTES) {
                        Err("Font exceeds 50 MiB".into())
                    } else {
                        Ok(false)
                    }
                }
            }
            Ok(ehttp::streaming::Part::Chunk(chunk)) => {
                if chunk.is_empty() {
                    FONT_MANAGER
                        .load_from_bytes("gallery-cjk", &state.bytes, 0)
                        .map(|()| {
                            FONT_MANAGER.set_fallback_chain(&["gallery-cjk"]);
                            true
                        })
                        .map_err(|error| error.to_string())
                } else if state.bytes.len().saturating_add(chunk.len()) > MAX_BYTES {
                    Err("Font exceeds 50 MiB".into())
                } else {
                    state.bytes.extend_from_slice(&chunk);
                    Ok(false)
                }
            }
        };
        let flow = match outcome {
            Ok(true) => {
                state.bytes = Vec::new();
                state.done = true;
                toast::dismiss(toast_id);
                toast::success("Multilingual font ready");
                ControlFlow::Break(())
            }
            Ok(false) => ControlFlow::Continue(()),
            Err(error) => {
                state.bytes = Vec::new();
                state.error = Some(error);
                ControlFlow::Break(())
            }
        };
        ctx.request_repaint();
        flow
    });
}
