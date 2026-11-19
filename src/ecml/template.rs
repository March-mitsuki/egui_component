use super::{
    Error, ValueKind,
    parser::Element,
    runtime::{Action, Binding, List},
};
use crate::theme::{ButtonSize, ButtonVariant, TextEditSize, TextEditVariant};
use std::collections::{HashMap, HashSet};

/// A resolved name: a root registration, or a registration of the list of the loop
/// at `depth` in the enclosing loops (0 is the outermost).
#[derive(Clone, Copy)]
pub(super) enum Ref {
    Root(usize),
    Row { depth: usize, index: usize },
}
pub(super) enum BoolExpr {
    Literal(bool),
    Binding(Ref, bool),
}
pub(super) enum TextPart {
    Literal(String),
    Binding(Ref),
}
pub(super) struct Node {
    pub key: String,
    pub explicit_id: Option<String>,
    pub kind: Kind,
}
pub(super) enum Kind {
    /// Repeats `body` once per list row; `line`/`column` locate the e-for attribute.
    For {
        /// A root list, or a child list (`Row::index`) of the row at `Row::depth`.
        list: Ref,
        body: Box<Node>,
        line: usize,
        column: usize,
    },
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
        click: Option<Ref>,
    },
    TextEdit {
        model: Ref,
        size: TextEditSize,
        variant: TextEditVariant,
        width: Option<f32>,
        placeholder: Vec<TextPart>,
        disabled: BoolExpr,
    },
    Switch {
        model: Ref,
        size: String,
        disabled: BoolExpr,
    },
    Checkbox {
        model: Ref,
        label: Vec<TextPart>,
        size: String,
        disabled: BoolExpr,
    },
}
type Names = HashMap<String, (usize, ValueKind, bool)>;
struct ListNames {
    index: usize,
    bindings: Names,
    actions: HashMap<String, usize>,
    lists: HashMap<String, ListNames>,
}
impl ListNames {
    fn new<S>(index: usize, list: &List<S>) -> Result<Self, String> {
        let mut names = Self {
            index,
            bindings: HashMap::new(),
            actions: HashMap::new(),
            lists: HashMap::new(),
        };
        for (i, b) in list.bindings.iter().enumerate() {
            if !identifier(&b.name)
                || names
                    .bindings
                    .insert(b.name.clone(), (i, b.kind, b.set.is_some()))
                    .is_some()
            {
                return Err(format!(
                    "invalid or duplicate row binding '{}' in list '{}'",
                    b.name, list.name
                ));
            }
        }
        for (i, a) in list.actions.iter().enumerate() {
            if !identifier(&a.name) || names.actions.insert(a.name.clone(), i).is_some() {
                return Err(format!(
                    "invalid or duplicate row handler '{}' in list '{}'",
                    a.name, list.name
                ));
            }
        }
        names.lists = lists(&list.lists, &names.bindings, &names.actions)?;
        Ok(names)
    }
}
/// Lists at one level; their names share a namespace with that level's bindings and handlers.
fn lists<S>(
    lists: &[List<S>],
    bindings: &Names,
    actions: &HashMap<String, usize>,
) -> Result<HashMap<String, ListNames>, String> {
    let mut result = HashMap::new();
    for (index, list) in lists.iter().enumerate() {
        if !identifier(&list.name) {
            return Err(format!("invalid list name '{}'", list.name));
        }
        if bindings.contains_key(&list.name)
            || actions.contains_key(&list.name)
            || result.contains_key(&list.name)
        {
            return Err(format!(
                "list '{}' collides with another registration",
                list.name
            ));
        }
        result.insert(list.name.clone(), ListNames::new(index, list)?);
    }
    Ok(result)
}
/// One enclosing loop; scopes form a chain from the outermost loop inwards.
struct Scope<'a> {
    alias: String,
    names: &'a ListNames,
}
pub(super) struct Registry {
    bindings: Names,
    actions: HashMap<String, usize>,
    lists: HashMap<String, ListNames>,
}
impl Registry {
    pub fn new<S>(
        bindings: &[Binding<S>],
        actions: &[Action<S>],
        lists: &[List<S>],
    ) -> Result<Self, String> {
        let mut result = Self {
            bindings: HashMap::new(),
            actions: HashMap::new(),
            lists: HashMap::new(),
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
        result.lists = self::lists(lists, &result.bindings, &result.actions)?;
        Ok(result)
    }
    pub fn compile(&self, root: Element, source: &str, file: &str) -> Result<Node, Error> {
        Compiler {
            registry: self,
            source,
            file,
            ids: HashSet::new(),
            scopes: vec![],
        }
        .node(root, "0".into(), true)
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
    scopes: Vec<Scope<'a>>,
}
impl<'r> Compiler<'r> {
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
    ) -> Result<Ref, Error> {
        let (r, actual, can_write) = if let Some((alias, field)) = name.split_once('.') {
            let (depth, list) = self.row_list(e, attr, alias)?;
            let Some(&(index, kind, can_write)) = list.bindings.get(field) else {
                return Err(self.error(
                    e,
                    attr,
                    format!("unknown row binding '{field}' for '{alias}'"),
                ));
            };
            (Ref::Row { depth, index }, kind, can_write)
        } else {
            let Some(&(index, kind, can_write)) = self.registry.bindings.get(name) else {
                return Err(self.error(e, attr, format!("unknown binding '{name}'")));
            };
            (Ref::Root(index), kind, can_write)
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
        Ok(r)
    }
    /// The depth and list of the loop whose alias is `alias`; aliases exist only inside
    /// their loop, including inside loops nested in it.
    fn row_list(
        &self,
        e: &Element,
        attr: &str,
        alias: &str,
    ) -> Result<(usize, &'r ListNames), Error> {
        self.scopes
            .iter()
            .rposition(|scope| scope.alias == alias)
            .map(|depth| (depth, self.scopes[depth].names))
            .ok_or_else(|| self.error(e, attr, format!("'{alias}' is not a loop alias in scope")))
    }
    fn action(&self, e: &Element, attr: &str, name: &str) -> Result<Ref, Error> {
        let found = if let Some((alias, field)) = name.split_once('.') {
            let (depth, list) = self.row_list(e, attr, alias)?;
            list.actions
                .get(field)
                .map(|&index| Ref::Row { depth, index })
        } else {
            self.registry.actions.get(name).map(|i| Ref::Root(*i))
        };
        found.ok_or_else(|| self.error(e, attr, format!("unknown handler '{name}'")))
    }
    /// Compiles `<x e-for="alias in list">` as a loop whose body is the element itself.
    /// `list` is a root list, or `outer.list` for a child list of an enclosing row.
    fn repeat(&mut self, mut e: Element, path: String, root: bool) -> Result<Node, Error> {
        if root {
            return Err(self.error(&e, "e-for", "e-for is not allowed on the root element"));
        }
        let value = self.attr(&e, "e-for", "");
        let parts: Vec<_> = value.split_whitespace().collect();
        let [alias, "in", list] = parts[..] else {
            return Err(self.error(
                &e,
                "e-for",
                "e-for must be written as \"<alias> in <list>\"",
            ));
        };
        let (outer, field) = match list.split_once('.') {
            Some((outer, field)) => (Some(outer), field),
            None => (None, list),
        };
        if !identifier(alias) || !identifier(field) || outer.is_some_and(|o| !identifier(o)) {
            return Err(self.error(&e, "e-for", "e-for alias and list must be plain names"));
        }
        let registry: &'r Registry = self.registry;
        let (list, names) = match outer {
            None => {
                let Some(names) = registry.lists.get(field) else {
                    return Err(self.error(&e, "e-for", format!("unknown list '{field}'")));
                };
                (Ref::Root(names.index), names)
            }
            Some(outer) => {
                let (depth, parent) = self.row_list(&e, "e-for", outer)?;
                let Some(names) = parent.lists.get(field) else {
                    return Err(self.error(
                        &e,
                        "e-for",
                        format!("unknown row list '{field}' for '{outer}'"),
                    ));
                };
                (
                    Ref::Row {
                        depth,
                        index: names.index,
                    },
                    names,
                )
            }
        };
        if registry.bindings.contains_key(alias)
            || registry.actions.contains_key(alias)
            || registry.lists.contains_key(alias)
            || self.scopes.iter().any(|scope| scope.alias == alias)
        {
            return Err(self.error(
                &e,
                "e-for",
                format!("loop alias '{alias}' collides with a registered name or outer alias"),
            ));
        }
        let at = self.error(&e, "e-for", "");
        let alias = alias.to_owned();
        e.attrs.remove("e-for");
        self.scopes.push(Scope { alias, names });
        let body = self.node(e, path, false);
        self.scopes.pop();
        let body = body?;
        Ok(Node {
            key: body.key.clone(),
            explicit_id: None,
            kind: Kind::For {
                list,
                body: Box::new(body),
                line: at.line,
                column: at.column,
            },
        })
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
    fn node(&mut self, e: Element, path: String, root: bool) -> Result<Node, Error> {
        if e.attrs.contains_key("e-for") {
            return self.repeat(e, path, root);
        }
        let allowed: &[&str] = match e.tag.as_str() {
            "layout" => &["direction", "cross_align", "item_spacing.x", "item_spacing.y"],
            "text" => &["size"],
            "button" => &["size", "variant", "width", "disabled", ":disabled", "@click"],
            "text_edit" => &["size", "variant", "width", "placeholder", "disabled", ":disabled", "e-model"],
            "switch" | "checkbox" => &["size", "disabled", ":disabled", "e-model"],
            _ => return Err(self.error(&e, "", format!("unknown component '{}'; prototype supports built-in layout and components text, button, text_edit, switch, checkbox (no aliases)", e.tag))),
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
        if !matches!(e.tag.as_str(), "text" | "button" | "checkbox") && !e.text.trim().is_empty() {
            return Err(self.error(
                &e,
                "",
                "text content is only allowed inside text, button and checkbox",
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
                    .map(|(i, child)| self.node(child, format!("{path}/{i}"), false))
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
                    .map(|a| self.action(&e, "@click", a.value.trim()))
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
            "checkbox" => Kind::Checkbox {
                model: self.binding(
                    &e,
                    "e-model",
                    self.attr(&e, "e-model", ""),
                    Some(ValueKind::Bool),
                    true,
                )?,
                label: self.text(&e, "", e.text.trim())?,
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
