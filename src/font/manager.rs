use std::path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{fs::File, io::Read};

use dashmap::{DashMap, DashSet};
use parking_lot::RwLock;

use crate::consts::DEFAULT_FONT_FAMILY_ALIAS;
use crate::font::weight::FontWeight;

static MAX_FONT_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50MB

pub struct FontFace {
    pub data: Arc<[u8]>,
    pub index: usize,
    pub weight: FontWeight,
    pub italic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FamilyKey(String);

impl FamilyKey {
    pub fn new(name: &str) -> Self {
        Self(name.to_lowercase())
    }
}

pub struct FontManager {
    families: DashMap<FamilyKey, Vec<FontFace>>,
    fallback_chain: RwLock<Vec<FamilyKey>>,
    /// egui font key → registered (e.g. "noto sans_400")
    registered_egui_font_keys: DashSet<String>,
    /// family name → sorted list of registered weights; built once in set_egui_fonts
    registered_weights_by_family: DashMap<String, Vec<FontWeight>>,
    /// Guards set_egui_fonts so it only applies once
    fonts_applied: AtomicBool,
}

impl FontManager {
    pub fn new() -> Self {
        Self {
            families: DashMap::new(),
            fallback_chain: RwLock::new(Vec::new()),
            registered_egui_font_keys: DashSet::new(),
            registered_weights_by_family: DashMap::new(),
            fonts_applied: AtomicBool::new(false),
        }
    }

    pub fn load_from_file(&self, family: &str, path: impl AsRef<path::Path>) -> anyhow::Result<()> {
        let path_obj = path.as_ref();
        if !path_obj.is_absolute() {
            anyhow::bail!("Font path must be absolute: {}", path_obj.display());
        }

        let mut file = File::open(path_obj)?;

        let metadata = file.metadata()?;
        if metadata.len() > MAX_FONT_FILE_SIZE {
            anyhow::bail!("Font file is too large (>50MB): {}", path_obj.display());
        }

        let mut font_data = Vec::new();
        file.read_to_end(&mut font_data)?;

        self.load_from_bytes(family, &font_data, 0)
    }

    /// Parse weight/italic from raw font bytes using ttf-parser (pure Rust, wasm32-compatible).
    /// Returns (weight, italic).
    ///
    /// Uses `Face::weight()` → `ttf_parser::Weight` and `Face::is_italic()`, both available
    /// directly on `Face` since ttf-parser 0.16+.  `Weight::to_number()` gives the numeric
    /// OS/2 usWeightClass value (100–900).
    fn parse_font_properties(data: &[u8], index: usize) -> anyhow::Result<(FontWeight, bool)> {
        let face = ttf_parser::Face::parse(data, index as u32)
            .map_err(|e| anyhow::anyhow!("Failed to parse font: {}", e))?;

        let weight = FontWeight::from_numeric(face.weight().to_number());
        let italic = face.is_italic();

        Ok((weight, italic))
    }

    pub fn load_from_bytes(&self, family: &str, data: &[u8], index: usize) -> anyhow::Result<()> {
        let arc_data: Arc<[u8]> = Arc::from(data);

        let (weight, italic) = Self::parse_font_properties(&arc_data, index)?;

        let face = FontFace {
            data: arc_data,
            index,
            weight,
            italic,
        };

        let key = FamilyKey::new(family);
        let mut entry = self.families.entry(key).or_insert_with(Vec::new);
        entry.push(face);
        entry.sort_by_key(|f| f.weight);

        Ok(())
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

    pub fn set_fallback_chain(&self, families: &[&str]) {
        let chain: Vec<FamilyKey> = families.iter().map(|&f| FamilyKey::new(f)).collect();
        *self.fallback_chain.write() = chain;
    }

    /// Look up the egui FontFamily for a given (family, weight) pair.
    ///
    /// Hot-path lookup uses the pre-built `registered_weights_by_family` index instead
    /// of iterating over `FontWeight::ALL` and calling `format!` + `DashSet::contains`
    /// for every weight on every call. The index is populated once inside `set_egui_fonts`.
    pub fn get_egui_font_family(&self, family: &str, weight: FontWeight) -> egui::FontFamily {
        let exact_key = format!("{}_{}", family, weight.numeric());
        if self.registered_egui_font_keys.contains(&exact_key) {
            return egui::FontFamily::Name(exact_key.into());
        }

        // Closest-weight fallback: consult the pre-built per-family weight list.
        if let Some(weights) = self.registered_weights_by_family.get(family) {
            let target_num = weight.numeric() as i32;
            if let Some(&closest) = weights
                .iter()
                .min_by_key(|w| (w.numeric() as i32 - target_num).unsigned_abs())
            {
                let fallback_key = format!("{}_{}", family, closest.numeric());
                tracing::warn!(
                    "egui font {} not found, fallback to {}",
                    exact_key,
                    fallback_key
                );
                return egui::FontFamily::Name(fallback_key.into());
            }
        }

        // Family not registered at all — fall back to the default alias.
        if family == DEFAULT_FONT_FAMILY_ALIAS {
            tracing::warn!(
                "egui default font {} not found, fallback to egui::FontFamily::Proportional",
                exact_key
            );
            return egui::FontFamily::Proportional;
        }

        tracing::warn!("egui font {} not found, fallback to default", exact_key);
        self.default_egui_font_family()
    }

    pub fn default_egui_font_family(&self) -> egui::FontFamily {
        self.get_egui_font_family(DEFAULT_FONT_FAMILY_ALIAS, FontWeight::Regular)
    }

    /// Register all loaded font faces with egui.
    ///
    /// Idempotent: subsequent calls are no-ops (calling `egui::Context::set_fonts`
    /// flushes the entire glyph cache, so duplicate calls must be avoided).
    pub fn set_egui_fonts(&self, ctx: &egui::Context) {
        if self.fonts_applied.swap(true, Ordering::SeqCst) {
            tracing::warn!("set_egui_fonts called more than once; skipping duplicate call");
            return;
        }

        let mut font_defs = egui::FontDefinitions::default();

        for entry in self.families.iter() {
            let family_name = &entry.key().0;

            for face in entry.value().iter() {
                let egui_key = format!("{}_{}", family_name, face.weight.numeric());

                // egui FontData::from_owned takes Vec<u8>; clone from our Arc<[u8]> once.
                // The Arc itself stays alive for other uses (e.g. direct rasterisation).
                let egui_font_data = egui::FontData::from_owned(face.data.to_vec());

                font_defs
                    .font_data
                    .insert(egui_key.clone(), Arc::new(egui_font_data));

                font_defs
                    .families
                    .entry(egui::FontFamily::Name(egui_key.clone().into()))
                    .or_default()
                    .push(egui_key.clone());

                self.registered_egui_font_keys.insert(egui_key.clone());

                // Build the per-family weight index while iterating, so
                // get_egui_font_family never has to scan FontWeight::ALL.
                self.registered_weights_by_family
                    .entry(family_name.clone())
                    .or_default()
                    .push(face.weight);
            }
        }

        // Keep each per-family weight list sorted for min_by_key to stay predictable.
        for mut weights in self.registered_weights_by_family.iter_mut() {
            weights.sort_by_key(|w| w.numeric());
        }

        ctx.set_fonts(font_defs);
        ctx.request_repaint();
    }

    pub fn find_closest_weight<'a>(
        faces: &'a [FontFace],
        target: FontWeight,
    ) -> Option<&'a FontFace> {
        if faces.is_empty() {
            return None;
        }

        if let Some(face) = faces.iter().find(|f| f.weight == target && !f.italic) {
            return Some(face);
        }

        if let Some(face) = faces.iter().find(|f| f.weight == target) {
            return Some(face);
        }

        let target_num = target.numeric();
        let mut best: Option<&FontFace> = None;
        let mut best_dist = u16::MAX;

        for face in faces {
            let dist = (face.weight.numeric() as i32 - target_num as i32).unsigned_abs() as u16;
            if dist < best_dist {
                best_dist = dist;
                best = Some(face);
            }
        }
        best
    }
}
