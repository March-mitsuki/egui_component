//! Version strings from the library manifest, embedded for native and Web builds.
use std::sync::LazyLock;

const MANIFEST: &str = include_str!("../../Cargo.toml");

pub struct Versions {
    pub component: &'static str,
    pub installation: String,
    pub lucide_installation: String,
}

pub static VERSIONS: LazyLock<Versions> = LazyLock::new(|| {
    let component = manifest_value("package", "version");
    let egui_dependency = manifest_value("dependencies", "egui");
    let egui = egui_dependency
        .strip_prefix('{')
        .and_then(|table| {
            table.split(',').find_map(|field| {
                let (key, value) = field.split_once('=')?;
                (key.trim() == "version").then(|| value.trim().trim_matches('"'))
            })
        })
        .expect("egui dependency must have a version in the library manifest");
    Versions {
        component,
        installation: format!(
            "[dependencies]\negui = \"{egui}\"\negui_component = \"{component}\""
        ),
        lucide_installation: format!(
            "egui_component = {{ version = \"{component}\", features = [\"lucide_icon\"] }}"
        ),
    }
});

// Read the single-line values used by the library manifest. This is intentionally
// limited to its current layout rather than implementing a general TOML parser.
fn manifest_value(section: &str, key: &str) -> &'static str {
    let mut active = false;
    for line in MANIFEST.lines().map(str::trim) {
        if line.starts_with('[') {
            active = line == format!("[{section}]");
        } else if active {
            if let Some((name, value)) = line.split_once('=') {
                if name.trim() == key {
                    return value.trim().trim_matches('"');
                }
            }
        }
    }
    panic!("missing [{section}] {key} in the library manifest");
}
