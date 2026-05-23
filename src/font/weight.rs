use serde::{Deserialize, Serialize};

/// CSS font-weight mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FontWeight {
    Thin,       // 100
    ExtraLight, // 200
    Light,      // 300
    Regular,    // 400
    Medium,     // 500
    SemiBold,   // 600
    Bold,       // 700
    ExtraBold,  // 800
    Black,      // 900
}

impl FontWeight {
    pub const ALL: [Self; 9] = [
        Self::Thin,
        Self::ExtraLight,
        Self::Light,
        Self::Regular,
        Self::Medium,
        Self::SemiBold,
        Self::Bold,
        Self::ExtraBold,
        Self::Black,
    ];

    pub fn numeric(&self) -> u16 {
        match self {
            Self::Thin => 100,
            Self::ExtraLight => 200,
            Self::Light => 300,
            Self::Regular => 400,
            Self::Medium => 500,
            Self::SemiBold => 600,
            Self::Bold => 700,
            Self::ExtraBold => 800,
            Self::Black => 900,
        }
    }

    /// Infer weight from font-kit numeric value
    pub fn from_numeric(value: u16) -> Self {
        match value {
            0..=150 => Self::Thin,
            151..=250 => Self::ExtraLight,
            251..=350 => Self::Light,
            351..=450 => Self::Regular,
            451..=550 => Self::Medium,
            551..=650 => Self::SemiBold,
            651..=750 => Self::Bold,
            751..=850 => Self::ExtraBold,
            _ => Self::Black,
        }
    }
}

impl Default for FontWeight {
    fn default() -> Self {
        Self::Regular
    }
}
