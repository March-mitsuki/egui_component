use super::{
    Error, ValueKind,
    parser::Element,
    runtime::{Action, Binding, List},
};
use crate::theme::{
    self, ButtonSize, ButtonVariant, SetColor, TextEditSize, TextEditVariant, Theme,
};
use egui::Color32;
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
/// An attribute value: a constant, or `cond ? then : otherwise` from a `:` attribute.
pub(super) enum Expr<T> {
    Const(T),
    If(BoolExpr, T, Box<Expr<T>>),
}
/// A `$theme.colors.<scale>.<level>` or `$theme.colors.<bg|fg>.<tone>` color, looked up
/// in the theme being drawn with. Paths mirror the Rust `Theme` fields; nothing is reflected.
#[derive(Clone, Copy)]
pub(super) enum ThemeColor {
    Scale(fn(&Theme) -> &theme::Color, fn(&theme::Color) -> Color32),
    Set(fn(&Theme) -> &SetColor, fn(&SetColor) -> Color32),
}
impl ThemeColor {
    pub fn get(self, theme: &Theme) -> Color32 {
        match self {
            Self::Scale(scale, level) => level(scale(theme)),
            Self::Set(set, tone) => tone(set(theme)),
        }
    }
}
/// A `$theme.colors.<scale>` color scale, as taken by button's `color`.
#[derive(Clone, Copy)]
pub(super) struct ThemeScale(pub fn(&Theme) -> &theme::Color);
const SCALES: &str = "primary, gray, red, green, blue, yellow, orange";
fn scale(name: &str) -> Option<fn(&Theme) -> &theme::Color> {
    Some(match name {
        "primary" => |t| &t.colors.primary,
        "gray" => |t| &t.colors.gray,
        "red" => |t| &t.colors.red,
        "green" => |t| &t.colors.green,
        "blue" => |t| &t.colors.blue,
        "yellow" => |t| &t.colors.yellow,
        "orange" => |t| &t.colors.orange,
        _ => return None,
    })
}
fn level(name: &str) -> Option<fn(&theme::Color) -> Color32> {
    Some(match name {
        "foreground" => |c| c.foreground,
        "l0" => |c| c.l0,
        "l1" => |c| c.l1,
        "l2" => |c| c.l2,
        "l3" => |c| c.l3,
        "l4" => |c| c.l4,
        "l5" => |c| c.l5,
        "l6" => |c| c.l6,
        "l7" => |c| c.l7,
        "l8" => |c| c.l8,
        "l9" => |c| c.l9,
        _ => return None,
    })
}
fn set(name: &str) -> Option<fn(&Theme) -> &SetColor> {
    Some(match name {
        "bg" => |t| &t.colors.bg,
        "fg" => |t| &t.colors.fg,
        _ => return None,
    })
}
fn tone(name: &str) -> Option<fn(&SetColor) -> Color32> {
    Some(match name {
        "base" => |c| c.base,
        "subtle" => |c| c.subtle,
        "accent" => |c| c.accent,
        _ => return None,
    })
}
/// The part of a theme path after `$theme.colors.`, split at dots.
fn theme_path(path: &str) -> Result<Vec<&str>, String> {
    let rest = path.strip_prefix("$theme.colors.").ok_or_else(|| {
        format!("unknown theme value '{path}'; only $theme.colors.… is available")
    })?;
    Ok(rest.split('.').collect())
}
fn theme_color(path: &str) -> Result<ThemeColor, String> {
    let found = match theme_path(path)?[..] {
        [s, l] => scale(s)
            .zip(level(l))
            .map(|(s, l)| ThemeColor::Scale(s, l))
            .or_else(|| set(s).zip(tone(l)).map(|(s, t)| ThemeColor::Set(s, t))),
        _ => None,
    };
    found.ok_or_else(|| {
        format!(
            "unknown theme color '{path}'; use $theme.colors.<scale>.<foreground|l0..l9> with scale one of {SCALES}, or $theme.colors.<bg|fg>.<base|subtle|accent>"
        )
    })
}
fn theme_scale(path: &str) -> Result<ThemeScale, String> {
    match theme_path(path)?[..] {
        [s] => scale(s).map(ThemeScale),
        _ => None,
    }
    .ok_or_else(|| {
        format!(
            "unknown color scale '{path}'; use $theme.colors.<scale> with scale one of {SCALES}"
        )
    })
}
/// One value in an attribute: a word (`ghost` statically, `'ghost'` in a `:` attribute)
/// or a `$theme` path.
#[derive(Clone, Copy)]
enum Leaf<'a> {
    Word(&'a str),
    Theme(&'a str),
}
type LeafFn<'f, T> = &'f dyn Fn(Leaf) -> Result<T, String>;
fn words<T: Copy>(name: &str, options: &[(&str, T)]) -> impl Fn(Leaf) -> Result<T, String> {
    let message = format!(
        "'{name}' must be one of {}",
        options
            .iter()
            .map(|(w, _)| *w)
            .collect::<Vec<_>>()
            .join(", ")
    );
    let options = options.to_vec();
    move |leaf| match leaf {
        Leaf::Word(w) => options
            .iter()
            .find(|(o, _)| *o == w)
            .map(|(_, v)| *v)
            .ok_or_else(|| message.clone()),
        Leaf::Theme(_) => Err(message.clone()),
    }
}
fn color(leaf: Leaf) -> Result<ThemeColor, String> {
    match leaf {
        Leaf::Theme(path) => theme_color(path),
        Leaf::Word(w) => Err(format!(
            "'{w}' is not a color; use a theme path such as $theme.colors.gray.l6"
        )),
    }
}
fn color_scale(leaf: Leaf) -> Result<ThemeScale, String> {
    match leaf {
        Leaf::Theme(path) => theme_scale(path),
        Leaf::Word(w) => Err(format!(
            "'{w}' is not a color scale; use a theme path such as $theme.colors.red"
        )),
    }
}
/// Splits the leading value off `s`: `'word'` or a `$theme` path.
fn leaf(s: &str) -> Option<(Leaf<'_>, &str)> {
    if let Some(rest) = s.strip_prefix('\'') {
        let end = rest.find('\'')?;
        Some((Leaf::Word(&rest[..end]), &rest[end + 1..]))
    } else if let Some(path) = s.strip_prefix('$') {
        let end = path
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.'))
            .map_or(s.len(), |i| i + 1);
        Some((Leaf::Theme(&s[..end]), &s[end..]))
    } else {
        None
    }
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
    /// A conditional chain: draws the first branch whose condition holds (`None` is e-else).
    If {
        branches: Vec<(Option<BoolExpr>, Node)>,
    },
    /// Children drawn directly into the parent Ui, without a layout of their own.
    Fragment { children: Vec<Node> },
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
        heading: Expr<bool>,
        color: Option<Expr<ThemeColor>>,
        underline: BoolExpr,
        strikethrough: BoolExpr,
        /// Handlers for egui's `clicked()` and `double_clicked()`; either makes the text
        /// clickable and not selectable.
        click: Option<Ref>,
        double_click: Option<Ref>,
    },
    Button {
        text: Vec<TextPart>,
        size: Expr<ButtonSize>,
        variant: Expr<ButtonVariant>,
        color: Option<Expr<ThemeScale>>,
        width: Option<f32>,
        disabled: BoolExpr,
        click: Option<Ref>,
    },
    TextEdit {
        model: Ref,
        size: Expr<TextEditSize>,
        variant: Expr<TextEditVariant>,
        width: Option<f32>,
        placeholder: Vec<TextPart>,
        disabled: BoolExpr,
        autofocus: bool,
        /// Handlers for losing focus by Enter, by Escape, and otherwise.
        submit: Option<Ref>,
        cancel: Option<Ref>,
        blur: Option<Ref>,
    },
    Switch {
        model: Ref,
        size: Expr<&'static str>,
        disabled: BoolExpr,
    },
    Checkbox {
        model: Ref,
        label: Vec<TextPart>,
        size: Expr<&'static str>,
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
        .root(root)
    }
}
const CONDITIONS: [&str; 3] = ["e-if", "e-else-if", "e-else"];
fn chain((condition, node): (Option<BoolExpr>, Node)) -> Node {
    Node {
        key: node.key.clone(),
        explicit_id: None,
        kind: Kind::If {
            branches: vec![(condition, node)],
        },
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
    /// The handler named by an optional event attribute such as `@click`.
    fn event(&self, e: &Element, attr: &str) -> Result<Option<Ref>, Error> {
        e.attrs
            .get(attr)
            .map(|a| self.action(e, attr, a.value.trim()))
            .transpose()
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
    fn root(&mut self, e: Element) -> Result<Node, Error> {
        match self.directive(&e)? {
            Some("e-if") => {
                let branch = self.branch(e, "0".into())?;
                Ok(chain(branch))
            }
            Some(d) => Err(self.error(
                &e,
                d,
                format!("{d} must follow an element with e-if or e-else-if; the root has none"),
            )),
            None => self.node(e, "0".into(), true),
        }
    }
    /// The conditional directive on an element, if any; at most one is allowed.
    fn directive(&self, e: &Element) -> Result<Option<&'static str>, Error> {
        let mut found = CONDITIONS.into_iter().filter(|d| e.attrs.contains_key(*d));
        let first = found.next();
        if let Some(second) = found.next() {
            return Err(self.error(
                e,
                second,
                "an element takes only one of e-if, e-else-if and e-else",
            ));
        }
        Ok(first)
    }
    /// Child elements, grouping e-if/e-else-if/e-else siblings into chains. Every branch
    /// keeps its own structural path, so branch selection never shifts sibling identity.
    fn children(&mut self, elements: Vec<Element>, path: &str) -> Result<Vec<Node>, Error> {
        let mut nodes: Vec<Node> = vec![];
        let mut open = false;
        for (i, child) in elements.into_iter().enumerate() {
            let child_path = format!("{path}/{i}");
            match self.directive(&child)? {
                Some("e-if") => {
                    let branch = self.branch(child, child_path)?;
                    nodes.push(chain(branch));
                    open = true;
                }
                Some(d) => {
                    if !open {
                        return Err(self.error(
                            &child,
                            d,
                            format!("{d} must directly follow an element with e-if or e-else-if"),
                        ));
                    }
                    open = d == "e-else-if";
                    let branch = self.branch(child, child_path)?;
                    let Some(Node {
                        kind: Kind::If { branches },
                        ..
                    }) = nodes.last_mut()
                    else {
                        unreachable!("an open chain ends the node list")
                    };
                    branches.push(branch);
                }
                None => {
                    nodes.push(self.node(child, child_path, false)?);
                    open = false;
                }
            }
        }
        Ok(nodes)
    }
    fn branch(&mut self, mut e: Element, path: String) -> Result<(Option<BoolExpr>, Node), Error> {
        let directive = self.directive(&e)?.expect("conditional element");
        if e.attrs.contains_key("e-for") {
            return Err(self.error(
                &e,
                directive,
                format!("{directive} cannot be combined with e-for; nest them, using a fragment if no layout is wanted"),
            ));
        }
        let condition = if directive == "e-else" {
            None
        } else {
            let (r, negate) = self.condition(&e, directive)?;
            Some(BoolExpr::Binding(r, negate))
        };
        e.attrs.remove(directive);
        Ok((condition, self.node(e, path, false)?))
    }
    /// A bool name, optionally negated with `!`: the condition syntax of :disabled and e-if.
    fn condition(&self, e: &Element, attr: &str) -> Result<(Ref, bool), Error> {
        self.condition_in(e, attr, self.attr(e, attr, ""))
    }
    fn condition_in(&self, e: &Element, attr: &str, expr: &str) -> Result<(Ref, bool), Error> {
        let expr = expr.trim();
        let (name, negate) = expr
            .strip_prefix('!')
            .map_or((expr, false), |n| (n.trim(), true));
        if name.is_empty() {
            return Err(self.error(e, attr, format!("{attr} needs a registered bool name")));
        }
        Ok((
            self.binding(e, attr, name, Some(ValueKind::Bool), false)?,
            negate,
        ))
    }
    /// A bool attribute such as `disabled`: `true`/`false`, or `:name` with a condition.
    fn flag(&self, e: &Element, name: &str) -> Result<BoolExpr, Error> {
        let dynamic = format!(":{name}");
        if e.attrs.contains_key(name) && e.attrs.contains_key(&dynamic) {
            return Err(self.error(e, &dynamic, format!("cannot combine {name} and {dynamic}")));
        }
        if e.attrs.contains_key(&dynamic) {
            let (r, negate) = self.condition(e, &dynamic)?;
            Ok(BoolExpr::Binding(r, negate))
        } else {
            Ok(BoolExpr::Literal(
                self.choice(e, name, "false", &["true", "false"])? == "true",
            ))
        }
    }
    /// A styling attribute: a constant `name="value"`, or `:name` with an expression.
    fn value<T: Copy>(
        &self,
        e: &Element,
        name: &str,
        default: T,
        leaf: LeafFn<T>,
    ) -> Result<Expr<T>, Error> {
        Ok(self
            .optional(e, name, leaf)?
            .unwrap_or(Expr::Const(default)))
    }
    fn optional<T: Copy>(
        &self,
        e: &Element,
        name: &str,
        leaf: LeafFn<T>,
    ) -> Result<Option<Expr<T>>, Error> {
        let dynamic = format!(":{name}");
        match (e.attrs.get(name), e.attrs.get(&dynamic)) {
            (Some(_), Some(_)) => {
                Err(self.error(e, &dynamic, format!("cannot combine {name} and {dynamic}")))
            }
            (Some(a), None) => {
                let value = a.value.trim();
                let parsed = if value.starts_with('$') {
                    Leaf::Theme(value)
                } else {
                    Leaf::Word(value)
                };
                leaf(parsed)
                    .map(|v| Some(Expr::Const(v)))
                    .map_err(|m| self.error(e, name, m))
            }
            (None, Some(a)) => self.expr(e, &dynamic, &a.value, leaf).map(Some),
            (None, None) => Ok(None),
        }
    }
    fn leaf_value<'s, T>(
        &self,
        e: &Element,
        attr: &str,
        s: &'s str,
        leaf: LeafFn<T>,
        shape: &str,
    ) -> Result<(T, &'s str), Error> {
        let (l, rest) = self::leaf(s.trim_start()).ok_or_else(|| self.error(e, attr, shape))?;
        Ok((
            leaf(l).map_err(|m| self.error(e, attr, m))?,
            rest.trim_start(),
        ))
    }
    /// `value` or `cond ? value : expr`, where a value is `'word'` or a `$theme` path and
    /// `cond` is a bool name, optionally negated with `!`.
    fn expr<T: Copy>(
        &self,
        e: &Element,
        attr: &str,
        src: &str,
        leaf: LeafFn<T>,
    ) -> Result<Expr<T>, Error> {
        const SHAPE: &str =
            "expected a quoted value like 'ghost', a $theme path, or cond ? value : value";
        let src = src.trim();
        let value = |s| self.leaf_value(e, attr, s, leaf, SHAPE);
        if src.starts_with(['\'', '$']) {
            let (v, rest) = value(src)?;
            if !rest.is_empty() {
                return Err(self.error(
                    e,
                    attr,
                    format!("unexpected '{rest}'; a condition goes first: cond ? value : value"),
                ));
            }
            return Ok(Expr::Const(v));
        }
        let Some((condition, rest)) = src.split_once('?') else {
            return Err(self.error(e, attr, SHAPE));
        };
        let (r, negate) = self.condition_in(e, attr, condition)?;
        let (then, rest) = value(rest)?;
        let Some(otherwise) = rest.strip_prefix(':') else {
            return Err(self.error(
                e,
                attr,
                "expected ':' after the value chosen when the condition holds",
            ));
        };
        Ok(Expr::If(
            BoolExpr::Binding(r, negate),
            then,
            Box::new(self.expr(e, attr, otherwise, leaf)?),
        ))
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
            // Only id and directives, which are handled before reaching here.
            "fragment" => &[],
            "layout" => &["direction", "cross_align", "item_spacing.x", "item_spacing.y"],
            "text" => &["size", ":size", "color", ":color", "underline", ":underline", "strikethrough", ":strikethrough", "@click", "@double_click"],
            "button" => &["size", ":size", "variant", ":variant", "color", ":color", "width", "disabled", ":disabled", "@click"],
            "text_edit" => &["size", ":size", "variant", ":variant", "width", "placeholder", "disabled", ":disabled", "e-model", "autofocus", "@submit", "@cancel", "@blur"],
            "switch" | "checkbox" => &["size", ":size", "disabled", ":disabled", "e-model"],
            _ => return Err(self.error(&e, "", format!("unknown component '{}'; prototype supports built-in layout and fragment and components text, button, text_edit, switch, checkbox (no aliases)", e.tag))),
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
        if !matches!(e.tag.as_str(), "layout" | "fragment") && !e.children.is_empty() {
            return Err(self.error(&e, "", "only layout and fragment accept child components"));
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
                let children = self.children(e.children, &path)?;
                Kind::Layout {
                    horizontal,
                    cross_align,
                    item_spacing_x,
                    item_spacing_y,
                    children,
                }
            }
            "fragment" => Kind::Fragment {
                children: self.children(e.children, &path)?,
            },
            "text" => Kind::Text {
                text: self.text(&e, "", e.text.trim())?,
                heading: self.value(
                    &e,
                    "size",
                    false,
                    &words("size", &[("md", false), ("heading", true)]),
                )?,
                color: self.optional(&e, "color", &color)?,
                underline: self.flag(&e, "underline")?,
                strikethrough: self.flag(&e, "strikethrough")?,
                click: self.event(&e, "@click")?,
                double_click: self.event(&e, "@double_click")?,
            },
            "button" => {
                let size = self.value(
                    &e,
                    "size",
                    ButtonSize::Md,
                    &words(
                        "size",
                        &[
                            ("xs", ButtonSize::Xs),
                            ("sm", ButtonSize::Sm),
                            ("md", ButtonSize::Md),
                            ("lg", ButtonSize::Lg),
                        ],
                    ),
                )?;
                let variant = self.value(
                    &e,
                    "variant",
                    ButtonVariant::Solid,
                    &words(
                        "variant",
                        &[
                            ("solid", ButtonVariant::Solid),
                            ("surface", ButtonVariant::Surface),
                            ("outline", ButtonVariant::Outline),
                            ("ghost", ButtonVariant::Ghost),
                        ],
                    ),
                )?;
                let click = self.event(&e, "@click")?;
                Kind::Button {
                    text: self.text(&e, "", e.text.trim())?,
                    size,
                    variant,
                    color: self.optional(&e, "color", &color_scale)?,
                    click,
                    width: self.number(&e, "width")?,
                    disabled: self.flag(&e, "disabled")?,
                }
            }
            "text_edit" => {
                let size = self.value(
                    &e,
                    "size",
                    TextEditSize::Md,
                    &words(
                        "size",
                        &[
                            ("xs", TextEditSize::Xs),
                            ("sm", TextEditSize::Sm),
                            ("md", TextEditSize::Md),
                            ("lg", TextEditSize::Lg),
                        ],
                    ),
                )?;
                let variant = self.value(
                    &e,
                    "variant",
                    TextEditVariant::Outline,
                    &words(
                        "variant",
                        &[
                            ("subtle", TextEditVariant::Subtle),
                            ("outline", TextEditVariant::Outline),
                            ("flushed", TextEditVariant::Flushed),
                        ],
                    ),
                )?;
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
                    disabled: self.flag(&e, "disabled")?,
                    autofocus: self.choice(&e, "autofocus", "false", &["true", "false"])? == "true",
                    submit: self.event(&e, "@submit")?,
                    cancel: self.event(&e, "@cancel")?,
                    blur: self.event(&e, "@blur")?,
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
                size: self.value(
                    &e,
                    "size",
                    "md",
                    &words("size", &[("sm", "sm"), ("md", "md"), ("lg", "lg")]),
                )?,
                disabled: self.flag(&e, "disabled")?,
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
                size: self.value(
                    &e,
                    "size",
                    "md",
                    &words("size", &[("sm", "sm"), ("md", "md"), ("lg", "lg")]),
                )?,
                disabled: self.flag(&e, "disabled")?,
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
