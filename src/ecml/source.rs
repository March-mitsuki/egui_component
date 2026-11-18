use super::{Error, parser};

/// Explicit sources override the platform/build-mode defaults of [`crate::ecml::source!`].
pub enum Source {
    Inline {
        name: String,
        text: String,
    },
    #[cfg(not(target_arch = "wasm32"))]
    File(std::path::PathBuf),
}
impl Source {
    pub fn inline(name: impl Into<String>, text: impl Into<String>) -> Self {
        Self::Inline {
            name: name.into(),
            text: text.into(),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn file(path: impl Into<std::path::PathBuf>) -> Self {
        Self::File(path.into())
    }
}

#[macro_export]
macro_rules! ecml_source {
    ($path:literal) => {{
        #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
        {
            $crate::ecml::Source::file(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path))
        }
        #[cfg(any(not(debug_assertions), target_arch = "wasm32"))]
        {
            $crate::ecml::Source::inline(
                $path,
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path)),
            )
        }
    }};
}

pub(super) struct Candidate {
    pub text: String,
    pub parsed: Result<parser::Element, Error>,
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::*;
    use std::{
        fs::File,
        io::Read,
        path::Path,
        sync::{Arc, Mutex, mpsc},
        thread,
        time::Duration,
    };

    pub(super) fn read(path: &Path) -> Result<String, String> {
        let mut text = String::new();
        File::open(path)
            .and_then(|file| {
                file.take((parser::MAX_BYTES + 1) as u64)
                    .read_to_string(&mut text)
            })
            .map_err(|e| e.to_string())?;
        if text.len() > parser::MAX_BYTES {
            return Err("template exceeds 256 KiB".into());
        }
        Ok(text)
    }

    #[derive(Default)]
    struct Shared {
        latest: Option<Candidate>,
        context: Option<(egui::Context, egui::ViewportId)>,
    }
    pub(crate) struct Watcher {
        shared: Arc<Mutex<Shared>>,
        stop: mpsc::Sender<()>,
        worker: Option<thread::JoinHandle<()>>,
    }
    impl Watcher {
        pub fn start(path: std::path::PathBuf, initial: String) -> Self {
            let shared = Arc::new(Mutex::new(Shared::default()));
            let work = shared.clone();
            let (stop, rx) = mpsc::channel();
            let worker = thread::spawn(move || {
                let mut previous = Ok(initial);
                while rx.recv_timeout(Duration::from_millis(150))
                    == Err(mpsc::RecvTimeoutError::Timeout)
                {
                    let next = read(&path);
                    if next == previous {
                        continue;
                    }
                    let file = path.display().to_string();
                    let candidate = match &next {
                        Ok(text) => Candidate {
                            parsed: parser::parse(text, &file),
                            text: text.clone(),
                        },
                        Err(message) => Candidate {
                            text: String::new(),
                            parsed: Err(Error::at(&file, "", 0, message)),
                        },
                    };
                    previous = next;
                    let context = {
                        let mut shared = work.lock().unwrap();
                        shared.latest = Some(candidate);
                        shared.context.clone()
                    };
                    if let Some((ctx, viewport)) = context {
                        ctx.request_repaint_of(viewport);
                    }
                }
            });
            Self {
                shared,
                stop,
                worker: Some(worker),
            }
        }
        pub fn poll(&self, ctx: &egui::Context) -> Option<Candidate> {
            let mut shared = self.shared.lock().unwrap();
            shared.context = Some((ctx.clone(), ctx.viewport_id()));
            shared.latest.take()
        }
    }
    impl Drop for Watcher {
        fn drop(&mut self) {
            let _ = self.stop.send(());
            if let Some(worker) = self.worker.take() {
                let _ = worker.join();
            }
        }
    }
    pub(crate) fn load(path: &Path) -> Result<String, Error> {
        read(path).map_err(|message| Error::at(&path.display().to_string(), "", 0, message))
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub(super) use native::{Watcher, load};
