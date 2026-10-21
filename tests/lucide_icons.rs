#![cfg(feature = "lucide_icon")]

use egui_component::icon::lucide;

#[test]
fn every_icon_rasterizes_as_a_white_mask() {
    assert!(!lucide::NAMES.is_empty());
    assert!(lucide::NAMES.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(lucide::get("not-an-icon").is_none());
    assert!(matches!(lucide::HEART, egui::ImageSource::Bytes { .. }));

    for name in lucide::NAMES {
        let egui::ImageSource::Bytes { bytes, .. } = lucide::get(name).unwrap() else {
            panic!("{name} must be embedded");
        };
        assert!(!String::from_utf8_lossy(bytes.as_ref()).contains("currentColor"));
        let image = egui_extras::image::load_svg_bytes(bytes.as_ref(), &Default::default())
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(image.size, [24, 24], "{name}");
        assert!(image.pixels.iter().any(|pixel| pixel.a() > 0), "{name}");
        assert!(image.pixels.iter().any(|pixel| pixel.a() == 0), "{name}");
        for pixel in &image.pixels {
            let [r, g, b, a] = pixel.to_array();
            assert_eq!([r, g, b], [a, a, a], "{name}: must be premultiplied white");
        }
    }
}
