//! Deliberately small markup grammar; no HTML recovery or script evaluation.
use super::Error;
use std::collections::BTreeMap;

pub(super) const MAX_BYTES: usize = 256 * 1024;

#[derive(Clone, Debug)]
pub(super) struct Attribute {
    pub value: String,
    pub offset: usize,
}

#[derive(Clone, Debug)]
pub(super) struct Element {
    pub tag: String,
    pub attrs: BTreeMap<String, Attribute>,
    pub children: Vec<Element>,
    pub text: String,
    pub offset: usize,
}

pub(super) fn parse(source: &str, file: &str) -> Result<Element, Error> {
    let mut p = Parser {
        source,
        file,
        pos: 0,
    };
    if source.len() > MAX_BYTES {
        return Err(p.error("template exceeds 256 KiB"));
    }
    p.space();
    let root = p.element(0)?;
    p.space();
    if p.pos != source.len() {
        return Err(p.error("expected exactly one root element"));
    }
    Ok(root)
}

struct Parser<'a> {
    source: &'a str,
    file: &'a str,
    pos: usize,
}

impl Parser<'_> {
    fn error(&self, message: impl Into<String>) -> Error {
        Error::at(self.file, self.source, self.pos, message)
    }
    fn rest(&self) -> &str {
        &self.source[self.pos..]
    }
    fn space(&mut self) {
        while let Some(c) = self.rest().chars().next().filter(|c| c.is_whitespace()) {
            self.pos += c.len_utf8();
        }
    }
    fn take(&mut self, token: &str) -> bool {
        if self.rest().starts_with(token) {
            self.pos += token.len();
            true
        } else {
            false
        }
    }
    fn name(&mut self, attribute: bool) -> Result<String, Error> {
        let start = self.pos;
        if attribute && (self.rest().starts_with(':') || self.rest().starts_with('@')) {
            self.pos += 1;
        }
        loop {
            if !self
                .rest()
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            {
                return Err(self.error("expected a name segment starting with a letter or '_'"));
            }
            self.pos += 1;
            while let Some(c) = self.rest().chars().next() {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            if !attribute || !self.take(".") {
                break;
            }
        }
        Ok(self.source[start..self.pos].into())
    }
    fn element(&mut self, depth: usize) -> Result<Element, Error> {
        if depth >= 64 {
            return Err(self.error("nesting exceeds 64 levels"));
        }
        let offset = self.pos;
        if !self.take("<") {
            return Err(self.error("expected '<'"));
        }
        let tag = self.name(false)?;
        let mut attrs = BTreeMap::new();
        loop {
            let before = self.pos;
            self.space();
            if self.take("/>") {
                return Ok(Element {
                    tag,
                    attrs,
                    children: vec![],
                    text: String::new(),
                    offset,
                });
            }
            if self.take(">") {
                break;
            }
            if self.pos == before {
                return Err(self.error("expected whitespace before attribute"));
            }
            let offset = self.pos;
            let name = self.name(true)?;
            let after_name = self.pos;
            self.space();
            if name == "e-else" {
                // The only bare attribute: it ends a conditional chain and has no value.
                if self.rest().starts_with('=') {
                    return Err(self.error("e-else takes no value"));
                }
                self.pos = after_name;
                if attrs
                    .insert(
                        name,
                        Attribute {
                            value: String::new(),
                            offset,
                        },
                    )
                    .is_some()
                {
                    return Err(self.error("duplicate attribute 'e-else'"));
                }
                continue;
            }
            if !self.take("=") {
                return Err(self.error("expected '='"));
            }
            self.space();
            let quote = if self.take("\"") {
                '"'
            } else if self.take("'") {
                '\''
            } else {
                return Err(self.error("attribute values must be quoted"));
            };
            let start = self.pos;
            let Some(end) = self.rest().find(quote) else {
                return Err(self.error("unterminated attribute"));
            };
            self.pos += end;
            let value = decode(&self.source[start..self.pos]).map_err(|m| self.error(m))?;
            self.pos += 1;
            if attrs
                .insert(name.clone(), Attribute { value, offset })
                .is_some()
            {
                return Err(self.error(format!("duplicate attribute '{name}'")));
            }
        }
        let mut children = vec![];
        let mut text = String::new();
        loop {
            if self.take("</") {
                let closing = self.name(false)?;
                self.space();
                if closing != tag || !self.take(">") {
                    return Err(self.error(format!("expected </{tag}>")));
                }
                break;
            }
            if self.rest().is_empty() {
                return Err(self.error(format!("missing </{tag}>")));
            }
            if self.take("<!--") {
                let Some(end) = self.rest().find("-->") else {
                    return Err(self.error("unterminated comment"));
                };
                self.pos += end + 3;
            } else if self.rest().starts_with('<') {
                children.push(self.element(depth + 1)?);
            } else {
                let count = self.rest().find('<').unwrap_or(self.rest().len());
                text.push_str(&decode(&self.rest()[..count]).map_err(|m| self.error(m))?);
                self.pos += count;
            }
        }
        Ok(Element {
            tag,
            attrs,
            children,
            text,
            offset,
        })
    }
}

fn decode(text: &str) -> Result<String, &'static str> {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('&') {
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        let Some(end) = rest.find(';') else {
            return Err("unterminated entity; use &amp; for '&'");
        };
        out.push(match &rest[..=end] {
            "&amp;" => '&',
            "&lt;" => '<',
            "&gt;" => '>',
            "&quot;" => '"',
            "&apos;" => '\'',
            _ => return Err("unsupported entity (supported: amp, lt, gt, quot, apos)"),
        });
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}
