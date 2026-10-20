use std::sync::mpsc::{self, Receiver, TryRecvError};

use eframe::egui;
use egui_component::{
    components::{button, switch, text, text_edit},
    consts::{DEFAULT_FONT_FAMILY_ALIAS, FONT_MANAGER},
    font::weight::FontWeight,
    theme::Theme,
};

const GOOGLE_FONT_URL: &str =
    "https://fonts.gstatic.com/s/blackopsone/v21/qWcsB6-ypo7xBdr6Xshe96H3aDvbtw.woff2";
const MAX_FONT_BYTES: usize = 50 * 1024 * 1024;

struct Download {
    response: Receiver<ehttp::Result<ehttp::Response>>,
}

pub struct FontDemo {
    primary: &'static str,
    cjk_fallback: bool,
    status: String,
    font_url: String,
    download: Option<Download>,
}

impl Default for FontDemo {
    fn default() -> Self {
        Self {
            primary: "CJK fallback",
            cjk_fallback: true,
            status: "Choose a font to update the running application.".into(),
            font_url: GOOGLE_FONT_URL.into(),
            download: None,
        }
    }
}

impl FontDemo {
    // Run even when another Gallery page is visible. Network callbacks only deliver
    // data; the UI thread validates and commits it before applying the catalog.
    pub fn poll_download(&mut self, ctx: &egui::Context) {
        let Some(download) = &self.download else {
            return;
        };
        let result = match download.response.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => Err("The font download was interrupted.".into()),
        };
        self.download = None;
        match result.and_then(|response| apply_download(&FONT_MANAGER, response)) {
            Ok(downloaded_bytes) => {
                self.primary = "Downloaded font";
                self.status = format!("Loaded {downloaded_bytes} bytes. Font applied to Gallery.");
            }
            Err(error) => {
                self.status = format!("Could not load font: {error}. Previous font preserved.")
            }
        }
        ctx.request_repaint();
    }

    fn start_download(&mut self, ctx: &egui::Context) {
        let url = self.font_url.trim().to_owned();
        if !(url.starts_with("https://") || url.starts_with("http://")) {
            self.status = "Enter a direct HTTP or HTTPS font URL.".into();
            return;
        }
        let (sender, response) = mpsc::channel();
        self.download = Some(Download { response });
        self.status = "Downloading font…".into();
        let ctx = ctx.clone();
        let request =
            ehttp::Request::get(url).with_timeout(Some(std::time::Duration::from_secs(30)));
        ehttp::fetch(request, move |result| {
            let _ = sender.send(result);
            ctx.request_repaint();
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        text::render(ui, "Dynamic fonts", text::Style::new_heading(theme));
        text::render(
            ui,
            "Switch the Gallery font at runtime. Text and inputs use our components.",
            text::Style::new(theme),
        );
        ui.horizontal(|ui| {
            for (label, font_name) in [("Sans", "Ubuntu-Light"), ("Monospace", "Hack")] {
                if button::render(ui, label, button::Style::new_outline_sm(theme)).clicked() {
                    // These bytes could also come from an asynchronous download.
                    let fonts = egui::FontDefinitions::default();
                    let data = &fonts.font_data[font_name];
                    self.download = None; // A newer user choice supersedes any in-flight download.
                    match FONT_MANAGER
                        .replace_family(DEFAULT_FONT_FAMILY_ALIAS, &[(data.as_ref().as_ref(), 0)])
                    {
                        Ok(()) => {
                            self.primary = label;
                            self.status =
                                format!("Queued {label}; egui activates fonts on the next pass.");
                        }
                        Err(error) => self.status = error.to_string(),
                    }
                    ui.ctx().request_repaint();
                }
            }
            if button::render(ui, "Clear primary", button::Style::new_outline_sm(theme)).clicked() {
                self.download = None;
                FONT_MANAGER.remove_family(DEFAULT_FONT_FAMILY_ALIAS);
                self.primary = "CJK fallback";
                self.status = "Primary removed; the fallback chain is used.".into();
                ui.ctx().request_repaint();
            }
            if button::render(ui, "Try invalid font", button::Style::new_outline_sm(theme))
                .clicked()
            {
                let error = FONT_MANAGER
                    .replace_family(DEFAULT_FONT_FAMILY_ALIAS, &[(b"invalid font", 0)])
                    .expect_err("Invalid data must be rejected");
                self.status = format!("Rejected: {error}. Previous font preserved.");
                ui.ctx().request_repaint();
            }
        });
        ui.add_space(8.0);
        text::render(ui, "Font URL", text::Style::new(theme));
        let loading = self.download.is_some();
        let submit = ui
            .horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let state = if loading {
                        button::ButtonState::Loading
                    } else {
                        button::ButtonState::None
                    };
                    let clicked = button::render(
                        ui,
                        "Load font",
                        button::Style::new_solid_sm(theme).state(state),
                    )
                    .clicked();
                    let input = ui
                        .push_id("download_font_url", |ui| {
                            text_edit::render(
                                ui,
                                &mut self.font_url,
                                text_edit::Style::new_outline_sm(theme)
                                    .placeholder("https://example.com/font.woff2")
                                    .width(Some(ui.available_width())),
                            )
                        })
                        .inner;
                    let enter =
                        input.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
                    !loading && (enter || clicked)
                })
                .inner
            })
            .inner;
        if submit {
            self.start_download(ui.ctx());
        }
        text::render(
            ui,
            "Direct TTF, OTF, TTC or WOFF2 URL. Browser downloads require CORS permission from the font server.",
            text::Style::new(theme),
        );
        if self.download.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                text::render(ui, "Downloading font…", text::Style::new(theme));
                if button::render(ui, "Cancel", button::Style::new_outline_sm(theme)).clicked() {
                    // Dropping the receiver prevents a late response from changing the font.
                    self.download = None;
                    self.status = "Download canceled. Current font preserved.".into();
                }
            });
        }
        ui.horizontal(|ui| {
            if switch::render(ui, &mut self.cjk_fallback, switch::Style::new_sm(theme)).changed() {
                FONT_MANAGER.set_fallback_chain(if self.cjk_fallback {
                    &["gallery-cjk"]
                } else {
                    &[]
                });
                ui.ctx().request_repaint();
            }
            text::render(ui, "Enable CJK glyph fallback", text::Style::new(theme));
        });
        text::render(
            ui,
            &format!("Primary: {}", self.primary),
            text::Style::new(theme),
        );
        text::render(ui, &self.status, text::Style::new(theme).truncate(true));
        ui.add_space(16.0);
        text::render(ui, "BLACK OPS ONE", text::Style::new(theme).size(32.0));
        ui.add_space(8.0);
        for (label, weight) in [
            ("Regular", FontWeight::Regular),
            ("Bold (nearest available weight)", FontWeight::Bold),
        ] {
            text::render(ui, label, text::Style::new(theme));
            text::render(
                ui,
                "Hello, dynamic fonts! 0123456789 — 中文字体回退",
                text::Style::new(theme).size(24.0).font_weight(weight),
            );
        }
        ui.add_space(12.0);
        text::render(
            ui,
            "Sans / Monospace do not contain CJK glyphs: toggle fallback to compare.",
            text::Style::new(theme),
        );
    }
}

fn apply_download(
    manager: &egui_component::font::manager::FontManager,
    response: ehttp::Response,
) -> Result<usize, String> {
    if !response.ok {
        return Err(format!("HTTP {} {}", response.status, response.status_text));
    }
    let downloaded_bytes = response.bytes.len();
    let bytes = decode_font(response.bytes)?;
    manager
        .replace_family(DEFAULT_FONT_FAMILY_ALIAS, &[(bytes.as_slice(), 0)])
        .map_err(|error| format!("Invalid font file ({error}); use a direct font file URL"))?;
    Ok(downloaded_bytes)
}

// The catalog accepts uncompressed OpenType bytes. WOFF2 is a transport format,
// so decompression belongs to this download example rather than the component library.
fn decode_font(bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    if bytes.len() > MAX_FONT_BYTES {
        return Err("Font exceeds the 50 MiB size limit".into());
    }
    let decoded = if bytes.starts_with(b"wOF2") {
        if let Some(size) = bytes.get(16..20) {
            let declared_size = u32::from_be_bytes(size.try_into().unwrap()) as usize;
            if declared_size > MAX_FONT_BYTES {
                return Err("Decompressed font exceeds the 50 MiB size limit".into());
            }
        }
        wuff::decompress_woff2(&bytes).map_err(|error| format!("Invalid WOFF2 font ({error:?})"))?
    } else {
        bytes
    };
    if decoded.len() > MAX_FONT_BYTES {
        return Err("Decompressed font exceeds the 50 MiB size limit".into());
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_component::font::manager::FontManager;

    fn response(status: u16, bytes: Vec<u8>) -> ehttp::Response {
        ehttp::Response {
            url: GOOGLE_FONT_URL.into(),
            ok: (200..300).contains(&status),
            status,
            status_text: String::new(),
            headers: Default::default(),
            bytes,
        }
    }

    #[test]
    fn google_woff2_download_decodes_and_registers() {
        let bytes = include_bytes!("../../assets/font-fixtures/BlackOpsOne-latin.woff2");
        let manager = FontManager::with_base_fonts(egui::FontDefinitions::empty());
        assert_eq!(
            apply_download(&manager, response(200, bytes.to_vec())).unwrap(),
            bytes.len()
        );
        let ctx = egui::Context::default();
        assert!(manager.apply(&ctx));
        ctx.begin_pass(Default::default());
        let family = manager.default_egui_font_family(&ctx);
        assert_ne!(family, egui::FontFamily::Proportional);
        // The reconstructed OpenType file must also be accepted by egui's renderer.
        ctx.fonts_mut(|fonts| {
            let font = egui::FontId::new(24.0, family);
            assert!(fonts.has_glyph(&font, 'A'));
            assert!(fonts.has_glyph(&font, 'H'));
            assert!(fonts.has_glyph(&font, '0'));
        });
        let _ = ctx.end_pass();
    }

    #[test]
    fn http_errors_and_invalid_fonts_preserve_the_current_catalog() {
        let manager = FontManager::new();
        let ctx = egui::Context::default();
        let fonts = egui::FontDefinitions::default();
        let regular = fonts.font_data["Hack"].as_ref().as_ref().to_vec();
        apply_download(&manager, response(200, regular)).unwrap();
        manager.apply(&ctx);
        for (status, data) in [
            (404, b"not found".to_vec()),
            (200, b"<html>not a font</html>".to_vec()),
            (200, b"wOF2".to_vec()),
        ] {
            assert!(apply_download(&manager, response(status, data)).is_err());
            assert!(!manager.apply(&ctx));
        }
    }

    #[test]
    fn rejects_oversized_woff2_before_decompression() {
        let mut bytes = vec![0; 48];
        bytes[..4].copy_from_slice(b"wOF2");
        bytes[16..20].copy_from_slice(&((MAX_FONT_BYTES + 1) as u32).to_be_bytes());
        assert!(decode_font(bytes).unwrap_err().contains("size limit"));
    }
}
