use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
#[cfg(not(target_arch = "wasm32"))]
use std::{fs::File, io::Read, path};

use parking_lot::RwLock;

use crate::{consts::DEFAULT_FONT_FAMILY_ALIAS, font::weight::FontWeight};

const MAX_FONT_FILE_SIZE: usize = 50 * 1024 * 1024;
static NEXT_MANAGER_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone)]
pub struct FontFace {
    pub data: Arc<[u8]>,
    pub index: usize,
    pub weight: FontWeight,
    pub italic: bool,
}

#[cfg(test)]
#[path = "manager_tests.rs"]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FamilyKey(String);

impl FamilyKey {
    pub fn new(name: &str) -> Self {
        Self(name.trim().to_lowercase())
    }
}

#[derive(Clone)]
struct RegisteredFace {
    face: FontFace,
    // Prepared on ingestion, then shared across revisions and contexts.
    egui_data: Arc<egui::FontData>,
}

struct FontState {
    revision: u64,
    families: BTreeMap<FamilyKey, Vec<RegisteredFace>>,
    fallback_chain: Vec<FamilyKey>,
    base: egui::FontDefinitions,
    snapshot: Option<Arc<FontSnapshot>>,
}

impl FontState {
    fn changed(&mut self) {
        self.revision += 1;
        self.snapshot = None;
    }
}

struct FontSnapshot {
    revision: u64,
    definitions: egui::FontDefinitions,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct SubmittedFonts {
    manager_id: usize,
    revision: u64,
}

/// Mutable font catalog. Call [`Self::apply`] before rendering each frame.
///
/// Updates are atomic; egui activates submitted definitions on its next pass.
/// One manager may serve multiple contexts. A context has one font catalog owner:
/// supply application fonts through [`Self::set_base_fonts`] rather than calling
/// `Context::set_fonts` independently after applying this manager.
pub struct FontManager {
    id: usize,
    state: RwLock<FontState>,
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FontManager {
    pub fn new() -> Self {
        Self::with_base_fonts(egui::FontDefinitions::default())
    }

    /// Preserve application font definitions, including proportional/monospace families.
    pub fn with_base_fonts(base: egui::FontDefinitions) -> Self {
        Self {
            id: NEXT_MANAGER_ID.fetch_add(1, Ordering::Relaxed),
            state: RwLock::new(FontState {
                revision: 0,
                families: BTreeMap::new(),
                fallback_chain: Vec::new(),
                base,
                snapshot: None,
            }),
        }
    }

    pub fn set_base_fonts(&self, base: egui::FontDefinitions) {
        let mut state = self.state.write();
        if state.base != base {
            state.base = base;
            state.changed();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_from_file(&self, family: &str, path: impl AsRef<path::Path>) -> anyhow::Result<()> {
        let path = path.as_ref();
        anyhow::ensure!(
            path.is_absolute(),
            "Font path must be absolute: {}",
            path.display()
        );
        let mut file = File::open(path)?;
        anyhow::ensure!(
            file.metadata()?.len() <= MAX_FONT_FILE_SIZE as u64,
            "Font file is too large (>50MB)"
        );
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        self.load_from_bytes(family, &data, 0)
    }

    fn parse_face(data: &[u8], index: usize) -> anyhow::Result<RegisteredFace> {
        anyhow::ensure!(
            data.len() <= MAX_FONT_FILE_SIZE,
            "Font file is too large (>50MB)"
        );
        let index_u32 = u32::try_from(index)?;
        let parsed = ttf_parser::Face::parse(data, index_u32)
            .map_err(|e| anyhow::anyhow!("Failed to parse font: {e}"))?;
        let face = FontFace {
            data: Arc::from(data),
            index,
            weight: FontWeight::from_numeric(parsed.weight().to_number()),
            italic: parsed.is_italic(),
        };
        let mut egui_data = egui::FontData::from_owned(data.to_vec());
        egui_data.index = index_u32;
        Ok(RegisteredFace {
            face,
            egui_data: Arc::new(egui_data),
        })
    }

    fn family_key(family: &str) -> anyhow::Result<FamilyKey> {
        let key = FamilyKey::new(family);
        anyhow::ensure!(!key.0.is_empty(), "Font family must not be empty");
        Ok(key)
    }

    /// Insert or replace the face with this family, parsed weight and italic style.
    /// Invalid data leaves the catalog unchanged. Call `apply` to submit changes.
    pub fn load_from_bytes(&self, family: &str, data: &[u8], index: usize) -> anyhow::Result<()> {
        let key = Self::family_key(family)?;
        let face = Self::parse_face(data, index)?;
        let mut state = self.state.write();
        let faces = state.families.entry(key).or_default();
        if let Some(existing) = faces
            .iter_mut()
            .find(|f| f.face.weight == face.face.weight && f.face.italic == face.face.italic)
        {
            if existing.egui_data == face.egui_data {
                return Ok(());
            }
            *existing = face;
        } else {
            faces.push(face);
        }
        faces.sort_by_key(|f| (f.face.weight, f.face.italic));
        state.changed();
        Ok(())
    }

    /// Atomically replace a complete family with `(font bytes, collection index)` pairs.
    /// All faces are validated before committing. Empty input and duplicate weight/style
    /// slots are errors; use `remove_family` to remove a family.
    pub fn replace_family(&self, family: &str, sources: &[(&[u8], usize)]) -> anyhow::Result<()> {
        let key = Self::family_key(family)?;
        anyhow::ensure!(
            !sources.is_empty(),
            "A font family must contain at least one face"
        );
        let mut faces = sources
            .iter()
            .map(|(data, index)| Self::parse_face(data, *index))
            .collect::<anyhow::Result<Vec<_>>>()?;
        faces.sort_by_key(|f| (f.face.weight, f.face.italic));
        anyhow::ensure!(
            !faces
                .windows(2)
                .any(|pair| pair[0].face.weight == pair[1].face.weight
                    && pair[0].face.italic == pair[1].face.italic),
            "Duplicate weight/style in font family"
        );
        let mut state = self.state.write();
        if state.families.get(&key).is_some_and(|old| {
            old.len() == faces.len()
                && old
                    .iter()
                    .zip(&faces)
                    .all(|(a, b)| a.egui_data == b.egui_data)
        }) {
            return Ok(());
        }
        state.families.insert(key, faces);
        state.changed();
        Ok(())
    }

    pub fn remove_family(&self, family: &str) -> bool {
        let mut state = self.state.write();
        if state.families.remove(&FamilyKey::new(family)).is_some() {
            state.changed();
            true
        } else {
            false
        }
    }

    pub fn remove_face(&self, family: &str, weight: FontWeight, italic: bool) -> bool {
        let mut state = self.state.write();
        let key = FamilyKey::new(family);
        let Some(faces) = state.families.get_mut(&key) else {
            return false;
        };
        let len = faces.len();
        faces.retain(|f| f.face.weight != weight || f.face.italic != italic);
        if faces.len() == len {
            return false;
        }
        if faces.is_empty() {
            state.families.remove(&key);
        }
        state.changed();
        true
    }

    /// Enumerate and load system fonts by family name.
    ///
    /// Not available on wasm32 — use `load_from_bytes` with bundled font data instead:
    /// ```rust
    /// #[cfg(target_arch = "wasm32")]
    /// font_manager.load_from_bytes(
    ///     "my-font",
    ///     include_bytes!("../assets/MyFont-Regular.ttf"),
    ///     0,
    /// )?;
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_system_font_with_fallbacks(
        &self,
        alias: &str,
        families: &[&str],
    ) -> anyhow::Result<()> {
        use font_kit::family_name::FamilyName;
        use font_kit::properties::Properties;
        use font_kit::source::SystemSource;

        let source = SystemSource::new();

        let mut loaded_count = 0usize;
        let mut found_family: Option<&str> = None;

        'outer: for &family_name in families {
            let family_handle = match source.select_family_by_name(family_name) {
                Ok(h) => h,
                Err(_) => continue,
            };

            let handles = family_handle.fonts();
            if handles.is_empty() {
                continue;
            }

            for handle in handles {
                let (font_data, font_index) = match handle {
                    font_kit::handle::Handle::Path { path, font_index } => {
                        let mut file = match File::open(&path) {
                            Ok(f) => f,
                            Err(e) => {
                                tracing::warn!("Cannot open font file {:?}: {}", path, e);
                                continue;
                            }
                        };
                        let mut data = Vec::new();
                        if let Err(e) = file.read_to_end(&mut data) {
                            tracing::warn!("Cannot read font file {:?}: {}", path, e);
                            continue;
                        }
                        (data, *font_index as usize)
                    }
                    font_kit::handle::Handle::Memory { bytes, font_index } => {
                        (bytes.to_vec(), *font_index as usize)
                    }
                };

                match self.load_from_bytes(alias, &font_data, font_index) {
                    Ok(()) => loaded_count += 1,
                    Err(e) => tracing::warn!("Skipping font face in '{}': {}", family_name, e),
                }
            }

            if loaded_count > 0 {
                found_family = Some(family_name);
                break 'outer;
            }
        }

        if loaded_count > 0 {
            tracing::info!(
                "Loaded {} face(s) for alias '{}' from system family '{}'",
                loaded_count,
                alias,
                found_family.unwrap_or("unknown")
            );
            return Ok(());
        }

        tracing::warn!(
            "None of {:?} found as full families; falling back to select_best_match",
            families
        );
        let mut family_names: Vec<FamilyName> = families
            .iter()
            .map(|&f| FamilyName::Title(f.to_string()))
            .collect();
        family_names.push(FamilyName::SansSerif);

        let font_handle = source
            .select_best_match(&family_names, &Properties::new())
            .map_err(|e| anyhow::anyhow!("Failed to find any suitable system font: {}", e))?;

        let (font_data, font_index) = match font_handle {
            font_kit::handle::Handle::Path { path, font_index } => {
                let mut file = File::open(&path)?;
                let mut data = Vec::new();
                file.read_to_end(&mut data)?;
                (data, font_index as usize)
            }
            font_kit::handle::Handle::Memory { bytes, font_index } => {
                (bytes.to_vec(), font_index as usize)
            }
        };

        self.load_from_bytes(alias, &font_data, font_index)?;
        tracing::info!("Loaded single-weight fallback font for alias '{}'", alias);
        Ok(())
    }

    /// Glyph fallback order for managed families. Missing families are skipped until loaded.
    /// Each family contributes its closest weight, followed by the base proportional fonts.
    pub fn set_fallback_chain(&self, families: &[&str]) {
        let mut chain = Vec::new();
        for family in families {
            let key = FamilyKey::new(family);
            if !key.0.is_empty() && !chain.contains(&key) {
                chain.push(key);
            }
        }
        let mut state = self.state.write();
        if state.fallback_chain != chain {
            state.fallback_chain = chain;
            state.changed();
        }
    }

    fn font_key(&self, family: &FamilyKey, weight: FontWeight, italic: bool) -> String {
        format!(
            "egui_component/{}/{}/{}_{}",
            self.id,
            family.0,
            weight.numeric(),
            if italic { "italic" } else { "normal" }
        )
    }

    fn closest(
        faces: &[RegisteredFace],
        weight: FontWeight,
        italic: bool,
    ) -> Option<&RegisteredFace> {
        faces.iter().min_by_key(|f| {
            (
                f.face.weight.numeric().abs_diff(weight.numeric()),
                f.face.italic != italic,
            )
        })
    }

    fn build_definitions(&self, state: &FontState) -> egui::FontDefinitions {
        let mut definitions = state.base.clone();
        // egui compares against active definitions, not an already queued update.
        // A revision stamp makes A -> B -> A within one pass replace the pending B
        // too. The reserved, unused family carries no font bytes or glyphs.
        definitions.families.insert(
            egui::FontFamily::Name(
                format!("egui_component/{}/revision/{}", self.id, state.revision).into(),
            ),
            Vec::new(),
        );
        for (family, faces) in &state.families {
            for registered in faces {
                let face = &registered.face;
                definitions.font_data.insert(
                    self.font_key(family, face.weight, face.italic),
                    registered.egui_data.clone(),
                );
            }
        }
        let mut families: Vec<_> = state.families.keys().cloned().collect();
        let default = FamilyKey::new(DEFAULT_FONT_FAMILY_ALIAS);
        if !families.contains(&default) {
            families.push(default);
        }
        for family in families {
            for weight in FontWeight::ALL {
                for italic in [false, true] {
                    let mut chain = Vec::new();
                    for fallback in std::iter::once(&family).chain(&state.fallback_chain) {
                        if let Some(face) = state
                            .families
                            .get(fallback)
                            .and_then(|faces| Self::closest(faces, weight, italic))
                        {
                            let key = self.font_key(fallback, face.face.weight, face.face.italic);
                            if !chain.contains(&key) {
                                chain.push(key);
                            }
                        }
                    }
                    if let Some(base) = state.base.families.get(&egui::FontFamily::Proportional) {
                        for key in base {
                            if !chain.contains(key) {
                                chain.push(key.clone());
                            }
                        }
                    }
                    definitions.families.insert(
                        egui::FontFamily::Name(self.font_key(&family, weight, italic).into()),
                        chain,
                    );
                }
            }
        }
        definitions
    }

    fn snapshot(&self) -> Arc<FontSnapshot> {
        if let Some(snapshot) = &self.state.read().snapshot {
            return snapshot.clone();
        }
        let mut state = self.state.write();
        if let Some(snapshot) = &state.snapshot {
            return snapshot.clone();
        }
        let snapshot = Arc::new(FontSnapshot {
            revision: state.revision,
            definitions: self.build_definitions(&state),
        });
        state.snapshot = Some(snapshot.clone());
        snapshot
    }

    /// Submit changed fonts once per context/version. Safe to call every frame.
    /// Returns true when a new catalog was queued. egui activates it next pass;
    /// queries below keep using the currently active definitions in the meantime.
    pub fn apply(&self, ctx: &egui::Context) -> bool {
        let snapshot = self.snapshot();
        let submitted = SubmittedFonts {
            manager_id: self.id,
            revision: snapshot.revision,
        };
        let id = egui::Id::new("egui_component/font_catalog");
        if ctx.data(|data| data.get_temp::<SubmittedFonts>(id)) == Some(submitted) {
            return false;
        }
        ctx.set_fonts(snapshot.definitions.clone());
        ctx.data_mut(|data| data.insert_temp(id, submitted));
        ctx.request_repaint();
        true
    }

    /// Compatibility entry point; now applies revisions instead of running only once.
    pub fn set_egui_fonts(&self, ctx: &egui::Context) {
        self.apply(ctx);
    }

    /// Resolve a family against this context's active fonts. Call during an egui pass.
    pub fn get_egui_font_family(
        &self,
        ctx: &egui::Context,
        family: &str,
        weight: FontWeight,
    ) -> egui::FontFamily {
        self.get_egui_font_family_with_style(ctx, family, weight, false)
    }

    pub fn get_egui_font_family_with_style(
        &self,
        ctx: &egui::Context,
        family: &str,
        weight: FontWeight,
        italic: bool,
    ) -> egui::FontFamily {
        let requested = egui::FontFamily::Name(
            self.font_key(&FamilyKey::new(family), weight, italic)
                .into(),
        );
        let default = egui::FontFamily::Name(
            self.font_key(&FamilyKey::new(DEFAULT_FONT_FAMILY_ALIAS), weight, italic)
                .into(),
        );
        ctx.fonts(|fonts| {
            let families = &fonts.definitions().families;
            if families.contains_key(&requested) {
                requested
            } else if families.contains_key(&default) {
                default
            } else {
                egui::FontFamily::Proportional
            }
        })
    }

    pub fn default_egui_font_family(&self, ctx: &egui::Context) -> egui::FontFamily {
        self.get_egui_font_family(ctx, DEFAULT_FONT_FAMILY_ALIAS, FontWeight::Regular)
    }

    pub fn find_closest_weight(faces: &[FontFace], target: FontWeight) -> Option<&FontFace> {
        faces
            .iter()
            .min_by_key(|f| (f.weight.numeric().abs_diff(target.numeric()), f.italic))
    }
}
