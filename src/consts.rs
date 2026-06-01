use std::sync::LazyLock;

use crate::font::manager::FontManager;

pub const DEFAULT_FONT_FAMILY_ALIAS: &str = "NotoSans";
pub static FONT_MANAGER: LazyLock<FontManager> = LazyLock::new(|| FontManager::new());
