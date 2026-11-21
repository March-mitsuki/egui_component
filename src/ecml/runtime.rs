//! Experimental, typed ECML interpreter. See `examples/ecml_hello_world` for its deliberately small grammar.
//!
//! Component tags use the exact names of `components` modules; `layout` is built in. A View is one mounted UI instance:
//! show it once per pass, in one egui Context/viewport. Use separate Views for two mounts.
//! Setters must only assign state; external effects belong in handlers. Each frame's first
//! show submits edits immediately and then handlers in tree order. Later passes render the
//! resulting state without resubmitting interactions. This is not rollback/transaction semantics.
use super::Source;
use super::template::{BoolExpr, Expr, Kind, Node, Ref, TextPart, TextSize};
use super::{parser, source, template};
use crate::{
    components::{button, checkbox, switch, text, text_edit},
    theme::Theme,
};
use std::{
    any::Any,
    collections::{HashMap, HashSet},
    fmt,
    hash::Hash,
    marker::PhantomData,
    rc::Rc,
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
/// A row's key, type-erased so loops of different row and key types share one runtime.
/// `own` locates the row in its parent; `path` is what row handlers receive.
#[derive(Clone)]
struct RowKey {
    own: Rc<dyn Any>,
    path: Rc<dyn Any>,
    id: egui::Id,
}
/// Rows of a list for one parent (the state for root lists, else the parent row and its
/// key path): calls back once per unique key and returns the first duplicate key, if any.
type Rows<S> = Box<
    dyn for<'s> Fn(
        &'s S,
        &'s dyn Any,
        Option<&dyn Any>,
        &mut dyn FnMut(&'s dyn Any, RowKey),
    ) -> Option<String>,
>;
/// Locates a row by its own key in its parent (the state, or the parent row).
type Find = Box<dyn for<'s> Fn(&'s mut dyn Any, &dyn Any) -> Option<&'s mut dyn Any>>;
type RowGetter<S> = Box<dyn Fn(&S, &dyn Any) -> Value>;
type RowSetter = Box<dyn Fn(&mut dyn Any, Value)>;
type RowHandler<S> = Box<dyn FnMut(&mut S, &dyn Any)>;
pub(super) struct RowBinding<S> {
    pub(super) name: String,
    pub(super) kind: ValueKind,
    get: RowGetter<S>,
    pub(super) set: Option<RowSetter>,
}
pub(super) struct RowAction<S> {
    pub(super) name: String,
    handler: RowHandler<S>,
}
pub(super) struct List<S> {
    pub(super) name: String,
    pub(super) bindings: Vec<RowBinding<S>>,
    pub(super) actions: Vec<RowAction<S>>,
    /// Child lists of each row, reached as `e-for="item in alias.name"`.
    pub(super) lists: Vec<List<S>>,
    rows: Rows<S>,
    find: Find,
}
impl<S: 'static> List<S> {
    /// Erases a list whose rows `T` live in a parent `P` and are handled with key path `Path`.
    fn new<P: 'static, T: 'static, K: Clone + Eq + Hash + fmt::Debug + 'static, Path: 'static>(
        name: String,
        read: impl for<'a> Fn(&'a S, &'a P) -> Box<dyn Iterator<Item = &'a T> + 'a> + 'static,
        find: impl for<'a, 'b> Fn(&'a mut P, &'b K) -> Option<&'a mut T> + 'static,
        key: impl Fn(&T) -> K + 'static,
        path: impl Fn(Option<&dyn Any>, &K) -> Path + 'static,
        row: RowBuilder<S, T, Path>,
    ) -> Self {
        Self {
            name,
            bindings: row.bindings,
            actions: row.actions,
            lists: row.lists,
            rows: Box::new(move |s, parent, parent_path, each| {
                // Duplicates are checked among the rows of one parent only.
                let mut seen = HashSet::new();
                let mut duplicate = None;
                for item in read(s, downcast(parent)) {
                    let k = key(item);
                    if seen.contains(&k) {
                        duplicate.get_or_insert_with(|| format!("{k:?}"));
                        continue;
                    }
                    let row = RowKey {
                        id: egui::Id::unique(&k),
                        path: Rc::new(path(parent_path, &k)),
                        own: Rc::new(k.clone()),
                    };
                    seen.insert(k);
                    each(item, row);
                }
                duplicate
            }),
            find: Box::new(move |parent, k| {
                find(
                    parent.downcast_mut().expect("validated parent type"),
                    downcast(k),
                )
                .map(|t| t as &mut dyn Any)
            }),
        }
    }
}
fn list_at<'a, S>(lists: &'a [List<S>], path: &[usize]) -> &'a List<S> {
    let (first, rest) = path.split_first().expect("nonempty list path");
    rest.iter().fold(&lists[*first], |list, i| &list.lists[*i])
}
/// Registrations for one row of a list, reached in templates as `alias.name`. `K` is the
/// key path row handlers receive: the key for root lists, `(ParentPath, Key)` for child lists.
pub struct RowBuilder<S, T, K> {
    bindings: Vec<RowBinding<S>>,
    actions: Vec<RowAction<S>>,
    lists: Vec<List<S>>,
    _row: PhantomData<fn(&T, &K)>,
}
impl<S, T, K> RowBuilder<S, T, K> {
    fn new() -> Self {
        Self {
            bindings: vec![],
            actions: vec![],
            lists: vec![],
            _row: PhantomData,
        }
    }
}
impl<S: 'static, T: 'static, K: 'static> RowBuilder<S, T, K> {
    /// A writable row value. The getter returns the value (clone a field, or compute it);
    /// the setter receives only the row.
    pub fn model<V: BindingValue>(
        mut self,
        name: impl Into<String>,
        get: impl Fn(&T) -> V + 'static,
        set: impl Fn(&mut T, V) + 'static,
    ) -> Self {
        self.bindings.push(RowBinding {
            name: name.into(),
            kind: V::KIND,
            get: Box::new(move |_, row| get(downcast(row)).into_value()),
            set: Some(Box::new(move |row, v| {
                set(row.downcast_mut().expect("row type"), V::from_value(v))
            })),
        });
        self
    }
    /// A read-only row value, which may also depend on the whole state.
    pub fn bind<V: BindingValue>(
        mut self,
        name: impl Into<String>,
        get: impl Fn(&S, &T) -> V + 'static,
    ) -> Self {
        self.bindings.push(RowBinding {
            name: name.into(),
            kind: V::KIND,
            get: Box::new(move |s, row| get(s, downcast(row)).into_value()),
            set: None,
        });
        self
    }
    /// A child list of each row, for `e-for="item in alias.name"`. `read` receives the
    /// parent row, and `find` locates a child by its own key in the mutable parent row.
    /// Child rows are handled with the key path `(K, IK)`.
    pub fn list<I: 'static, IK: Clone + Eq + Hash + fmt::Debug + 'static>(
        mut self,
        name: impl Into<String>,
        read: impl for<'a> Fn(&'a S, &'a T) -> Box<dyn Iterator<Item = &'a I> + 'a> + 'static,
        find: impl for<'a, 'b> Fn(&'a mut T, &'b IK) -> Option<&'a mut I> + 'static,
        key: impl Fn(&I) -> IK + 'static,
        rows: impl FnOnce(RowBuilder<S, I, (K, IK)>) -> RowBuilder<S, I, (K, IK)>,
    ) -> Self
    where
        K: Clone,
    {
        self.lists.push(List::new(
            name.into(),
            read,
            find,
            key,
            |parent: Option<&dyn Any>, k: &IK| {
                let parent: &K = downcast(parent.expect("child rows have a parent"));
                (parent.clone(), k.clone())
            },
            rows(RowBuilder::new()),
        ));
        self
    }
    /// A row handler; it receives the key path of the row whose control was clicked.
    pub fn on(
        mut self,
        name: impl Into<String>,
        mut handler: impl FnMut(&mut S, &K) + 'static,
    ) -> Self {
        self.actions.push(RowAction {
            name: name.into(),
            handler: Box::new(move |s, key| handler(s, downcast(key))),
        });
        self
    }
}
fn downcast<T: 'static>(value: &dyn Any) -> &T {
    value.downcast_ref().expect("validated row type")
}
pub struct Builder<S> {
    bindings: Vec<Binding<S>>,
    actions: Vec<Action<S>>,
    lists: Vec<List<S>>,
    error_window: bool,
}
impl<S: 'static> Builder<S> {
    /// A readable and writable value for `e-model`. The getter returns the value: clone a
    /// field, or compute one (such as "all done", with a setter that applies it to all).
    pub fn model<T: BindingValue>(
        mut self,
        name: impl Into<String>,
        get: impl Fn(&S) -> T + 'static,
        set: impl Fn(&mut S, T) + 'static,
    ) -> Self {
        self.bindings.push(Binding {
            name: name.into(),
            kind: T::KIND,
            get: Box::new(move |s| get(s).into_value()),
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
    /// A list for `e-for="alias in name"`. `read` yields the rows to show, in order (filter
    /// and sort in Rust). `find` locates a row by key for model writes; `None` drops the
    /// write. `key` identifies rows, so focus and editing state follow the row, not its
    /// position. Only the first row for each key is shown; duplicates are reported.
    pub fn list<T: 'static, K: Clone + Eq + Hash + fmt::Debug + 'static>(
        mut self,
        name: impl Into<String>,
        read: impl for<'a> Fn(&'a S) -> Box<dyn Iterator<Item = &'a T> + 'a> + 'static,
        find: impl for<'a, 'b> Fn(&'a mut S, &'b K) -> Option<&'a mut T> + 'static,
        key: impl Fn(&T) -> K + 'static,
        rows: impl FnOnce(RowBuilder<S, T, K>) -> RowBuilder<S, T, K>,
    ) -> Self {
        self.lists.push(List::new(
            name.into(),
            move |s, _: &S| read(s),
            find,
            key,
            |_, k: &K| k.clone(),
            rows(RowBuilder::new()),
        ));
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
        let registry = template::Registry::new(&self.bindings, &self.actions, &self.lists)
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
            lists: self.lists,
            error_window: self.error_window,
            file,
            error: None,
            runtime_error: None,
            autofocused: HashSet::new(),
            rects: HashMap::new(),
            focus_within: HashSet::new(),
            autofocus_retry: HashMap::new(),
            text_focus: HashSet::new(),
            lost_reported: HashSet::new(),
            instance: egui::Id::unique(("ecml", NEXT_INSTANCE.fetch_add(1, Ordering::Relaxed))),
            frame: None,
            responses: HashMap::new(),
            #[cfg(test)]
            row_responses: HashMap::new(),
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
    lists: Vec<List<S>>,
    error_window: bool,
    instance: egui::Id,
    file: String,
    /// The last invalid template; the previous valid tree stays visible.
    error: Option<Error>,
    /// A problem found while drawing, such as a duplicate list key; recomputed each pass.
    runtime_error: Option<Error>,
    /// Autofocus controls drawn on the last frame, so they are focused only on appearing.
    autofocused: HashSet<egui::Id>,
    /// Areas of elements with an id, from the last show, for `$contains_pointer`.
    rects: HashMap<egui::Id, (egui::Rect, egui::LayerId)>,
    /// Elements with an id holding the focused control on the last show, for
    /// `$contains_focus`.
    focus_within: HashSet<egui::Id>,
    autofocus_retry: HashMap<egui::Id, egui::Id>,
    /// Text edits focused when last drawn and those whose focus loss was reported on the
    /// last frame, so each loss fires one handler (see the text_edit renderer).
    text_focus: HashSet<egui::Id>,
    lost_reported: HashSet<egui::Id>,
    frame: Option<(egui::ViewportId, u64)>,
    responses: HashMap<String, egui::Response>,
    /// Explicitly named controls inside loops, by id and row key, for tests.
    #[cfg(test)]
    row_responses: HashMap<(String, egui::Id), egui::Response>,
    pending: Option<source::Candidate>,
    #[cfg(not(target_arch = "wasm32"))]
    watcher: Option<source::Watcher>,
}
impl<S: 'static> View<S> {
    pub fn builder() -> Builder<S> {
        Builder {
            bindings: vec![],
            actions: vec![],
            lists: vec![],
            error_window: true,
        }
    }
    /// The current template error, otherwise a problem found while drawing the last pass.
    pub fn last_error(&self) -> Option<&Error> {
        self.error.as_ref().or(self.runtime_error.as_ref())
    }
    /// Response for an explicitly named component from the most recent show.
    /// Components inside `e-for` loops are not included.
    pub fn response(&self, id: &str) -> Option<&egui::Response> {
        self.responses.get(id)
    }
    #[cfg(test)]
    pub(super) fn row_response(
        &self,
        id: &str,
        key: &(impl Hash + fmt::Debug),
    ) -> Option<&egui::Response> {
        self.row_responses
            .get(&(id.to_owned(), egui::Id::unique(key)))
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
                    template::Registry::new(&self.bindings, &self.actions, &self.lists)
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
        self.responses.clear();
        #[cfg(test)]
        self.row_responses.clear();
        let mut runtime = Runtime {
            bindings: &self.bindings,
            lists: &self.lists,
            file: &self.file,
            first,
            events: vec![],
            writes: vec![],
            instance: self.instance.with(stamp.0),
            root: self.instance.with(stamp.0),
            rows: vec![],
            ctx: ui.ctx().clone(),
            rects: &self.rects,
            rects_now: HashMap::new(),
            focus_within: &self.focus_within,
            focus_within_now: HashSet::new(),
            focus_inside: false,
            responses: &mut self.responses,
            #[cfg(test)]
            row_responses: &mut self.row_responses,
            changed: false,
            runtime_error: None,
            autofocused: &self.autofocused,
            autofocus_now: HashSet::new(),
            autofocus_retry: &self.autofocus_retry,
            autofocus_retry_now: HashMap::new(),
            text_focus: &self.text_focus,
            text_focus_now: HashSet::new(),
            lost_reported: &self.lost_reported,
            lost_now: HashSet::new(),
        };
        render(
            &self.root,
            ui,
            theme,
            &mut runtime,
            &mut Access::Root(state),
        );
        let Runtime {
            changed,
            events,
            runtime_error,
            autofocus_now,
            autofocus_retry_now,
            text_focus_now,
            lost_now,
            rects_now,
            focus_within_now,
            ..
        } = runtime;
        self.rects = rects_now;
        self.focus_within = focus_within_now;
        self.runtime_error = runtime_error;
        if first {
            self.autofocused = autofocus_now;
            self.autofocus_retry = autofocus_retry_now;
            self.text_focus = text_focus_now;
            self.lost_reported = lost_now;
        }
        for event in &events {
            match event {
                Event::Root(action) => (self.actions[*action].handler)(state),
                Event::Row { lists, action, key } => {
                    let mut list = &mut self.lists[lists[0]];
                    for i in &lists[1..] {
                        list = &mut list.lists[*i];
                    }
                    (list.actions[*action].handler)(state, &**key)
                }
            }
        }
        if changed || !events.is_empty() {
            ui.ctx().request_repaint();
        }
        if self.error_window {
            let (title, error, hint) = match (&self.error, &self.runtime_error) {
                (Some(error), _) => (
                    "ECML template error",
                    error,
                    "Showing the last valid template. Save a correction to recover.",
                ),
                (None, Some(error)) => (
                    "ECML runtime error",
                    error,
                    "Only the first row with each key is shown.",
                ),
                (None, None) => return,
            };
            egui::Window::new(title)
                .id(self.instance.with("error"))
                .show(ui.ctx(), |ui| {
                    ui.colored_label(egui::Color32::LIGHT_RED, error.to_string());
                    ui.label(hint);
                });
        }
    }
}
enum Event {
    Root(usize),
    Row {
        lists: Rc<[usize]>,
        action: usize,
        key: Rc<dyn Any>,
    },
}
/// A model write inside a loop, committed in tree order once the outermost loop finishes.
enum Write {
    Root(usize, Value),
    /// `lists` and `keys` lead from the state to the row, one level at a time.
    Row {
        lists: Rc<[usize]>,
        keys: Vec<Rc<dyn Any>>,
        binding: usize,
        value: Value,
    },
}
/// One row of an enclosing loop. `lists` is the list's index path from the root lists;
/// `parent` is the frame of the row the list belongs to, for child lists.
#[derive(Clone)]
struct Frame<'s> {
    item: &'s dyn Any,
    key: RowKey,
    lists: Rc<[usize]>,
    parent: Option<usize>,
}
/// Outside loops the renderer may write state directly. Inside loops the lists'
/// iterators borrow the state, so writes are buffered and rows are read-only.
enum Access<'s, S> {
    Root(&'s mut S),
    Rows {
        state: &'s S,
        frames: Vec<Frame<'s>>,
    },
}
impl<S> Access<'_, S> {
    fn state(&self) -> &S {
        match self {
            Self::Root(state) => state,
            Self::Rows { state, .. } => state,
        }
    }
    fn frame(&self, depth: usize) -> &Frame<'_> {
        match self {
            Self::Rows { frames, .. } => &frames[depth],
            Self::Root(_) => unreachable!("row names resolve only inside loops"),
        }
    }
}
struct Runtime<'a, S> {
    bindings: &'a [Binding<S>],
    lists: &'a [List<S>],
    file: &'a str,
    first: bool,
    events: Vec<Event>,
    writes: Vec<Write>,
    instance: egui::Id,
    /// The identity outside loops and of each enclosing row, outermost first.
    root: egui::Id,
    rows: Vec<egui::Id>,
    ctx: egui::Context,
    rects: &'a HashMap<egui::Id, (egui::Rect, egui::LayerId)>,
    rects_now: HashMap<egui::Id, (egui::Rect, egui::LayerId)>,
    focus_within: &'a HashSet<egui::Id>,
    focus_within_now: HashSet<egui::Id>,
    /// Whether the node being drawn holds the focused control so far.
    focus_inside: bool,
    responses: &'a mut HashMap<String, egui::Response>,
    #[cfg(test)]
    row_responses: &'a mut HashMap<(String, egui::Id), egui::Response>,
    changed: bool,
    runtime_error: Option<Error>,
    /// Autofocus controls drawn on the previous frame, and those drawn on this one.
    autofocused: &'a HashSet<egui::Id>,
    autofocus_now: HashSet<egui::Id>,
    /// Autofocus controls that appeared on the previous frame (node -> widget id), whose
    /// focus request is repeated once before they are drawn; and those appearing now.
    autofocus_retry: &'a HashMap<egui::Id, egui::Id>,
    autofocus_retry_now: HashMap<egui::Id, egui::Id>,
    /// Text edits focused when last drawn, and those focused on this frame.
    text_focus: &'a HashSet<egui::Id>,
    text_focus_now: HashSet<egui::Id>,
    /// Text edits whose focus loss was reported on the previous frame, and on this one.
    lost_reported: &'a HashSet<egui::Id>,
    lost_now: HashSet<egui::Id>,
}
impl<S: 'static> Runtime<'_, S> {
    fn get(&self, access: &Access<'_, S>, r: Ref) -> Value {
        match r {
            Ref::Root(i) => (self.bindings[i].get)(access.state()),
            Ref::Row { depth, index } => {
                let frame = access.frame(depth);
                (list_at(self.lists, &frame.lists).bindings[index].get)(access.state(), frame.item)
            }
        }
    }
    fn write(&mut self, access: &mut Access<'_, S>, r: Ref, value: Value) {
        if !self.first {
            return;
        }
        self.changed = true;
        match (r, access) {
            (Ref::Root(i), Access::Root(state)) => {
                (self.bindings[i].set.as_ref().expect("validated model"))(state, value)
            }
            (Ref::Root(i), Access::Rows { .. }) => self.writes.push(Write::Root(i, value)),
            (Ref::Row { depth, index }, Access::Rows { frames, .. }) => {
                let mut keys = vec![];
                let mut at = Some(depth);
                while let Some(d) = at {
                    keys.push(frames[d].key.own.clone());
                    at = frames[d].parent;
                }
                keys.reverse();
                self.writes.push(Write::Row {
                    lists: frames[depth].lists.clone(),
                    keys,
                    binding: index,
                    value,
                })
            }
            (Ref::Row { .. }, Access::Root(_)) => {
                unreachable!("row names resolve only inside loops")
            }
        }
    }
    fn commit(&mut self, state: &mut S) {
        'writes: for write in std::mem::take(&mut self.writes) {
            match write {
                Write::Root(i, value) => {
                    (self.bindings[i].set.as_ref().expect("validated model"))(state, value)
                }
                Write::Row {
                    lists,
                    keys,
                    binding,
                    value,
                } => {
                    let mut target: &mut dyn Any = state;
                    let mut level = self.lists;
                    let mut list = None;
                    for (i, key) in lists.iter().zip(&keys) {
                        let l = &level[*i];
                        // A row removed since drawing simply drops the write.
                        match (l.find)(target, &**key) {
                            Some(row) => target = row,
                            None => continue 'writes,
                        }
                        level = &l.lists;
                        list = Some(l);
                    }
                    let list = list.expect("nonempty list path");
                    (list.bindings[binding]
                        .set
                        .as_ref()
                        .expect("validated model"))(target, value);
                }
            }
        }
    }
    /// Queues a handler to run after the traversal, in tree order.
    fn handle(&mut self, access: &Access<'_, S>, r: Ref) {
        self.events.push(match r {
            Ref::Root(i) => Event::Root(i),
            Ref::Row { depth, index } => {
                let frame = access.frame(depth);
                Event::Row {
                    lists: frame.lists.clone(),
                    action: index,
                    key: frame.key.path.clone(),
                }
            }
        });
    }
    fn boolean(&self, access: &Access<'_, S>, expr: &BoolExpr) -> bool {
        match expr {
            BoolExpr::Literal(v) => *v,
            BoolExpr::Binding(r, not) => matches!(self.get(access, *r), Value::Bool(true)) ^ not,
            BoolExpr::Contains {
                focus,
                depth,
                key,
                negate,
            } => {
                let scope = depth.map_or(self.root, |depth| self.rows[depth]).with(key);
                let inside = if *focus {
                    self.focus_within.contains(&scope)
                } else {
                    // Last show's area against this frame's pointer; nothing while unseen.
                    self.rects
                        .get(&scope)
                        .is_some_and(|(rect, layer)| self.ctx.rect_contains_pointer(*layer, *rect))
                };
                inside ^ negate
            }
            BoolExpr::Any(terms) => terms.iter().any(|t| self.boolean(access, t)),
            BoolExpr::All(terms) => terms.iter().all(|t| self.boolean(access, t)),
        }
    }
    fn value<T: Copy>(&self, access: &Access<'_, S>, expr: &Expr<T>) -> T {
        match expr {
            Expr::Const(v) => *v,
            Expr::If(condition, then, otherwise) => {
                if self.boolean(access, condition) {
                    *then
                } else {
                    self.value(access, otherwise)
                }
            }
        }
    }
    fn text(&self, access: &Access<'_, S>, parts: &[TextPart]) -> String {
        parts
            .iter()
            .map(|p| match p {
                TextPart::Literal(s) => s.clone(),
                TextPart::Binding(r) => self.get(access, *r).to_string(),
            })
            .collect()
    }
}
/// Draws `body` once per row of `list` for one parent; returns a duplicate key, if any.
#[allow(clippy::too_many_arguments)]
fn repeat<'s, S: 'static>(
    node: &Node,
    body: &Node,
    ui: &mut egui::Ui,
    theme: &Theme,
    rt: &mut Runtime<'_, S>,
    list: &List<S>,
    path: &Rc<[usize]>,
    parent: Option<usize>,
    state: &'s S,
    parent_item: &'s dyn Any,
    parent_path: Option<&dyn Any>,
    frames: &[Frame<'s>],
) -> Option<String> {
    let base = rt.instance;
    let duplicate = (list.rows)(state, parent_item, parent_path, &mut |item, key| {
        // Row identity: loop identity plus key, so rows keep state when moved.
        rt.instance = base.with(&node.key).with(key.id);
        rt.rows.push(rt.instance);
        let mut frames = frames.to_vec();
        frames.push(Frame {
            item,
            key,
            lists: path.clone(),
            parent,
        });
        render(body, ui, theme, rt, &mut Access::Rows { state, frames });
        rt.rows.pop();
    });
    rt.instance = base;
    duplicate
}
fn render<S: 'static>(
    node: &Node,
    ui: &mut egui::Ui,
    theme: &Theme,
    rt: &mut Runtime<'_, S>,
    access: &mut Access<'_, S>,
) {
    // Chains and fragments are transparent: they draw straight into the parent Ui.
    match &node.kind {
        Kind::If { branches } => {
            let chosen = branches.iter().find(|(condition, _)| {
                condition
                    .as_ref()
                    .is_none_or(|condition| rt.boolean(access, condition))
            });
            if let Some((_, branch)) = chosen {
                render(branch, ui, theme, rt, access);
            }
            return;
        }
        Kind::Fragment { children } => {
            let Some(visible) = &node.visible else {
                for child in children {
                    render(child, ui, theme, rt, access);
                }
                return;
            };
            // Hiding needs a scope, so it does not leak to later siblings.
            let visible = rt.boolean(access, visible);
            ui.scope(|ui| {
                if !visible {
                    ui.set_invisible();
                }
                for child in children {
                    render(child, ui, theme, rt, access);
                }
            });
            return;
        }
        _ => {}
    }
    let focus_before = std::mem::take(&mut rt.focus_inside);
    // Explicit scope_id keeps auto-allocated IDs inside existing components stable too.
    let scope = ui.scope_builder(
        egui::UiBuilder::new().scope_id(rt.instance.with(&node.key)),
        |ui| {
            if let Some(visible) = &node.visible
                && !rt.boolean(access, visible)
            {
                ui.set_invisible();
            }
            let response = match &node.kind {
                Kind::If { .. } | Kind::Fragment { .. } => unreachable!("drawn above"),
                Kind::For {
                    list,
                    body,
                    line,
                    column,
                } => {
                    let lists = rt.lists;
                    let (path, parent): (Rc<[usize]>, _) = match *list {
                        Ref::Root(i) => (Rc::from([i]), None),
                        Ref::Row { depth, index } => {
                            let mut path = access.frame(depth).lists.to_vec();
                            path.push(index);
                            (path.into(), Some(depth))
                        }
                    };
                    let target = list_at(lists, &path);
                    let duplicate = match access {
                        Access::Root(state) => {
                            let shared: &S = state;
                            let duplicate = repeat(
                                node,
                                body,
                                ui,
                                theme,
                                rt,
                                target,
                                &path,
                                None,
                                shared,
                                shared,
                                None,
                                &[],
                            );
                            // Only the outermost loop can write; inner loops buffer into it.
                            rt.commit(state);
                            duplicate
                        }
                        Access::Rows { state, frames } => {
                            let state: &S = state;
                            let parent_frame = &frames[parent.expect("child list")];
                            repeat(
                                node,
                                body,
                                ui,
                                theme,
                                rt,
                                target,
                                &path,
                                parent,
                                state,
                                parent_frame.item,
                                Some(&*parent_frame.key.path),
                                frames,
                            )
                        }
                    };
                    if let Some(key) = duplicate
                        && rt.runtime_error.is_none()
                    {
                        rt.runtime_error = Some(Error {
                            file: rt.file.into(),
                            line: *line,
                            column: *column,
                            message: format!(
                                "list '{}' has duplicate key {key}; later rows with it are skipped",
                                target.name
                            ),
                        });
                    }
                    None
                }
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
                            render(child, ui, theme, rt, access);
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
                    size,
                    font_weight,
                    color,
                    underline,
                    strikethrough,
                    truncate,
                    selectable,
                    click,
                    double_click,
                } => {
                    let sizes = &theme.text_size;
                    let mut style = match rt.value(access, size) {
                        TextSize::Heading => text::Style::new_heading(theme),
                        TextSize::Xxs => text::Style::new(theme).size(sizes.xxs),
                        TextSize::Xs => text::Style::new(theme).size(sizes.xs),
                        TextSize::Sm => text::Style::new(theme).size(sizes.sm),
                        TextSize::Md => text::Style::new(theme),
                        TextSize::Lg => text::Style::new(theme).size(sizes.lg),
                        TextSize::Xl => text::Style::new(theme).size(sizes.xl),
                    };
                    if let Some(weight) = font_weight {
                        style = style.font_weight(rt.value(access, weight));
                    }
                    style = style.truncate(rt.boolean(access, truncate));
                    if let Some(color) = color {
                        style = style.color(rt.value(access, color).get(theme));
                    }
                    // Lines take the text's color, as egui's RichText decorations do.
                    let line = egui::Stroke::new(1.0, style.color);
                    if rt.boolean(access, underline) {
                        style = style.underline(line);
                    }
                    if rt.boolean(access, strikethrough) {
                        style = style.strikethrough(line);
                    }
                    let mut options = text::Options::default();
                    if let Some(selectable) = selectable {
                        options = options.selectable(rt.boolean(access, selectable));
                    }
                    if click.is_some() || double_click.is_some() {
                        // A double click would otherwise also select a word.
                        options = options.selectable(false).sense(egui::Sense::click());
                    }
                    let response = text::render(ui, &rt.text(access, parts), style, options);
                    if rt.first {
                        // egui reports the second release of a double click as a click too,
                        // as browsers do; the click handler runs first.
                        if let (true, Some(click)) = (response.clicked(), click) {
                            rt.handle(access, *click);
                        }
                        if let (true, Some(double_click)) =
                            (response.double_clicked(), double_click)
                        {
                            rt.handle(access, *double_click);
                        }
                    }
                    Some(response)
                }
                Kind::Button {
                    text,
                    size,
                    variant,
                    color,
                    width,
                    disabled,
                    loading,
                    aria_label,
                    click,
                } => {
                    let style = button::Style::new(
                        theme,
                        color.as_ref().map(|c| *(rt.value(access, c).0)(theme)),
                        rt.value(access, size),
                        rt.value(access, variant),
                        button::ButtonIcon::None,
                        button::ButtonIcon::None,
                    )
                    .width(*width);
                    let aria_label = aria_label.as_ref().map(|label| rt.text(access, label));
                    let options = button::Options {
                        // Disabled wins over loading: neither can be activated, but only a
                        // loading button keeps focus.
                        state: if rt.boolean(access, disabled) {
                            button::ButtonState::Disable
                        } else if rt.boolean(access, loading) {
                            button::ButtonState::Loading
                        } else {
                            button::ButtonState::None
                        },
                        aria_label: aria_label.as_deref(),
                    };
                    let response = button::render(ui, &rt.text(access, text), style, options);
                    if rt.first
                        && response.clicked()
                        && let Some(click) = click
                    {
                        rt.handle(access, *click);
                    }
                    Some(response)
                }
                Kind::TextEdit {
                    model,
                    size,
                    variant,
                    color,
                    width,
                    placeholder,
                    disabled,
                    read_only,
                    input_type,
                    password_toggle_aria_label,
                    autofocus,
                    submit,
                    cancel,
                    blur,
                } => {
                    let Value::String(mut value) = rt.get(access, *model) else {
                        unreachable!()
                    };
                    let disabled = rt.boolean(access, disabled);
                    // Repeat the focus request of the frame it appeared in, before drawing:
                    // on the web a field that draws unfocused can lose a late request.
                    let node_id = rt.instance.with(&node.key);
                    if rt.first
                        && !disabled
                        && let Some(widget) = rt.autofocus_retry.get(&node_id)
                    {
                        ui.memory_mut(|m| m.request_focus(*widget));
                    }
                    let style = text_edit::Style::new(
                        theme,
                        color.as_ref().map(|c| *(rt.value(access, c).0)(theme)),
                        rt.value(access, size),
                        rt.value(access, variant),
                    )
                    .width(*width);
                    let read_only = rt.boolean(access, read_only);
                    let placeholder = rt.text(access, placeholder);
                    let toggle_label = password_toggle_aria_label
                        .as_ref()
                        .map(|label| rt.text(access, label));
                    let mut options = text_edit::Options::default()
                        .placeholder(&placeholder)
                        .input_type(rt.value(access, input_type))
                        .disabled(disabled)
                        // Later passes of a frame must not apply the same typing again.
                        .read_only(read_only || !rt.first);
                    if let Some(label) = &toggle_label {
                        options = options.password_toggle_aria_label(label);
                    }
                    let response = text_edit::render(ui, &mut value, style, options);
                    if response.changed() {
                        rt.write(access, *model, Value::String(value));
                    }
                    if rt.first && !disabled {
                        if *autofocus {
                            // Focus only when the control appears, not on every frame.
                            if !rt.autofocused.contains(&response.id) {
                                response.request_focus();
                                rt.autofocus_retry_now.insert(node_id, response.id);
                            }
                            rt.autofocus_now.insert(response.id);
                        }
                        // egui keeps lost_focus() true for two frames, so with one pass per
                        // frame one loss would fire twice. Skip a loss already reported on
                        // the previous frame, unless the field was focused again since.
                        let id = response.id;
                        if response.has_focus() {
                            rt.text_focus_now.insert(id);
                        }
                        if response.lost_focus()
                            && (rt.text_focus.contains(&id) || !rt.lost_reported.contains(&id))
                        {
                            rt.lost_now.insert(id);
                            // egui ends single-line editing on Enter and Escape; each focus
                            // loss fires exactly one handler.
                            let (enter, escape) = ui.input(|i| {
                                (
                                    i.key_pressed(egui::Key::Enter),
                                    i.key_pressed(egui::Key::Escape),
                                )
                            });
                            let handler = match (enter, escape) {
                                (true, _) if submit.is_some() => {
                                    // Keep typing in an entry field after submitting.
                                    response.request_focus();
                                    rt.text_focus_now.insert(response.id);
                                    *submit
                                }
                                (false, true) if cancel.is_some() => *cancel,
                                _ => *blur,
                            };
                            // A read-only field reports nothing, like a disabled one.
                            if let (false, Some(handler)) = (read_only, handler) {
                                rt.handle(access, handler);
                            }
                        }
                    }
                    Some(response)
                }
                Kind::Switch {
                    model,
                    size,
                    color,
                    disabled,
                    aria_label,
                } => {
                    let Value::Bool(mut value) = rt.get(access, *model) else {
                        unreachable!()
                    };
                    let color = color
                        .as_ref()
                        .map_or(theme.colors.primary, |c| *(rt.value(access, c).0)(theme));
                    let style = match rt.value(access, size) {
                        "sm" => switch::Style::new_sm_color(theme, color),
                        "lg" => switch::Style::new_lg_color(theme, color),
                        _ => switch::Style::new_md_color(theme, color),
                    };
                    let aria_label = aria_label.as_ref().map(|label| rt.text(access, label));
                    let options = switch::Options {
                        disabled: rt.boolean(access, disabled),
                        aria_label: aria_label.as_deref(),
                    };
                    let response = switch::render(ui, &mut value, style, options);
                    if response.changed() {
                        rt.write(access, *model, Value::Bool(value));
                    }
                    Some(response)
                }
                Kind::Checkbox {
                    model,
                    label,
                    size,
                    color,
                    disabled,
                    indeterminate,
                    aria_label,
                } => {
                    let Value::Bool(mut value) = rt.get(access, *model) else {
                        unreachable!()
                    };
                    let color = color
                        .as_ref()
                        .map_or(theme.colors.primary, |c| *(rt.value(access, c).0)(theme));
                    let style = match rt.value(access, size) {
                        "sm" => checkbox::Style::new_sm_color(theme, color),
                        "lg" => checkbox::Style::new_lg_color(theme, color),
                        _ => checkbox::Style::new_md_color(theme, color),
                    };
                    let aria_label = aria_label.as_ref().map(|label| rt.text(access, label));
                    let options = checkbox::Options {
                        disabled: rt.boolean(access, disabled),
                        indeterminate: rt.boolean(access, indeterminate),
                        aria_label: aria_label.as_deref(),
                    };
                    let label = rt.text(access, label);
                    let response = if label.is_empty() {
                        checkbox::render(ui, &mut value, style, options)
                    } else {
                        checkbox::render_with_label(ui, &mut value, &label, style, options)
                    };
                    if response.changed() {
                        rt.write(access, *model, Value::Bool(value));
                    }
                    Some(response)
                }
            };
            if response.as_ref().is_some_and(egui::Response::has_focus) {
                rt.focus_inside = true;
            }
            if let (Some(id), Some(response)) = (&node.explicit_id, response) {
                match access {
                    Access::Root(_) => {
                        rt.responses.insert(id.clone(), response);
                    }
                    // Keyed by the innermost row's own key; tests keep keys distinct.
                    #[cfg(test)]
                    Access::Rows { frames, .. } => {
                        let key = frames.last().expect("rows have a frame").key.id;
                        rt.row_responses.insert((id.clone(), key), response);
                    }
                    #[cfg(not(test))]
                    Access::Rows { .. } => {}
                }
            }
        },
    );
    // A loop's element records each row's area itself, under the row's identity.
    if node.explicit_id.is_some() && !matches!(node.kind, Kind::For { .. }) {
        let id = rt.instance.with(&node.key);
        rt.rects_now
            .insert(id, (scope.response.rect, ui.layer_id()));
        if rt.focus_inside {
            rt.focus_within_now.insert(id);
        }
    }
    rt.focus_inside |= focus_before;
}
