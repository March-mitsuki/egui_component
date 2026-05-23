use std::path;
use std::sync::Arc;
use std::{fs::File, io::Read};

use dashmap::{DashMap, DashSet};
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use parking_lot::RwLock;

use crate::consts::DEFAULT_FONT_FAMILY_ALIAS;
use crate::font::weight::FontWeight;

static MAX_FONT_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50MB

pub struct FontFace {
    pub data: Arc<[u8]>,
    pub index: usize,
    pub weight: FontWeight,
    pub italic: bool,
    pub font: font_kit::font::Font,
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
    registed_egui_font_families: DashSet<String>,
}

impl FontManager {
    pub fn new() -> Self {
        Self {
            families: DashMap::new(),
            fallback_chain: RwLock::new(Vec::new()),
            registed_egui_font_families: DashSet::new(),
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

    pub fn load_from_bytes(&self, family: &str, data: &[u8], index: usize) -> anyhow::Result<()> {
        let data: Arc<[u8]> = Arc::from(data);

        let font_data_vec = data.to_vec();
        let font = font_kit::font::Font::from_bytes(Arc::new(font_data_vec), index as u32)
            .map_err(|e| anyhow::anyhow!("Failed to create font_kit Font: {:?}", e))?;

        let properties = font.properties();
        let weight = FontWeight::from_numeric(properties.weight.0 as u16);
        let italic = match properties.style {
            font_kit::properties::Style::Normal => false,
            _ => true,
        };

        let face = FontFace {
            data,
            index,
            weight,
            italic,
            font,
        };

        let key = FamilyKey::new(family);
        let mut entry = self.families.entry(key.clone()).or_insert_with(Vec::new);
        entry.push(face);
        entry.sort_by_key(|f| f.weight);

        Ok(())
    }

    pub fn load_system_font_with_fallbacks(
        &self,
        alias: &str,
        families: &[&str],
    ) -> anyhow::Result<()> {
        let source = SystemSource::new();

        // Try each candidate family in order; load ALL faces from the first one found.
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

        // Last-resort fallback: select_best_match with SansSerif generic.
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

    pub fn get_font(
        &self,
        family: &str,
        weight: FontWeight,
    ) -> anyhow::Result<font_kit::font::Font> {
        let key = FamilyKey::new(family);

        if let Some(faces) = self.families.get(&key) {
            if let Some(face) = Self::find_closest_weight(&faces, weight) {
                return Ok(face.font.clone());
            }
        }

        // Try fallbacks
        let fallbacks = self.fallback_chain.read().clone();
        for fallback_key in fallbacks {
            if fallback_key == key {
                continue;
            }
            if let Some(faces) = self.families.get(&fallback_key) {
                if let Some(face) = Self::find_closest_weight(&faces, weight) {
                    return Ok(face.font.clone());
                }
            }
        }

        anyhow::bail!("Font not found for family: {} weight: {:?}", family, weight)
    }

    pub fn get_font_regular(&self, family: &str) -> anyhow::Result<font_kit::font::Font> {
        self.get_font(family, FontWeight::Regular)
    }

    pub fn get_egui_font_family(&self, family: &str, weight: FontWeight) -> egui::FontFamily {
        let name = format!("{}_{}", family, weight.numeric());
        if self.registed_egui_font_families.contains(&name) {
            return egui::FontFamily::Name(name.into());
        }

        // Fallback: find the closest registered weight within the same family.
        let target_num = weight.numeric() as i32;
        let mut candidates: Vec<FontWeight> = FontWeight::ALL
            .iter()
            .filter(|&&w| {
                let n = format!("{}_{}", family, w.numeric());
                self.registed_egui_font_families.contains(&n)
            })
            .copied()
            .collect();
        candidates.sort_by_key(|w| (w.numeric() as i32 - target_num).unsigned_abs());

        if let Some(&closest) = candidates.first() {
            let fallback_name = format!("{}_{}", family, closest.numeric());
            tracing::debug!(
                "egui font {} not found, fallback to {}",
                name,
                fallback_name
            );
            return egui::FontFamily::Name(fallback_name.into());
        }

        // Fallback to default font family when no face from this family is registered.
        tracing::debug!("egui font {} not found, fallback to default", name);
        self.default_egui_font_family()
    }

    pub fn default_egui_font_family(&self) -> egui::FontFamily {
        self.get_egui_font_family(DEFAULT_FONT_FAMILY_ALIAS, FontWeight::Regular)
    }

    pub fn set_egui_fonts(&self, ctx: &egui::Context) {
        let mut font_defs = egui::FontDefinitions::default();

        for entry in self.families.iter() {
            let family_name = &entry.key().0;

            for face in entry.value().iter() {
                let egui_key = format!("{}_{}", family_name, face.weight.numeric());

                let egui_font_data = egui::FontData::from_owned(face.data.to_vec());

                font_defs
                    .font_data
                    .insert(egui_key.clone(), std::sync::Arc::new(egui_font_data));

                let egui_family = egui::FontFamily::Name(egui_key.clone().into());
                font_defs
                    .families
                    .entry(egui_family)
                    .or_default()
                    .push(egui_key.clone());

                self.registed_egui_font_families.insert(egui_key);
            }
        }

        ctx.set_fonts(font_defs);
        ctx.request_repaint();
    }

    fn find_closest_weight<'a>(faces: &'a [FontFace], target: FontWeight) -> Option<&'a FontFace> {
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
