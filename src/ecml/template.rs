use super::{
    Error, ValueKind,
    parser::Element,
    runtime::{Action, Binding},
};
use crate::theme::{ButtonSize, ButtonVariant, TextEditSize, TextEditVariant};
use std::collections::{HashMap, HashSet};

pub(super) enum BoolExpr {
    Literal(bool),
    Binding(usize, bool),
}
pub(super) enum TextPart {
    Literal(String),
    Binding(usize),
}
pub(super) struct Node {
    pub key: String,
    pub explicit_id: Option<String>,
    pub kind: Kind,
}
pub(super) enum Kind {
    Layout {
        horizontal: bool,
        cross_align: egui::Align,
        item_spacing_x: Option<f32>,
        item_spacing_y: Option<f32>,
        children: Vec<Node>,
    },
    Text {
        text: Vec<TextPart>,
        heading: bool,
    },
    Button {
        text: Vec<TextPart>,
        size: ButtonSize,
        variant: ButtonVariant,
        width: Option<f32>,
        disabled: BoolExpr,
        click: Option<usize>,
    },
    TextEdit {
        model: usize,
        size: TextEditSize,
        variant: TextEditVariant,
        width: Option<f32>,
        placeholder: Vec<TextPart>,
        disabled: BoolExpr,
    },
    Switch {
        model: usize,
        size: String,
        disabled: BoolExpr,
    },
}
pub(super) struct Registry {
    bindings: HashMap<String, (usize, ValueKind, bool)>,
    actions: HashMap<String, usize>,
}
impl Registry {
    pub fn new<S>(bindings: &[Binding<S>], actions: &[Action<S>]) -> Result<Self, String> {
        let mut result = Self {
            bindings: HashMap::new(),
            actions: HashMap::new(),
        };
        for (i, b) in bindings.iter().enumerate() {
            if !identifier(&b.name) {
                return Err(format!("invalid binding name '{}'", b.name));
            }
            if result
                .bindings
                .insert(b.name.clone(), (i, b.kind, b.set.is_some()))
                .is_some()
            {
                return Err(format!("duplicate binding '{}'", b.name));
            }
        }
        for (i, a) in actions.iter().enumerate() {
            if !identifier(&a.name) {
                return Err(format!("invalid handler name '{}'", a.name));
            }
            if result.actions.insert(a.name.clone(), i).is_some() {
                return Err(format!("duplicate handler '{}'", a.name));
            }
        }
        Ok(result)
    }
    pub fn compile(&self, root: Element, source: &str, file: &str) -> Result<Node, Error> {
        Compiler {
            registry: self,
            source,
            file,
            ids: HashSet::new(),
        }
        .node(root, "0".into())
    }
}
fn identifier(s: &str) -> bool {
    let mut chars = s.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
struct Compiler<'a> {
    registry: &'a Registry,
    source: &'a str,
    file: &'a str,
    ids: HashSet<String>,
}
impl Compiler<'_> {
    fn error(&self, element: &Element, attr: &str, message: impl Into<String>) -> Error {
        let offset = element.attrs.get(attr).map_or(element.offset, |a| a.offset);
        Error::at(self.file, self.source, offset, message)
    }
    fn attr<'a>(&self, e: &'a Element, name: &str, default: &'a str) -> &'a str {
        e.attrs.get(name).map_or(default, |a| a.value.as_str())
    }
    fn choice<'a>(
        &self,
        e: &'a Element,
        name: &str,
        default: &'a str,
        choices: &[&str],
    ) -> Result<&'a str, Error> {
        let value = self.attr(e, name, default);
        if choices.contains(&value) {
            Ok(value)
        } else {
            Err(self.error(
                e,
                name,
                format!("'{name}' must be one of {}", choices.join(", ")),
            ))
        }
    }
    fn number(&self, e: &Element, name: &str) -> Result<Option<f32>, Error> {
        e.attrs
            .get(name)
            .map(|a| {
                a.value
                    .parse::<f32>()
                    .ok()
                    .filter(|n| n.is_finite() && *n >= 0.0)
                    .ok_or_else(|| {
                        self.error(
                            e,
                            name,
                            format!("'{name}' must be a finite nonnegative number"),
                        )
                    })
            })
            .transpose()
    }
    fn binding(
        &self,
        e: &Element,
        attr: &str,
        name: &str,
        kind: Option<ValueKind>,
        writable: bool,
    ) -> Result<usize, Error> {
        let Some(&(index, actual, can_write)) = self.registry.bindings.get(name) else {
            return Err(self.error(e, attr, format!("unknown binding '{name}'")));
        };
        if kind.is_some_and(|k| k != actual) {
            return Err(self.error(
                e,
                attr,
                format!(
                    "binding '{name}' has type {actual:?}, expected {:?}",
                    kind.unwrap()
                ),
            ));
        }
        if writable && !can_write {
            return Err(self.error(
                e,
                attr,
                format!("binding '{name}' is read-only; e-model requires .model()"),
            ));
        }
        Ok(index)
    }
    fn boolean(&self, e: &Element) -> Result<BoolExpr, Error> {
        if e.attrs.contains_key("disabled") && e.attrs.contains_key(":disabled") {
            return Err(self.error(e, ":disabled", "cannot combine disabled and :disabled"));
        }
        if let Some(a) = e.attrs.get(":disabled") {
            let expr = a.value.trim();
            let (name, negate) = expr
                .strip_prefix('!')
                .map_or((expr, false), |n| (n.trim(), true));
            Ok(BoolExpr::Binding(
                self.binding(e, ":disabled", name, Some(ValueKind::Bool), false)?,
                negate,
            ))
        } else {
            Ok(BoolExpr::Literal(
                self.choice(e, "disabled", "false", &["true", "false"])? == "true",
            ))
        }
    }
    fn text(&self, e: &Element, attr: &str, raw: &str) -> Result<Vec<TextPart>, Error> {
        let mut parts = vec![];
        let mut remaining = raw;
        while let Some(start) = remaining.find("{{") {
            parts.push(TextPart::Literal(remaining[..start].into()));
            remaining = &remaining[start + 2..];
            let Some(end) = remaining.find("}}") else {
                return Err(self.error(e, attr, "unclosed text interpolation"));
            };
            let name = remaining[..end].trim();
            parts.push(TextPart::Binding(self.binding(e, attr, name, None, false)?));
            remaining = &remaining[end + 2..];
        }
        parts.push(TextPart::Literal(remaining.into()));
        Ok(parts)
    }
    fn node(&mut self, e: Element, path: String) -> Result<Node, Error> {
        let allowed: &[&str] = match e.tag.as_str() {
            "layout" => &["direction", "cross_align", "item_spacing.x", "item_spacing.y"],
            "text" => &["size"],
            "button" => &["size", "variant", "width", "disabled", ":disabled", "@click"],
            "text_edit" => &["size", "variant", "width", "placeholder", "disabled", ":disabled", "e-model"],
            "switch" => &["size", "disabled", ":disabled", "e-model"],
            _ => return Err(self.error(&e, "", format!("unknown component '{}'; prototype supports built-in layout and components text, button, text_edit, switch (no aliases)", e.tag))),
        };
        for name in e.attrs.keys() {
            if name != "id" && !allowed.contains(&name.as_str()) {
                return Err(self.error(
                    &e,
                    name,
                    format!("unknown attribute '{name}' on <{}>", e.tag),
                ));
            }
        }
        let explicit_id = e.attrs.get("id").map(|a| a.value.clone());
        if let Some(id) = &explicit_id
            && (id.trim().is_empty() || !self.ids.insert(id.clone()))
        {
            return Err(self.error(&e, "id", "id must be nonempty and unique within the View"));
        }
        // Include tag so a type change never inherits another widget's editing state.
        let key = match &explicit_id {
            Some(id) => format!("id:{id}:{}", e.tag),
            None => format!("path:{path}:{}", e.tag),
        };
        let layout = e.tag == "layout";
        if !layout && !e.children.is_empty() {
            return Err(self.error(&e, "", "only layout accepts child components"));
        }
        if !matches!(e.tag.as_str(), "text" | "button") && !e.text.trim().is_empty() {
            return Err(self.error(
                &e,
                "",
                "text content is only allowed inside text and button",
            ));
        }
        let kind = match e.tag.as_str() {
            "layout" => {
                let horizontal =
                    self.choice(&e, "direction", "top_down", &["top_down", "left_to_right"])?
                        == "left_to_right";
                let cross_align =
                    match self.choice(&e, "cross_align", "min", &["min", "center", "max"])? {
                        "center" => egui::Align::Center,
                        "max" => egui::Align::Max,
                        _ => egui::Align::Min,
                    };
                let item_spacing_x = self.number(&e, "item_spacing.x")?;
                let item_spacing_y = self.number(&e, "item_spacing.y")?;
                let children = e
                    .children
                    .into_iter()
                    .enumerate()
                    .map(|(i, child)| self.node(child, format!("{path}/{i}")))
                    .collect::<Result<_, _>>()?;
                Kind::Layout {
                    horizontal,
                    cross_align,
                    item_spacing_x,
                    item_spacing_y,
                    children,
                }
            }
            "text" => Kind::Text {
                text: self.text(&e, "", e.text.trim())?,
                heading: self.choice(&e, "size", "md", &["md", "heading"])? == "heading",
            },
            "button" => {
                let size = match self.choice(&e, "size", "md", &["xs", "sm", "md", "lg"])? {
                    "xs" => ButtonSize::Xs,
                    "sm" => ButtonSize::Sm,
                    "lg" => ButtonSize::Lg,
                    _ => ButtonSize::Md,
                };
                let variant = match self.choice(
                    &e,
                    "variant",
                    "solid",
                    &["solid", "surface", "outline", "ghost"],
                )? {
                    "surface" => ButtonVariant::Surface,
                    "outline" => ButtonVariant::Outline,
                    "ghost" => ButtonVariant::Ghost,
                    _ => ButtonVariant::Solid,
                };
                let click = e
                    .attrs
                    .get("@click")
                    .map(|a| {
                        self.registry
                            .actions
                            .get(a.value.trim())
                            .copied()
                            .ok_or_else(|| {
                                self.error(&e, "@click", format!("unknown handler '{}'", a.value))
                            })
                    })
                    .transpose()?;
                Kind::Button {
                    text: self.text(&e, "", e.text.trim())?,
                    size,
                    variant,
                    click,
                    width: self.number(&e, "width")?,
                    disabled: self.boolean(&e)?,
                }
            }
            "text_edit" => {
                let size = match self.choice(&e, "size", "md", &["xs", "sm", "md", "lg"])? {
                    "xs" => TextEditSize::Xs,
                    "sm" => TextEditSize::Sm,
                    "lg" => TextEditSize::Lg,
                    _ => TextEditSize::Md,
                };
                let variant = match self.choice(
                    &e,
                    "variant",
                    "outline",
                    &["subtle", "outline", "flushed"],
                )? {
                    "subtle" => TextEditVariant::Subtle,
                    "flushed" => TextEditVariant::Flushed,
                    _ => TextEditVariant::Outline,
                };
                Kind::TextEdit {
                    model: self.binding(
                        &e,
                        "e-model",
                        self.attr(&e, "e-model", ""),
                        Some(ValueKind::String),
                        true,
                    )?,
                    size,
                    variant,
                    placeholder: self.text(&e, "placeholder", self.attr(&e, "placeholder", ""))?,
                    width: self.number(&e, "width")?,
                    disabled: self.boolean(&e)?,
                }
            }
            "switch" => Kind::Switch {
                model: self.binding(
                    &e,
                    "e-model",
                    self.attr(&e, "e-model", ""),
                    Some(ValueKind::Bool),
                    true,
                )?,
                size: self.choice(&e, "size", "md", &["sm", "md", "lg"])?.into(),
                disabled: self.boolean(&e)?,
            },
            _ => unreachable!(),
        };
        Ok(Node {
            key,
            explicit_id,
            kind,
        })
    }
}
