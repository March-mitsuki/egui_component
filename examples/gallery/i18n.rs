//! Gallery-only translations. Resources are embedded for native and Web builds.
use serde::{
    Deserialize,
    de::{self, MapAccess, Visitor},
};
use std::{collections::BTreeMap, sync::LazyLock};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Locale {
    #[default]
    En,
    Zh,
    Ja,
}

impl Locale {
    pub const ALL: [Self; 3] = [Self::En, Self::Zh, Self::Ja];
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Zh => "zh",
            Self::Ja => "ja",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Zh => "简体中文",
            Self::Ja => "日本語",
        }
    }
    pub fn parse(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|locale| locale.code() == code)
    }
    pub fn text(self, key: &str) -> &str {
        let catalogs = &*CATALOGS;
        catalogs[self.index()]
            .0
            .get(key)
            .or_else(|| catalogs[0].0.get(key))
            .map(String::as_str)
            .unwrap_or_else(|| {
                tracing::warn!(key, "Missing gallery translation");
                key
            })
    }
    pub fn format(self, key: &str, args: &[(&str, &str)]) -> String {
        // Substitute in one pass so braces inside user input remain literal.
        let template = self.text(key);
        let mut result = String::new();
        let mut rest = template;
        while let Some(start) = rest.find('{') {
            let Some(end) = rest[start..].find('}').map(|end| start + end) else {
                break;
            };
            result.push_str(&rest[..start]);
            let name = &rest[start + 1..end];
            if let Some((_, value)) = args.iter().find(|(key, _)| *key == name) {
                result.push_str(value);
            } else {
                result.push_str(&rest[start..=end]);
            }
            rest = &rest[end + 1..];
        }
        result.push_str(rest);
        result
    }
    fn index(self) -> usize {
        match self {
            Self::En => 0,
            Self::Zh => 1,
            Self::Ja => 2,
        }
    }
}

// serde_json's default map handling silently overwrites duplicate keys.
struct Catalog(BTreeMap<String, String>);
impl<'de> Deserialize<'de> for Catalog {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct CatalogVisitor;
        impl<'de> Visitor<'de> for CatalogVisitor {
            type Value = Catalog;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a flat translation object with unique keys")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Catalog, M::Error> {
                let mut entries = BTreeMap::new();
                while let Some((key, value)) = map.next_entry::<String, String>()? {
                    if entries.insert(key.clone(), value).is_some() {
                        return Err(de::Error::custom(format!(
                            "duplicate translation key: {key}"
                        )));
                    }
                }
                Ok(Catalog(entries))
            }
        }
        deserializer.deserialize_map(CatalogVisitor)
    }
}
static CATALOGS: LazyLock<[Catalog; 3]> = LazyLock::new(|| {
    [
        include_str!("locales/en.json"),
        include_str!("locales/zh.json"),
        include_str!("locales/ja.json"),
    ]
    .map(|source| serde_json::from_str(source).expect("valid gallery translation catalog"))
});

#[cfg(test)]
mod tests {
    use super::*;
    fn placeholders(value: &str) -> std::collections::BTreeSet<&str> {
        value
            .split('{')
            .skip(1)
            .map(|part| part.split_once('}').expect("closed placeholder").0)
            .collect()
    }
    #[test]
    fn gallery_source_keys_exist_in_the_catalog() {
        let sources = [
            include_str!("app.rs"),
            include_str!("docs.rs"),
            include_str!("routes.rs"),
            include_str!("main.rs"),
        ];
        for source in sources.into_iter().chain(
            crate::routes::Component::ALL
                .iter()
                .map(|component| component.source()),
        ) {
            for literal in source.split('"').skip(1).step_by(2) {
                if let Some((namespace, name)) = literal.split_once(':') {
                    if !namespace.is_empty()
                        && !name.is_empty()
                        && namespace
                            .chars()
                            .all(|c| c.is_ascii_alphabetic() || c == '.')
                        && name.chars().all(|c| c.is_ascii_alphanumeric())
                    {
                        assert!(
                            CATALOGS[0].0.contains_key(literal),
                            "unknown translation key: {literal}"
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn catalogs_have_identical_keys_and_placeholders() {
        let english = &CATALOGS[0].0;
        for catalog in CATALOGS.iter() {
            assert_eq!(
                english.keys().collect::<Vec<_>>(),
                catalog.0.keys().collect::<Vec<_>>()
            );
            for (key, value) in &catalog.0 {
                assert!(!value.trim().is_empty(), "{key}");
                assert_eq!(placeholders(&english[key]), placeholders(value), "{key}");
            }
        }
    }
    #[test]
    fn duplicate_keys_are_rejected() {
        assert!(
            serde_json::from_str::<Catalog>(r#"{"nav:home":"Home","nav:home":"Other"}"#).is_err()
        );
    }
    #[test]
    fn substitution_does_not_interpret_user_input() {
        assert_eq!(
            Locale::En.format("error:notFound", &[("path", "{path}")]),
            "We couldn't find {path}. Let's get you back to building."
        );
    }
}
