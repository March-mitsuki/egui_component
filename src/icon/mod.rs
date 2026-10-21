use std::sync::LazyLock;

/// Optional Lucide icons, embedded as white SVGs for egui tinting.
#[cfg(feature = "lucide_icon")]
pub mod lucide;

pub struct Icons {
    /// Multicolor brand marks: render with a white tint to preserve their colors.
    pub logo_light: egui::ImageSource<'static>,
    pub logo_dark: egui::ImageSource<'static>,
    pub no_img: egui::ImageSource<'static>,

    pub play: egui::ImageSource<'static>,
    pub pause: egui::ImageSource<'static>,
    pub stop: egui::ImageSource<'static>,
    pub volume1: egui::ImageSource<'static>,

    pub add: egui::ImageSource<'static>,
    pub language: egui::ImageSource<'static>,
    pub trash: egui::ImageSource<'static>,
    pub close: egui::ImageSource<'static>,
    pub arrow_down: egui::ImageSource<'static>,
    pub lock: egui::ImageSource<'static>,
    pub lock_open: egui::ImageSource<'static>,

    pub eye: egui::ImageSource<'static>,
    pub eye_slash: egui::ImageSource<'static>,
}

pub static ICONS: LazyLock<Icons> = LazyLock::new(|| Icons {
    logo_light: egui::include_image!(
        "../../assets/icons/egui_component-icon-256-light-transparent.svg"
    ),
    logo_dark: egui::include_image!(
        "../../assets/icons/egui_component-icon-256-dark-transparent.svg"
    ),
    no_img: egui::include_image!("../../assets/icons/no_img.svg"),

    play: egui::include_image!("../../assets/icons/play.svg"),
    pause: egui::include_image!("../../assets/icons/pause.svg"),
    stop: egui::include_image!("../../assets/icons/stop.svg"),
    volume1: egui::include_image!("../../assets/icons/volume1.svg"),

    add: egui::include_image!("../../assets/icons/add.svg"),
    language: egui::include_image!("../../assets/icons/language.svg"),
    trash: egui::include_image!("../../assets/icons/trash.svg"),
    close: egui::include_image!("../../assets/icons/close.svg"),
    arrow_down: egui::include_image!("../../assets/icons/arrow_down.svg"),
    lock: egui::include_image!("../../assets/icons/lock.svg"),
    lock_open: egui::include_image!("../../assets/icons/lock_open.svg"),

    eye: egui::include_image!("../../assets/icons/eye.svg"),
    eye_slash: egui::include_image!("../../assets/icons/eye_slash.svg"),
});
