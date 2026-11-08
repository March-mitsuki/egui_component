use std::sync::LazyLock;

/// Optional Lucide icons, embedded as white SVGs for egui tinting.
#[cfg(feature = "lucide_icon")]
pub mod lucide;

/// Built-in icons for component behavior and status feedback.
/// Enable `lucide_icon` for general-purpose application icons.
pub struct Icons {
    pub close: egui::ImageSource<'static>,
    pub arrow_down: egui::ImageSource<'static>,
    pub eye: egui::ImageSource<'static>,
    pub eye_slash: egui::ImageSource<'static>,

    pub success: egui::ImageSource<'static>,
    pub warning: egui::ImageSource<'static>,
    pub error: egui::ImageSource<'static>,
}

pub static ICONS: LazyLock<Icons> = LazyLock::new(|| Icons {
    close: egui::include_image!("../../assets/icons/close.svg"),
    arrow_down: egui::include_image!("../../assets/icons/arrow_down.svg"),
    eye: egui::include_image!("../../assets/icons/eye.svg"),
    eye_slash: egui::include_image!("../../assets/icons/eye_slash.svg"),

    success: egui::include_image!("../../assets/icons/success.svg"),
    warning: egui::include_image!("../../assets/icons/warning.svg"),
    error: egui::include_image!("../../assets/icons/error.svg"),
});
