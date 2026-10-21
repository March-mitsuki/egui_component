use crate::{NavigationError, validate_path};
use percent_encoding::{NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};
use std::collections::BTreeMap;

/// Ordered decoded search pairs, including duplicate keys and empty values.
/// Parsing follows application/x-www-form-urlencoded (`+` means space).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchParams(Vec<(String, String)>);

impl SearchParams {
    pub fn new() -> Self {
        Self::default()
    }
    /// Accepts a query with or without its leading `?`.
    pub fn parse(query: &str) -> Self {
        Self(
            form_urlencoded::parse(query.strip_prefix('?').unwrap_or(query).as_bytes())
                .into_owned()
                .collect(),
        )
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
    pub fn get_all<'a>(&'a self, key: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        self.0
            .iter()
            .filter(move |(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn append(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.0.push((key.into(), value.into()));
    }
    /// Replace the first value and remove other values for this key.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        if let Some(index) = self.0.iter().position(|(k, _)| k == &key) {
            self.0[index].1 = value;
            let mut position = 0;
            self.0.retain(|(k, _)| {
                let keep = k != &key || position == index;
                position += 1;
                keep
            });
        } else {
            self.0.push((key, value));
        }
    }
    pub fn remove(&mut self, key: &str) {
        self.0.retain(|(k, _)| k != key);
    }
}
impl std::fmt::Display for SearchParams {
    /// Encoded query without a leading `?`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        serializer.extend_pairs(self.iter());
        f.write_str(&serializer.finish())
    }
}

/// Named, decoded path captures. Encoded slashes stay inside one parameter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathParams(BTreeMap<String, String>);
impl PathParams {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0.get(name).map(String::as_str)
    }
}

#[derive(Clone, Debug)]
enum Segment {
    Literal(String),
    Param(String),
}

/// Exact segment pattern, e.g. `/components/:name`. No wildcards or optional
/// segments. Trailing slashes are significant; parameter names must be unique.
#[derive(Clone, Debug)]
pub struct PathPattern {
    segments: Vec<Segment>,
}
impl PathPattern {
    pub fn new(pattern: &str) -> Result<Self, NavigationError> {
        validate_path(pattern)?;
        let mut names = std::collections::BTreeSet::new();
        let mut segments = Vec::new();
        for segment in pattern[1..].split('/') {
            if let Some(name) = segment.strip_prefix(':') {
                if name.is_empty()
                    || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    || !names.insert(name.to_owned())
                {
                    return Err(NavigationError(
                        "Path parameter names must be nonempty, unique ASCII identifiers".into(),
                    ));
                }
                segments.push(Segment::Param(name.into()));
            } else {
                segments.push(Segment::Literal(decode(segment).ok_or_else(|| {
                    NavigationError("Invalid encoded pattern segment".into())
                })?));
            }
        }
        Ok(Self { segments })
    }
    pub fn match_path(&self, path: &str) -> Option<PathParams> {
        validate_path(path).ok()?;
        let parts: Vec<_> = path[1..].split('/').collect();
        if parts.len() != self.segments.len() {
            return None;
        }
        let mut params = BTreeMap::new();
        for (segment, part) in self.segments.iter().zip(parts) {
            let value = decode(part)?;
            if value == "." || value == ".." {
                return None;
            }
            match segment {
                Segment::Literal(literal) if literal != &value => return None,
                Segment::Param(_) if value.is_empty() => return None,
                Segment::Param(name) => {
                    params.insert(name.clone(), value);
                }
                _ => {}
            }
        }
        Some(PathParams(params))
    }
    /// Generate a pathname, percent-encoding each supplied decoded parameter.
    /// Missing, unused or duplicate parameters are errors.
    pub fn build(&self, params: &[(&str, &str)]) -> Result<String, NavigationError> {
        let provided: BTreeMap<_, _> = params.iter().copied().collect();
        let count = self
            .segments
            .iter()
            .filter(|s| matches!(s, Segment::Param(_)))
            .count();
        if provided.len() != params.len() || provided.len() != count {
            return Err(NavigationError(
                "Path parameters do not match the pattern".into(),
            ));
        }
        let mut parts = Vec::new();
        for segment in &self.segments {
            let value = match segment {
                Segment::Literal(value) => value.as_str(),
                Segment::Param(name) => *provided
                    .get(name.as_str())
                    .filter(|v| !v.is_empty())
                    .ok_or_else(|| NavigationError(format!("Missing path parameter: {name}")))?,
            };
            // Browsers normalize dot segments even when percent-encoded.
            if value == "." || value == ".." {
                return Err(NavigationError(
                    "Dot path segments are not supported".into(),
                ));
            }
            parts.push(utf8_percent_encode(value, NON_ALPHANUMERIC).to_string());
        }
        Ok(format!("/{}", parts.join("/")))
    }
}

fn decode(segment: &str) -> Option<String> {
    let bytes = segment.as_bytes();
    for (i, byte) in bytes.iter().enumerate() {
        if *byte == b'%'
            && !(bytes.get(i + 1)?.is_ascii_hexdigit() && bytes.get(i + 2)?.is_ascii_hexdigit())
        {
            return None;
        }
    }
    percent_decode_str(segment)
        .decode_utf8()
        .ok()
        .map(|s| s.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn query_round_trip_and_mutations() {
        let mut p =
            SearchParams::parse("?tag=a&empty=&tag=b&q=%E4%BD%A0%E5%A5%BD+world&plus=%2B&flag");
        assert_eq!(p.get("q"), Some("你好 world"));
        assert_eq!(p.get("plus"), Some("+"));
        assert_eq!(p.get_all("tag").collect::<Vec<_>>(), ["a", "b"]);
        assert_eq!(p.get("flag"), Some(""));
        assert_eq!(SearchParams::parse(&p.to_string()), p);
        p.set("tag", "c&d");
        assert_eq!(p.get_all("tag").collect::<Vec<_>>(), ["c&d"]);
        p.append("tag", "e");
        assert_eq!(p.get_all("tag").count(), 2);
        p.remove("tag");
        assert_eq!(p.get("tag"), None);
        assert_eq!(SearchParams::parse(&p.to_string()), p);
    }
    #[test]
    fn typed_path_values_round_trip() {
        let p = PathPattern::new("/components/:name").unwrap();
        for value in ["button", "你好 world", "a/b?c#d+%", "a_b-c"] {
            let path = p.build(&[("name", value)]).unwrap();
            assert_eq!(p.match_path(&path).unwrap().get("name"), Some(value));
        }
        for path in [
            "/components",
            "/components/",
            "/components/a/b",
            "/components/%FF",
            "/components/%ZZ",
            "/components/%",
        ] {
            assert!(p.match_path(path).is_none());
        }
        assert!(p.build(&[]).is_err());
        assert!(p.build(&[("wrong", "x")]).is_err());
        assert!(p.build(&[("name", "..")]).is_err());
        assert!(PathPattern::new("/:id/:id").is_err());
        assert!(PathPattern::new("/:").is_err());
    }
}
