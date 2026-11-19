//! Experimental, typed ECML interpreter. See `examples/ecml_hello_world` for its deliberately small grammar.
//!
//! Component tags use the exact names of `components` modules; `layout` is built in. A View is one mounted UI instance:
//! show it once per pass, in one egui Context/viewport. Use separate Views for two mounts.
//! Setters must only assign state; external effects belong in handlers. Each frame's first
//! show submits edits immediately and then handlers in tree order. Later passes render the
//! resulting state without resubmitting interactions. This is not rollback/transaction semantics.
use super::Source;
use super::template::{BoolExpr, Kind, Node, TextPart};
use super::{parser, source, template};
use crate::{
    components::{button, switch, text, text_edit},
    theme::Theme,
};
use std::{
    collections::HashMap,
    fmt,
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
}
impl Error {
    pub(super) fn at(file: &str, source: &str, offset: usize, message: impl Into<String>) -> Self {
        let before = &source[..offset.min(source.len())];
        Self {
            file: file.into(),
            line: before.bytes().filter(|b| *b == b'\n').count() + 1,
            column: before.rsplit('\n').next().unwrap_or("").chars().count() + 1,
            message: message.into(),
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}",
            self.file, self.line, self.column, self.message
        )
    }
}
impl std::error::Error for Error {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueKind {
    String,
    Bool,
    Integer,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    String(String),
    Bool(bool),
    Integer(i64),
}
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(v) => v.fmt(f),
            Self::Bool(v) => v.fmt(f),
            Self::Integer(v) => v.fmt(f),
        }
    }
}
mod sealed {
    pub trait Sealed {}
}
/// The prototype accepts String, bool, i32 and i64, without lossy coercions.
pub trait BindingValue: sealed::Sealed + Sized + 'static {
    const KIND: ValueKind;
    fn into_value(self) -> Value;
    fn from_value(value: Value) -> Self;
}
macro_rules! value_type {
    ($ty:ty, $kind:ident) => {
        impl sealed::Sealed for $ty {}
        impl BindingValue for $ty {
            const KIND: ValueKind = ValueKind::$kind;
            fn into_value(self) -> Value {
                Value::$kind(self)
            }
            fn from_value(value: Value) -> Self {
                match value {
                    Value::$kind(v) => v,
                    _ => unreachable!("validated binding type"),
                }
            }
        }
    };
}
value_type!(String, String);
value_type!(bool, Bool);
value_type!(i64, Integer);
impl sealed::Sealed for i32 {}
impl BindingValue for i32 {
    const KIND: ValueKind = ValueKind::Integer;
    fn into_value(self) -> Value {
        Value::Integer(i64::from(self))
    }
    fn from_value(value: Value) -> Self {
        match value {
            Value::Integer(v) => i32::try_from(v).expect("i32 binding range"),
            _ => unreachable!(),
        }
    }
}

type Getter<S> = Box<dyn Fn(&S) -> Value>;
type Setter<S> = Box<dyn Fn(&mut S, Value)>;
type Handler<S> = Box<dyn FnMut(&mut S)>;
pub(super) struct Binding<S> {
    pub(super) name: String,
    pub(super) kind: ValueKind,
    get: Getter<S>,
    pub(super) set: Option<Setter<S>>,
}
pub(super) struct Action<S> {
    pub(super) name: String,
    handler: Handler<S>,
}
pub struct Builder<S> {
    bindings: Vec<Binding<S>>,
    actions: Vec<Action<S>>,
    error_window: bool,
}
impl<S: 'static> Builder<S> {
    pub fn model<T: BindingValue + Clone>(
        mut self,
        name: impl Into<String>,
        get: impl for<'a> Fn(&'a S) -> &'a T + 'static,
        set: impl Fn(&mut S, T) + 'static,
    ) -> Self {
        self.bindings.push(Binding {
            name: name.into(),
            kind: T::KIND,
            get: Box::new(move |s| get(s).clone().into_value()),
            set: Some(Box::new(move |s, v| set(s, T::from_value(v)))),
        });
        self
    }
    pub fn bind<T: BindingValue>(
        mut self,
        name: impl Into<String>,
        get: impl Fn(&S) -> T + 'static,
    ) -> Self {
        self.bindings.push(Binding {
            name: name.into(),
            kind: T::KIND,
            get: Box::new(move |s| get(s).into_value()),
            set: None,
        });
        self
    }
    pub fn on(mut self, name: impl Into<String>, handler: impl FnMut(&mut S) + 'static) -> Self {
        self.actions.push(Action {
            name: name.into(),
            handler: Box::new(handler),
        });
        self
    }
    /// Whether `show` draws the built-in floating window for replacement errors (default
    /// `true`). Disable it to present [`View::last_error`] in your own UI instead.
    pub fn error_window(mut self, show: bool) -> Self {
        self.error_window = show;
        self
    }
    pub fn load(self, source: Source) -> Result<View<S>, Error> {
        let (file, content, path) = match source {
            Source::Inline { name, text } => (name, text, None::<std::path::PathBuf>),
            #[cfg(not(target_arch = "wasm32"))]
            Source::File(path) => (path.display().to_string(), source::load(&path)?, Some(path)),
        };
        let registry = template::Registry::new(&self.bindings, &self.actions)
            .map_err(|message| Error::at(&file, &content, 0, message))?;
        let root = registry.compile(parser::parse(&content, &file)?, &content, &file)?;
        #[cfg(not(target_arch = "wasm32"))]
        let watcher = path.map(|path| source::Watcher::start(path, content));
        #[cfg(target_arch = "wasm32")]
        let _ = path;
        static NEXT_INSTANCE: AtomicU64 = AtomicU64::new(1);
        Ok(View {
            root,
            bindings: self.bindings,
            actions: self.actions,
            error_window: self.error_window,
            file,
            error: None,
            instance: egui::Id::unique(("ecml", NEXT_INSTANCE.fetch_add(1, Ordering::Relaxed))),
            frame: None,
            responses: HashMap::new(),
            pending: None,
            #[cfg(not(target_arch = "wasm32"))]
            watcher,
        })
    }
}

/// One mounted template; state is always owned by the caller.
pub struct View<S> {
    root: Node,
    bindings: Vec<Binding<S>>,
    actions: Vec<Action<S>>,
    error_window: bool,
    instance: egui::Id,
    file: String,
    error: Option<Error>,
    frame: Option<(egui::ViewportId, u64)>,
    responses: HashMap<String, egui::Response>,
    pending: Option<source::Candidate>,
    #[cfg(not(target_arch = "wasm32"))]
    watcher: Option<source::Watcher>,
}
impl<S: 'static> View<S> {
    pub fn builder() -> Builder<S> {
        Builder {
            bindings: vec![],
            actions: vec![],
            error_window: true,
        }
    }
    pub fn last_error(&self) -> Option<&Error> {
        self.error.as_ref()
    }
    /// Response for an explicitly named component from the most recent show.
    pub fn response(&self, id: &str) -> Option<&egui::Response> {
        self.responses.get(id)
    }
    /// Queue a replacement for the next frame. Invalid replacements preserve the current tree.
    pub fn replace(&mut self, text: impl Into<String>) {
        let text = text.into();
        self.pending = Some(source::Candidate {
            parsed: parser::parse(&text, &self.file),
            text,
        });
    }
    pub fn show(&mut self, ui: &mut egui::Ui, theme: &Theme, state: &mut S) {
        let stamp = (ui.ctx().viewport_id(), ui.ctx().cumulative_frame_nr());
        let first = self.frame != Some(stamp);
        if first {
            self.frame = Some(stamp);
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(watcher) = &self.watcher
                && let Some(candidate) = watcher.poll(ui.ctx())
            {
                self.pending = Some(candidate);
            }
            if let Some(candidate) = self.pending.take() {
                let result = candidate.parsed.and_then(|tree| {
                    template::Registry::new(&self.bindings, &self.actions)
                        .map_err(|m| Error::at(&self.file, &candidate.text, 0, m))?
                        .compile(tree, &candidate.text, &self.file)
                });
                match result {
                    Ok(root) => {
                        self.root = root;
                        self.error = None;
                    }
                    Err(error) => self.error = Some(error),
                }
            }
        }
        let mut events = vec![];
        self.responses.clear();
        let mut runtime = Runtime {
            bindings: &self.bindings,
            state,
            first,
            events: &mut events,
            instance: self.instance.with(stamp.0),
            responses: &mut self.responses,
            changed: false,
        };
        render(&self.root, ui, theme, &mut runtime);
        let changed = runtime.changed;
        for action in &events {
            (self.actions[*action].handler)(state);
        }
        if changed || !events.is_empty() {
            ui.ctx().request_repaint();
        }
        if self.error_window
            && let Some(error) = &self.error
        {
            egui::Window::new("ECML template error")
                .id(self.instance.with("error"))
                .show(ui.ctx(), |ui| {
                    ui.colored_label(egui::Color32::LIGHT_RED, error.to_string());
                    ui.label("Showing the last valid template. Save a correction to recover.");
                });
        }
    }
}
struct Runtime<'a, S> {
    bindings: &'a [Binding<S>],
    state: &'a mut S,
    first: bool,
    events: &'a mut Vec<usize>,
    instance: egui::Id,
    responses: &'a mut HashMap<String, egui::Response>,
    changed: bool,
}
impl<S> Runtime<'_, S> {
    fn get(&self, index: usize) -> Value {
        (self.bindings[index].get)(self.state)
    }
    fn write(&mut self, index: usize, value: Value) {
        if self.first {
            (self.bindings[index].set.as_ref().expect("validated model"))(self.state, value);
            self.changed = true;
        }
    }
    fn boolean(&self, expr: &BoolExpr) -> bool {
        match expr {
            BoolExpr::Literal(v) => *v,
            BoolExpr::Binding(i, not) => matches!(self.get(*i), Value::Bool(true)) ^ not,
        }
    }
    fn text(&self, parts: &[TextPart]) -> String {
        parts
            .iter()
            .map(|p| match p {
                TextPart::Literal(s) => s.clone(),
                TextPart::Binding(i) => self.get(*i).to_string(),
            })
            .collect()
    }
}
fn render<S>(node: &Node, ui: &mut egui::Ui, theme: &Theme, rt: &mut Runtime<'_, S>) {
    // Explicit scope_id keeps auto-allocated IDs inside existing components stable too.
    ui.scope_builder(
        egui::UiBuilder::new().scope_id(rt.instance.with(&node.key)),
        |ui| {
            let response = match &node.kind {
                Kind::Layout {
                    horizontal,
                    cross_align,
                    item_spacing_x,
                    item_spacing_y,
                    children,
                } => {
                    // This Ui is already scoped to the node. Overrides inherit into descendants,
                    // never into siblings or the parent. Missing axes retain host spacing.
                    if let Some(x) = item_spacing_x {
                        ui.spacing_mut().item_spacing.x = *x;
                    }
                    if let Some(y) = item_spacing_y {
                        ui.spacing_mut().item_spacing.y = *y;
                    }
                    let contents = |ui: &mut egui::Ui| {
                        for child in children {
                            render(child, ui, theme, rt);
                        }
                    };
                    if *horizontal {
                        // Match Ui::horizontal's initial height, not all remaining height.
                        let initial_size = egui::vec2(
                            ui.available_size_before_wrap().x,
                            ui.spacing().interact_size.y,
                        );
                        ui.allocate_ui_with_layout(
                            initial_size,
                            egui::Layout::left_to_right(*cross_align),
                            contents,
                        );
                    } else {
                        ui.with_layout(egui::Layout::top_down(*cross_align), contents);
                    }
                    None
                }
                Kind::Text {
                    text: parts,
                    heading,
                } => {
                    let style = if *heading {
                        text::Style::new_heading(theme)
                    } else {
                        text::Style::new(theme)
                    };
                    Some(text::render(ui, &rt.text(parts), style))
                }
                Kind::Button {
                    text,
                    size,
                    variant,
                    width,
                    disabled,
                    click,
                } => {
                    let style = button::Style::new(
                        theme,
                        None,
                        *size,
                        *variant,
                        button::ButtonIcon::None,
                        button::ButtonIcon::None,
                    )
                    .width(*width)
                    .state(if rt.boolean(disabled) {
                        button::ButtonState::Disable
                    } else {
                        button::ButtonState::None
                    });
                    let response = button::render(ui, &rt.text(text), style);
                    if rt.first
                        && response.clicked()
                        && let Some(click) = click
                    {
                        rt.events.push(*click);
                    }
                    Some(response)
                }
                Kind::TextEdit {
                    model,
                    size,
                    variant,
                    width,
                    placeholder,
                    disabled,
                } => {
                    let Value::String(mut value) = rt.get(*model) else {
                        unreachable!()
                    };
                    let style = text_edit::Style::new(theme, None, *size, *variant)
                        .width(*width)
                        .placeholder(rt.text(placeholder))
                        .disabled(rt.boolean(disabled))
                        .read_only(!rt.first);
                    let response = text_edit::render(ui, &mut value, style);
                    if response.changed() {
                        rt.write(*model, Value::String(value));
                    }
                    Some(response)
                }
                Kind::Switch {
                    model,
                    size,
                    disabled,
                } => {
                    let Value::Bool(mut value) = rt.get(*model) else {
                        unreachable!()
                    };
                    let style = match size.as_str() {
                        "sm" => switch::Style::new_sm(theme),
                        "lg" => switch::Style::new_lg(theme),
                        _ => switch::Style::new_md(theme),
                    };
                    let response =
                        switch::render(ui, &mut value, style.disabled(rt.boolean(disabled)));
                    if response.changed() {
                        rt.write(*model, Value::Bool(value));
                    }
                    Some(response)
                }
            };
            if let (Some(id), Some(response)) = (&node.explicit_id, response) {
                rt.responses.insert(id.clone(), response);
            }
        },
    );
}
