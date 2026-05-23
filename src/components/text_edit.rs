use egui::util::undoer;
use egui::{Color32, CornerRadius, CursorIcon, Stroke, StrokeKind, Ui};

use crate::{
    components::icon_button,
    consts::FONT_MANAGER,
    theme::{Color, TextEditPalette, TextEditSize, TextEditVariant, Theme},
};

#[derive(Clone, PartialEq, Default)]
pub enum InputType {
    #[default]
    Text,
    Password,
}

#[derive(Clone)]
pub struct Style {
    pub padding_x: f32,
    pub height: f32,
    pub corner_radius: CornerRadius,
    pub text_size: f32,
    pub palette: TextEditPalette,
    /// placeholder 文字
    pub placeholder: String,
    /// 是否强制固定宽度（None = 占满可用宽度）
    pub width: Option<f32>,

    pub variant: TextEditVariant,
    pub input_type: InputType,
    pub icon_button_style: icon_button::Style,
}

pub const TEXT_EDIT_HEIGHT_XS: f32 = 18.0;
pub const TEXT_EDIT_HEIGHT_SM: f32 = 24.0;
pub const TEXT_EDIT_HEIGHT_MD: f32 = 32.0;
pub const TEXT_EDIT_HEIGHT_LG: f32 = 40.0;

impl Style {
    pub fn new(
        theme: &Theme,
        color: Option<Color>,
        size: TextEditSize,
        variant: TextEditVariant,
    ) -> Self {
        let (height, padding_x, text_size, corner_radius) = match size {
            TextEditSize::Xs => (
                TEXT_EDIT_HEIGHT_XS,
                8.0,
                10.0,
                CornerRadius::same(theme.corner_radius.xs as u8),
            ),
            TextEditSize::Sm => (
                TEXT_EDIT_HEIGHT_SM,
                10.0,
                12.0,
                CornerRadius::same(theme.corner_radius.xs as u8),
            ),
            TextEditSize::Md => (
                TEXT_EDIT_HEIGHT_MD,
                12.0,
                14.0,
                CornerRadius::same(theme.corner_radius.md as u8),
            ),
            TextEditSize::Lg => (
                TEXT_EDIT_HEIGHT_LG,
                16.0,
                16.0,
                CornerRadius::same(theme.corner_radius.md as u8),
            ),
        };

        // Flushed 变体不需要圆角和水平内边距
        let (corner_radius, padding_x) = if variant == TextEditVariant::Flushed {
            (CornerRadius::ZERO, 0.0)
        } else {
            (corner_radius, padding_x)
        };

        let button_size = match size {
            TextEditSize::Xs => crate::theme::ButtonSize::Xs,
            TextEditSize::Sm => crate::theme::ButtonSize::Xs,
            TextEditSize::Md => crate::theme::ButtonSize::Sm,
            TextEditSize::Lg => crate::theme::ButtonSize::Md,
        };
        let icon_button_style = icon_button::Style::new(
            theme,
            color.clone(),
            button_size,
            crate::theme::ButtonVariant::Ghost,
        );

        let palette = match_text_edit_palette(theme, color, &variant);
        Self {
            height,
            padding_x,
            text_size,
            corner_radius,
            palette,
            placeholder: String::new(),
            width: None,
            variant,
            input_type: InputType::Text,
            icon_button_style,
        }
    }

    // ── Subtle 便捷构造 ──

    pub fn new_subtle_xs(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Xs, TextEditVariant::Subtle)
    }

    pub fn new_subtle_xs_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Xs,
            TextEditVariant::Subtle,
        )
    }

    pub fn new_subtle_sm(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Sm, TextEditVariant::Subtle)
    }

    pub fn new_subtle_sm_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Sm,
            TextEditVariant::Subtle,
        )
    }

    pub fn new_subtle_md(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Md, TextEditVariant::Subtle)
    }

    pub fn new_subtle_md_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Md,
            TextEditVariant::Subtle,
        )
    }

    pub fn new_subtle_lg(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Lg, TextEditVariant::Subtle)
    }

    pub fn new_subtle_lg_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Lg,
            TextEditVariant::Subtle,
        )
    }

    // ── Outline 便捷构造 ──

    pub fn new_outline_xs(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Xs, TextEditVariant::Outline)
    }

    pub fn new_outline_xs_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Xs,
            TextEditVariant::Outline,
        )
    }

    pub fn new_outline_sm(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Sm, TextEditVariant::Outline)
    }

    pub fn new_outline_sm_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Sm,
            TextEditVariant::Outline,
        )
    }

    pub fn new_outline_md(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Md, TextEditVariant::Outline)
    }

    pub fn new_outline_md_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Md,
            TextEditVariant::Outline,
        )
    }

    pub fn new_outline_lg(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Lg, TextEditVariant::Outline)
    }

    pub fn new_outline_lg_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Lg,
            TextEditVariant::Outline,
        )
    }

    // ── Flushed 便捷构造 ──

    pub fn new_flushed_xs(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Xs, TextEditVariant::Flushed)
    }

    pub fn new_flushed_xs_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Xs,
            TextEditVariant::Flushed,
        )
    }

    pub fn new_flushed_sm(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Sm, TextEditVariant::Flushed)
    }

    pub fn new_flushed_sm_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Sm,
            TextEditVariant::Flushed,
        )
    }

    pub fn new_flushed_md(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Md, TextEditVariant::Flushed)
    }

    pub fn new_flushed_md_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Md,
            TextEditVariant::Flushed,
        )
    }

    pub fn new_flushed_lg(theme: &Theme) -> Self {
        Self::new(theme, None, TextEditSize::Lg, TextEditVariant::Flushed)
    }

    pub fn new_flushed_lg_color(theme: &Theme, color: Color) -> Self {
        Self::new(
            theme,
            Some(color),
            TextEditSize::Lg,
            TextEditVariant::Flushed,
        )
    }

    // ── Builder setters ──

    pub fn padding_x(mut self, padding_x: f32) -> Self {
        self.padding_x = padding_x;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn corner_radius(mut self, corner_radius: CornerRadius) -> Self {
        self.corner_radius = corner_radius;
        self
    }

    pub fn text_size(mut self, text_size: f32) -> Self {
        self.text_size = text_size;
        self
    }

    pub fn palette(mut self, palette: TextEditPalette) -> Self {
        self.palette = palette;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn width(mut self, width: Option<f32>) -> Self {
        self.width = width;
        self
    }

    pub fn input_type(mut self, input_type: InputType) -> Self {
        self.input_type = input_type;
        self
    }
}

/// A snapshot of the text + cursor for undo/redo.
#[derive(Clone)]
struct UndoEntry {
    text: String,
    cursor: usize,
    anchor: usize,
}

impl PartialEq for UndoEntry {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// Cursor state — persisted across frames via egui temp storage
// ────────────────────────────────────────────────────────────────────────────────

/// Per-widget cursor state for the custom singleline TextEdit.
/// Stored in `ui.data_mut()` keyed by the widget's `Id`.
#[derive(Clone)]
struct CursorState {
    /// Cursor position (character index, 0 = before first char).
    cursor: usize,
    /// Selection anchor. When `anchor != cursor`, the range `[min, max)` is selected.
    anchor: usize,
    /// Horizontal scroll offset (pixels) for text wider than the visible area.
    scroll_offset: f32,
    /// Timestamp of the last user interaction — controls cursor blink.
    last_interaction_time: f64,
    /// True while the OS IME composition session is active (Enabled → Disabled/Commit).
    ime_enabled: bool,
    /// IME preedit text currently shown inline. Empty = not in IME composition.
    ime_preedit: String,
    /// Char index where the preedit text starts (for replacement on Commit).
    ime_start: usize,
    /// Undo/redo history.
    undoer: undoer::Undoer<UndoEntry>,
    /// Whether the password is visible (only used if input_type == Password)
    password_visible: bool,
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            cursor: 0,
            anchor: 0,
            scroll_offset: 0.0,
            last_interaction_time: f64::NEG_INFINITY,
            ime_enabled: false,
            ime_preedit: String::new(),
            ime_start: 0,
            undoer: undoer::Undoer::default(),
            password_visible: false,
        }
    }
}

impl CursorState {
    #[inline]
    fn has_selection(&self) -> bool {
        self.cursor != self.anchor
    }

    #[inline]
    fn ime_active(&self) -> bool {
        self.ime_enabled || !self.ime_preedit.is_empty()
    }

    /// Returns `(start, end)` in ascending order (character indices).
    #[inline]
    fn ordered_range(&self) -> (usize, usize) {
        if self.cursor <= self.anchor {
            (self.cursor, self.anchor)
        } else {
            (self.anchor, self.cursor)
        }
    }

    /// Move cursor to `pos` and collapse selection.
    #[inline]
    fn set_cursor(&mut self, pos: usize) {
        self.cursor = pos;
        self.anchor = pos;
    }

    /// Select all text.
    fn select_all(&mut self, char_count: usize) {
        self.anchor = 0;
        self.cursor = char_count;
    }

    /// Clamp cursor and anchor to `[0, char_count]`.
    fn clamp(&mut self, char_count: usize) {
        self.cursor = self.cursor.min(char_count);
        self.anchor = self.anchor.min(char_count);
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// Text helpers (character-index based)
// ────────────────────────────────────────────────────────────────────────────────

/// Convert a **char index** to a **byte offset** in a UTF-8 string.
fn char_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices().nth(char_idx).map_or(s.len(), |(i, _)| i)
}

/// Delete `[start_char, end_char)` from the string and return the deleted text.
fn delete_range(s: &mut String, start_char: usize, end_char: usize) -> String {
    let start = char_to_byte(s, start_char);
    let end = char_to_byte(s, end_char);
    let removed: String = s[start..end].to_owned();
    s.drain(start..end);
    removed
}

/// Insert `insert` at the given char position.
fn insert_at(s: &mut String, char_pos: usize, insert: &str) {
    let byte = char_to_byte(s, char_pos);
    s.insert_str(byte, insert);
}

/// Extract `[start_char, end_char)` as a new String.
fn slice_chars(s: &str, start_char: usize, end_char: usize) -> String {
    s.chars()
        .skip(start_char)
        .take(end_char - start_char)
        .collect()
}

/// Find the start of the previous word (for Ctrl+Left / Ctrl+Backspace).
fn prev_word_boundary(s: &str, from: usize) -> usize {
    let chars: Vec<char> = s.chars().collect();
    if from == 0 {
        return 0;
    }
    let mut i = from;
    // skip trailing whitespace
    while i > 0 && chars[i - 1].is_whitespace() {
        i -= 1;
    }
    // skip word characters
    while i > 0 && !chars[i - 1].is_whitespace() {
        i -= 1;
    }
    i
}

/// Find the end of the next word (for Ctrl+Right / Ctrl+Delete).
fn next_word_boundary(s: &str, from: usize) -> usize {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    if from >= len {
        return len;
    }
    let mut i = from;
    // skip current word characters
    while i < len && !chars[i].is_whitespace() {
        i += 1;
    }
    // skip whitespace
    while i < len && chars[i].is_whitespace() {
        i += 1;
    }
    i
}

/// Find the word boundaries around `pos` (for double-click word selection).
fn word_range_at(s: &str, pos: usize) -> (usize, usize) {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    if len == 0 {
        return (0, 0);
    }
    let pos = pos.min(len.saturating_sub(1));
    let is_word = !chars[pos].is_whitespace();

    let mut start = pos;
    let mut end = pos;

    if is_word {
        while start > 0 && !chars[start - 1].is_whitespace() {
            start -= 1;
        }
        while end < len && !chars[end].is_whitespace() {
            end += 1;
        }
    } else {
        while start > 0 && chars[start - 1].is_whitespace() {
            start -= 1;
        }
        while end < len && chars[end].is_whitespace() {
            end += 1;
        }
    }
    (start, end)
}

/// Smart cursor blink: solid for 0.5 s after interaction, then 1 Hz blink.
fn should_show_cursor(time: f64, last_interaction: f64) -> bool {
    let elapsed = time - last_interaction;
    if elapsed < 0.5 {
        return true;
    }
    ((elapsed - 0.5) % 1.0) < 0.5
}

// ────────────────────────────────────────────────────────────────────────────────
// Event handling — modelled after egui's `events()` + `check_for_mutating_key_press()`
// ────────────────────────────────────────────────────────────────────────────────

fn handle_events(
    ui: &Ui,
    id: egui::Id,
    text: &mut String,
    state: &mut CursorState,
    event_filter: &egui::EventFilter,
) -> bool {
    let events: Vec<egui::Event> = ui.input(|i| i.filtered_events(event_filter));
    let now = ui.input(|i| i.time);
    let char_count = text.chars().count();
    let mut changed = false;

    // Feed current state to the undoer before processing events.
    let current_undo_state = UndoEntry {
        text: text.clone(),
        cursor: state.cursor,
        anchor: state.anchor,
    };
    state.undoer.feed_state(now, &current_undo_state);

    for event in &events {
        match event {
            // ── Cursor movement (Arrow / Home / End) ──────────────
            // Arrow keys move the text cursor only when no preedit is showing.
            // When an active preedit exists the OS IME uses these keys for candidate
            // navigation; letting our handler also run would conflict with the
            // subsequent ImeEvent::Preedit that resets the cursor position.
            egui::Event::Key {
                key: egui::Key::ArrowLeft,
                pressed: true,
                modifiers,
                ..
            } if !state.ime_active() => {
                state.last_interaction_time = now;
                if modifiers.mac_cmd {
                    // Cmd+Left (macOS) → line start
                    state.cursor = 0;
                } else if modifiers.alt || modifiers.ctrl {
                    // Alt (macOS) / Ctrl (Win) → word boundary
                    state.cursor = prev_word_boundary(text, state.cursor);
                } else if !modifiers.shift && state.has_selection() {
                    // collapse selection to left edge
                    let (start, _) = state.ordered_range();
                    state.cursor = start;
                } else {
                    state.cursor = state.cursor.saturating_sub(1);
                }
                if !modifiers.shift {
                    state.anchor = state.cursor;
                }
            }

            egui::Event::Key {
                key: egui::Key::ArrowRight,
                pressed: true,
                modifiers,
                ..
            } if !state.ime_active() => {
                state.last_interaction_time = now;
                let cc = text.chars().count();
                if modifiers.mac_cmd {
                    state.cursor = cc;
                } else if modifiers.alt || modifiers.ctrl {
                    state.cursor = next_word_boundary(text, state.cursor);
                } else if !modifiers.shift && state.has_selection() {
                    let (_, end) = state.ordered_range();
                    state.cursor = end;
                } else {
                    state.cursor = (state.cursor + 1).min(cc);
                }
                if !modifiers.shift {
                    state.anchor = state.cursor;
                }
            }

            // Singleline: ArrowUp → beginning, ArrowDown → end (like Home/End)
            egui::Event::Key {
                key: egui::Key::ArrowUp,
                pressed: true,
                modifiers,
                ..
            } if !state.ime_active() => {
                state.last_interaction_time = now;
                state.cursor = 0;
                if !modifiers.shift {
                    state.anchor = state.cursor;
                }
            }

            egui::Event::Key {
                key: egui::Key::ArrowDown,
                pressed: true,
                modifiers,
                ..
            } if !state.ime_active() => {
                state.last_interaction_time = now;
                state.cursor = text.chars().count();
                if !modifiers.shift {
                    state.anchor = state.cursor;
                }
            }

            egui::Event::Key {
                key: egui::Key::Home,
                pressed: true,
                modifiers,
                ..
            } => {
                state.last_interaction_time = now;
                state.cursor = 0;
                if !modifiers.shift {
                    state.anchor = state.cursor;
                }
            }

            egui::Event::Key {
                key: egui::Key::End,
                pressed: true,
                modifiers,
                ..
            } => {
                state.last_interaction_time = now;
                state.cursor = text.chars().count();
                if !modifiers.shift {
                    state.anchor = state.cursor;
                }
            }

            // ── Undo (Ctrl+Z) ─────────────────────────────────────
            egui::Event::Key {
                key: egui::Key::Z,
                pressed: true,
                modifiers,
                ..
            } if modifiers.command && !modifiers.shift => {
                state.last_interaction_time = now;
                let current_undo_state = UndoEntry {
                    text: text.clone(),
                    cursor: state.cursor,
                    anchor: state.anchor,
                };
                if let Some(entry) = state.undoer.undo(&current_undo_state) {
                    let entry = entry.clone();
                    *text = entry.text;
                    state.cursor = entry.cursor;
                    state.anchor = entry.anchor;
                    state.clamp(text.chars().count());
                    changed = true;
                }
            }

            // ── Redo (Ctrl+Shift+Z or Ctrl+Y) ───────────────────
            egui::Event::Key {
                key: egui::Key::Z,
                pressed: true,
                modifiers,
                ..
            } if modifiers.command && modifiers.shift => {
                state.last_interaction_time = now;
                let current_undo_state = UndoEntry {
                    text: text.clone(),
                    cursor: state.cursor,
                    anchor: state.anchor,
                };
                if let Some(entry) = state.undoer.redo(&current_undo_state) {
                    let entry = entry.clone();
                    *text = entry.text;
                    state.cursor = entry.cursor;
                    state.anchor = entry.anchor;
                    state.clamp(text.chars().count());
                    changed = true;
                }
            }

            egui::Event::Key {
                key: egui::Key::Y,
                pressed: true,
                modifiers,
                ..
            } if modifiers.command => {
                state.last_interaction_time = now;
                let current_undo_state = UndoEntry {
                    text: text.clone(),
                    cursor: state.cursor,
                    anchor: state.anchor,
                };
                if let Some(entry) = state.undoer.redo(&current_undo_state) {
                    let entry = entry.clone();
                    *text = entry.text;
                    state.cursor = entry.cursor;
                    state.anchor = entry.anchor;
                    state.clamp(text.chars().count());
                    changed = true;
                }
            }

            // ── Select All ────────────────────────────────────────
            egui::Event::Key {
                key: egui::Key::A,
                pressed: true,
                modifiers,
                ..
            } if modifiers.command => {
                state.last_interaction_time = now;
                state.select_all(char_count);
            }

            // ── Clipboard events (high-level, from backend) ──────
            egui::Event::Copy => {
                if state.has_selection() {
                    let (s, e) = state.ordered_range();
                    ui.ctx().copy_text(slice_chars(text, s, e));
                }
            }

            egui::Event::Cut => {
                if state.has_selection() {
                    state.last_interaction_time = now;
                    let (s, e) = state.ordered_range();
                    let cut = delete_range(text, s, e);
                    ui.ctx().copy_text(cut);
                    state.set_cursor(s);
                    changed = true;
                }
            }

            egui::Event::Paste(paste_text) => {
                if !paste_text.is_empty() {
                    state.last_interaction_time = now;
                    // filter newlines for singleline
                    let filtered: String = paste_text
                        .chars()
                        .filter(|c| *c != '\n' && *c != '\r')
                        .collect();
                    if !filtered.is_empty() {
                        if state.has_selection() {
                            let (s, e) = state.ordered_range();
                            delete_range(text, s, e);
                            state.set_cursor(s);
                        }
                        insert_at(text, state.cursor, &filtered);
                        state.set_cursor(state.cursor + filtered.chars().count());
                        changed = true;
                    }
                }
            }

            // ── Text input (character insertion) ──────────────────
            // Skip Text events during IME composition — ImeEvent handles those.
            egui::Event::Text(t)
                if !t.is_empty() && t != "\n" && t != "\r" && !state.ime_active() =>
            {
                state.last_interaction_time = now;
                let filtered: String = t.chars().filter(|c| *c != '\n' && *c != '\r').collect();
                if !filtered.is_empty() {
                    if state.has_selection() {
                        let (s, e) = state.ordered_range();
                        delete_range(text, s, e);
                        state.set_cursor(s);
                    }
                    insert_at(text, state.cursor, &filtered);
                    state.set_cursor(state.cursor + filtered.chars().count());
                    changed = true;
                }
            }

            // ── Backspace ─────────────────────────────────────────
            egui::Event::Key {
                key: egui::Key::Backspace,
                pressed: true,
                modifiers,
                ..
            } => {
                state.last_interaction_time = now;
                if state.has_selection() {
                    let (s, e) = state.ordered_range();
                    delete_range(text, s, e);
                    state.set_cursor(s);
                    changed = true;
                } else if state.cursor > 0 {
                    if modifiers.mac_cmd {
                        // Cmd+Backspace → delete to line start
                        delete_range(text, 0, state.cursor);
                        state.set_cursor(0);
                    } else if modifiers.alt || modifiers.ctrl {
                        // word delete
                        let boundary = prev_word_boundary(text, state.cursor);
                        delete_range(text, boundary, state.cursor);
                        state.set_cursor(boundary);
                    } else {
                        delete_range(text, state.cursor - 1, state.cursor);
                        state.set_cursor(state.cursor - 1);
                    }
                    changed = true;
                }
            }

            // ── Delete ────────────────────────────────────────────
            egui::Event::Key {
                key: egui::Key::Delete,
                pressed: true,
                modifiers,
                ..
            } => {
                state.last_interaction_time = now;
                let cc = text.chars().count();
                if state.has_selection() {
                    let (s, e) = state.ordered_range();
                    delete_range(text, s, e);
                    state.set_cursor(s);
                    changed = true;
                } else if state.cursor < cc {
                    if modifiers.mac_cmd {
                        // Cmd+Delete → delete to line end
                        delete_range(text, state.cursor, cc);
                    } else if modifiers.alt || modifiers.ctrl {
                        let boundary = next_word_boundary(text, state.cursor);
                        delete_range(text, state.cursor, boundary);
                    } else {
                        delete_range(text, state.cursor, state.cursor + 1);
                    }
                    changed = true;
                }
            }

            // ── Emacs-style bindings (Ctrl+H/K/U/W) ──────────────
            egui::Event::Key {
                key: egui::Key::H,
                pressed: true,
                modifiers,
                ..
            } if modifiers.ctrl => {
                // Ctrl+H → same as Backspace
                state.last_interaction_time = now;
                if state.has_selection() {
                    let (s, e) = state.ordered_range();
                    delete_range(text, s, e);
                    state.set_cursor(s);
                    changed = true;
                } else if state.cursor > 0 {
                    delete_range(text, state.cursor - 1, state.cursor);
                    state.set_cursor(state.cursor - 1);
                    changed = true;
                }
            }

            egui::Event::Key {
                key: egui::Key::K,
                pressed: true,
                modifiers,
                ..
            } if modifiers.ctrl => {
                // Ctrl+K → delete to end of line
                state.last_interaction_time = now;
                let cc = text.chars().count();
                if state.cursor < cc {
                    delete_range(text, state.cursor, cc);
                    changed = true;
                }
            }

            egui::Event::Key {
                key: egui::Key::U,
                pressed: true,
                modifiers,
                ..
            } if modifiers.ctrl => {
                // Ctrl+U → delete to start of line
                state.last_interaction_time = now;
                if state.cursor > 0 {
                    delete_range(text, 0, state.cursor);
                    state.set_cursor(0);
                    changed = true;
                }
            }

            egui::Event::Key {
                key: egui::Key::W,
                pressed: true,
                modifiers,
                ..
            } if modifiers.ctrl => {
                // Ctrl+W → delete previous word
                state.last_interaction_time = now;
                if state.has_selection() {
                    let (s, e) = state.ordered_range();
                    delete_range(text, s, e);
                    state.set_cursor(s);
                    changed = true;
                } else if state.cursor > 0 {
                    let boundary = prev_word_boundary(text, state.cursor);
                    delete_range(text, boundary, state.cursor);
                    state.set_cursor(boundary);
                    changed = true;
                }
            }

            // ── IME composition ───────────────────────────────────
            egui::Event::Ime(ime_event) => {
                match ime_event {
                    egui::ImeEvent::Enabled => {
                        // Begin IME composition: remember where preedit starts.
                        // First delete any existing selection.
                        if state.has_selection() {
                            let (s, e) = state.ordered_range();
                            delete_range(text, s, e);
                            state.set_cursor(s);
                            changed = true;
                        }
                        state.ime_enabled = true;
                        state.ime_start = state.cursor;
                        state.ime_preedit = String::new();
                    }
                    egui::ImeEvent::Preedit(preedit) => {
                        if preedit == "\n" || preedit == "\r" {
                            // ignore
                        } else {
                            // Remove old preedit text from the buffer.
                            let old_len = state.ime_preedit.chars().count();
                            if old_len > 0 {
                                let preedit_end = state.ime_start + old_len;
                                delete_range(text, state.ime_start, preedit_end);
                                changed = true;
                            }
                            state.ime_preedit = preedit.clone();

                            if !preedit.is_empty() {
                                // Insert new preedit text and move cursor to its end.
                                state.last_interaction_time = now;
                                insert_at(text, state.ime_start, preedit);
                                state.set_cursor(state.ime_start + preedit.chars().count());
                                changed = true;
                            } else {
                                // Empty preedit: the IME is clearing composition state
                                // (e.g. after a Commit, or when the user presses Escape
                                // inside the IME). We deleted any stale preedit text above;
                                // do NOT reset the cursor — an ArrowLeft/Right event may
                                // have just moved it and must not be overridden here.
                                if old_len > 0 {
                                    // Cursor was inside the now-deleted preedit range;
                                    // bring it back to the insertion point.
                                    state.set_cursor(state.ime_start);
                                }
                                // If old_len == 0 the cursor is already where it should
                                // be (user might have moved it with an arrow).
                            }
                        }
                    }
                    egui::ImeEvent::Commit(commit) => {
                        state.last_interaction_time = now;
                        state.ime_enabled = false;
                        // Remove in-progress preedit.
                        let old_len = state.ime_preedit.chars().count();
                        if old_len > 0 {
                            let preedit_end = state.ime_start + old_len;
                            delete_range(text, state.ime_start, preedit_end);
                        }
                        state.ime_preedit = String::new();
                        // Insert the committed text (filter newlines for singleline).
                        if !commit.is_empty() && commit != "\n" && commit != "\r" {
                            let filtered: String = commit
                                .chars()
                                .filter(|c| *c != '\n' && *c != '\r')
                                .collect();
                            if !filtered.is_empty() {
                                insert_at(text, state.ime_start, &filtered);
                                state.set_cursor(state.ime_start + filtered.chars().count());
                            }
                        } else {
                            state.set_cursor(state.ime_start);
                        }
                        changed = true;
                    }
                    egui::ImeEvent::Disabled => {
                        state.ime_enabled = false;
                        // IME disabled without commit — discard preedit.
                        let old_len = state.ime_preedit.chars().count();
                        if old_len > 0 {
                            let preedit_end = state.ime_start + old_len;
                            delete_range(text, state.ime_start, preedit_end);
                            state.set_cursor(state.ime_start);
                            changed = true;
                        }
                        state.ime_preedit = String::new();
                    }
                }
            }

            // ── Focus management ──────────────────────────────────
            egui::Event::Key {
                key: egui::Key::Escape | egui::Key::Enter,
                pressed: true,
                ..
            } => {
                // Clear any active IME preedit first.
                if state.ime_active() {
                    let old_len = state.ime_preedit.chars().count();
                    let preedit_end = state.ime_start + old_len;
                    delete_range(text, state.ime_start, preedit_end);
                    state.set_cursor(state.ime_start);
                    state.ime_preedit = String::new();
                    changed = true;
                }
                ui.memory_mut(|m| m.surrender_focus(id));
            }

            _ => {}
        }
    }

    // clamp after all mutations
    state.clamp(text.chars().count());

    // Feed final state to the undoer.
    state.undoer.feed_state(
        now,
        &UndoEntry {
            text: text.clone(),
            cursor: state.cursor,
            anchor: state.anchor,
        },
    );

    changed
}

// ────────────────────────────────────────────────────────────────────────────────
// Rendering
// ────────────────────────────────────────────────────────────────────────────────

pub fn render(ui: &mut Ui, text: &mut String, style: Style) -> egui::Response {
    let font_family = FONT_MANAGER.default_egui_font_family();
    let font_id = egui::FontId::new(style.text_size, font_family);

    // 总宽度：指定宽度或占满可用宽度
    let total_width = style.width.unwrap_or_else(|| ui.available_width());
    let size = egui::vec2(total_width, style.height);

    // click_and_drag for drag-to-select support
    let (rect, mut response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());

    let id = response.id;
    let was_focused = ui.memory(|m| m.has_focus(id));

    let pointer_pressed = response.hovered() && ui.input(|i| i.pointer.any_pressed());

    // Click / drag-start / pointer down → request focus
    if response.clicked() || response.drag_started() || pointer_pressed {
        ui.memory_mut(|m| m.request_focus(id));
    }

    let is_focused = ui.memory(|m| m.has_focus(id));

    // ── Load persisted cursor state ──
    let mut cstate: CursorState = ui.data(|d| d.get_temp(id).unwrap_or_default());
    cstate.clamp(text.chars().count());

    // If focus was just gained, always reset interaction time so cursor is visible
    if is_focused && !was_focused {
        cstate.last_interaction_time = ui.input(|i| i.time);
        // Place cursor at end for non-pointer focus (e.g. Tab)
        if !response.clicked() && !response.drag_started() && !pointer_pressed {
            cstate.set_cursor(text.chars().count());
        }
    }

    // ── 动画（hover / focus 渐变 0.15s）──
    let hover_anim = ui
        .ctx()
        .animate_bool_with_time(id.with("hover"), response.hovered(), 0.15);
    let focus_anim = ui
        .ctx()
        .animate_bool_with_time(id.with("focus"), is_focused, 0.15);

    // ── 背景色（hover 混合）──
    let bg_color = {
        if focus_anim > 0.01 {
            let base_rgba = egui::Rgba::from(style.palette.normal_bg);
            let focus_rgba = egui::Rgba::from(style.palette.focus_bg);
            Color32::from(base_rgba * (1.0 - focus_anim) + focus_rgba * focus_anim)
        } else {
            let base_rgba = egui::Rgba::from(style.palette.normal_bg);
            let hover_rgba = egui::Rgba::from(style.palette.hover_bg);
            Color32::from(base_rgba * (1.0 - hover_anim) + hover_rgba * hover_anim)
        }
    };

    // ── 边框（focus 时切换 stroke）──
    let stroke = {
        let normal = style.palette.border_stroke;
        let focus = style.palette.focus_stroke;
        if focus_anim > 0.01 {
            let w = normal.width * (1.0 - focus_anim) + focus.width * focus_anim;
            let c_normal = egui::Rgba::from(normal.color);
            let c_focus = egui::Rgba::from(focus.color);
            let c = Color32::from(c_normal * (1.0 - focus_anim) + c_focus * focus_anim);
            Stroke::new(w, c)
        } else {
            normal
        }
    };

    // ── 绘制背景 ──
    let is_flushed = style.variant == TextEditVariant::Flushed;
    if is_flushed {
        let bottom_left = egui::pos2(rect.left(), rect.bottom());
        let bottom_right = egui::pos2(rect.right(), rect.bottom());
        ui.painter()
            .line_segment([bottom_left, bottom_right], stroke);
    } else {
        ui.painter().rect(
            rect,
            style.corner_radius,
            bg_color,
            stroke,
            StrokeKind::Inside,
        );
    }

    // ── 文字区域 ──
    let mut text_rect = rect.shrink2(egui::vec2(style.padding_x, 0.0));
    let mut icon_rect = egui::Rect::NOTHING;

    if style.input_type == InputType::Password {
        let icon_width = style.icon_button_style.size;
        icon_rect = egui::Rect::from_min_max(
            egui::pos2(text_rect.right() - icon_width, rect.top()),
            egui::pos2(text_rect.right(), rect.bottom()),
        );
        text_rect.set_right(text_rect.right() - icon_width - 4.0);
    }

    let available_width = text_rect.width();
    let clip_rect = text_rect.intersect(ui.clip_rect());

    // ── Display text: password mode masks each char with ● ──
    let display_text = match style.input_type {
        InputType::Text => text.clone(),
        InputType::Password => {
            if cstate.password_visible {
                text.clone()
            } else {
                "●".repeat(text.chars().count())
            }
        }
    };

    // ── Build galley for text measurement ──
    let galley = ui.fonts_mut(|f| {
        f.layout_job(egui::text::LayoutJob::simple_singleline(
            display_text.clone(),
            font_id.clone(),
            style.palette.text_color,
        ))
    });
    let text_height = galley.size().y;

    // ── Pointer interaction: click-to-position / drag-to-select / double-click ──
    if is_focused {
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            if style.input_type != InputType::Password || !icon_rect.contains(pointer_pos) {
                let now = ui.input(|i| i.time);
                // Pointer pos → galley-relative x
                let galley_x = pointer_pos.x - text_rect.left() + cstate.scroll_offset;
                let pos_in_galley = egui::vec2(galley_x, text_height / 2.0);
                let hit = galley.cursor_from_pos(pos_in_galley);
                let char_idx = hit.index;

                if response.double_clicked() {
                    // Double-click → select word
                    if !text.is_empty() {
                        let (ws, we) = word_range_at(text, char_idx);
                        cstate.anchor = ws;
                        cstate.cursor = we;
                    }
                    cstate.last_interaction_time = now;
                } else if pointer_pressed {
                    // First click → place cursor
                    if ui.input(|i| i.modifiers.shift) {
                        cstate.cursor = char_idx;
                    } else {
                        cstate.set_cursor(char_idx);
                    }
                    cstate.last_interaction_time = now;
                } else if response.dragged() {
                    // Dragging → extend selection
                    cstate.cursor = char_idx;
                    cstate.last_interaction_time = now;
                }
            }
        }
    }

    // ── Focus lock: prevent arrow keys from moving focus away ──
    let event_filter = egui::EventFilter {
        horizontal_arrows: true,
        vertical_arrows: true,
        tab: false,
        ..Default::default()
    };
    if is_focused {
        ui.memory_mut(|m| m.set_focus_lock_filter(id, event_filter));
    }

    // ── Keyboard event handling ──
    let text_changed = if is_focused {
        let c = handle_events(ui, id, text, &mut cstate, &event_filter);
        ui.ctx().request_repaint(); // cursor blink
        c
    } else {
        false
    };

    // Rebuild galley if text was mutated this frame
    let galley = if text_changed {
        let display_text = match style.input_type {
            InputType::Text => text.clone(),
            InputType::Password => {
                if cstate.password_visible {
                    text.clone()
                } else {
                    "●".repeat(text.chars().count())
                }
            }
        };
        ui.fonts_mut(|f| {
            f.layout_job(egui::text::LayoutJob::simple_singleline(
                display_text,
                font_id.clone(),
                style.palette.text_color,
            ))
        })
    } else {
        galley
    };

    let text_width = galley.size().x;
    let text_height = galley.size().y;

    // ── Scroll offset: keep cursor visible ──
    // During IME composition, keep the END of the preedit visible instead of
    // `cstate.cursor`, which may temporarily lag behind when the OS delivers
    // multiple preedit updates in a single frame.
    if is_focused && !text.is_empty() {
        let track_index = if cstate.ime_active() && !cstate.ime_preedit.is_empty() {
            cstate.ime_start + cstate.ime_preedit.chars().count()
        } else {
            cstate.cursor
        };
        let ccursor = egui::epaint::text::cursor::CCursor {
            index: track_index,
            prefer_next_row: false,
        };
        let cursor_x = galley.pos_from_cursor(ccursor).min.x;

        if cursor_x < cstate.scroll_offset {
            cstate.scroll_offset = cursor_x;
        } else if cursor_x > cstate.scroll_offset + available_width {
            cstate.scroll_offset = cursor_x - available_width;
        }
    }
    cstate.scroll_offset = cstate
        .scroll_offset
        .clamp(0.0, (text_width - available_width).max(0.0));

    // ── Rendering ──
    let painter = ui.painter_at(clip_rect);

    if text.is_empty() && !is_focused {
        // Placeholder only (no cursor)
        let pg = painter.layout_no_wrap(
            style.placeholder.clone(),
            font_id.clone(),
            style.palette.placeholder_color,
        );
        let text_pos = egui::pos2(text_rect.left(), text_rect.center().y - pg.size().y / 2.0);
        painter.galley(text_pos, pg, style.palette.placeholder_color);
    } else {
        let text_x = text_rect.left() - cstate.scroll_offset;
        let text_y = text_rect.center().y - text_height / 2.0;

        // ── Selection highlight ──
        if is_focused && cstate.has_selection() {
            let (sel_start, sel_end) = cstate.ordered_range();
            let start_ccursor = egui::epaint::text::cursor::CCursor {
                index: sel_start,
                prefer_next_row: false,
            };
            let end_ccursor = egui::epaint::text::cursor::CCursor {
                index: sel_end,
                prefer_next_row: false,
            };
            let sx = galley.pos_from_cursor(start_ccursor).min.x;
            let ex = galley.pos_from_cursor(end_ccursor).min.x;

            let sel_rect = egui::Rect::from_min_max(
                egui::pos2(text_x + sx, text_y),
                egui::pos2(text_x + ex, text_y + text_height),
            );
            painter.rect_filled(sel_rect, 0.0, ui.visuals().selection.bg_fill);
        }

        // ── Draw text galley ──
        let text_pos = egui::pos2(text_x, text_y);
        painter.galley(text_pos, galley.clone(), style.palette.text_color);

        // Show placeholder as dim hint when focused but empty
        if text.is_empty() && is_focused && !style.placeholder.is_empty() {
            let pg = painter.layout_no_wrap(
                style.placeholder.clone(),
                font_id.clone(),
                style.palette.placeholder_color,
            );
            let ph_pos = egui::pos2(text_rect.left(), text_rect.center().y - pg.size().y / 2.0);
            painter.galley(ph_pos, pg, style.palette.placeholder_color);
        }

        // ── IME preedit underline ──
        if is_focused && cstate.ime_active() {
            let preedit_len = cstate.ime_preedit.chars().count();
            let start_ccursor = egui::epaint::text::cursor::CCursor {
                index: cstate.ime_start,
                prefer_next_row: false,
            };
            let end_ccursor = egui::epaint::text::cursor::CCursor {
                index: cstate.ime_start + preedit_len,
                prefer_next_row: false,
            };
            let px = galley.pos_from_cursor(start_ccursor).min.x;
            let ex = galley.pos_from_cursor(end_ccursor).min.x;
            let underline_y = text_y + text_height + 1.0;
            painter.line_segment(
                [
                    egui::pos2(text_x + px, underline_y),
                    egui::pos2(text_x + ex, underline_y),
                ],
                Stroke::new(1.5, style.palette.text_color),
            );
        }

        // ── Cursor ──
        // During IME composition the cursor must always be visible at the end of
        // the preedit text — egui's own TextEdit has a bug where the cursor can
        // appear stale during IME, so we force a solid (non-blinking) cursor here
        // and re-read the cursor position straight from `cstate` after events ran.
        if is_focused {
            let now = ui.input(|i| i.time);
            let ime_active = cstate.ime_active();
            let show = ime_active || should_show_cursor(now, cstate.last_interaction_time);
            if show {
                // When IME is active, the cursor must sit at the end of the preedit
                // regardless of anything else. This guards against any stale cursor
                // position that survived the event loop.
                let cursor_index = if ime_active && !cstate.ime_preedit.is_empty() {
                    cstate.ime_start + cstate.ime_preedit.chars().count()
                } else {
                    cstate.cursor
                };
                let ccursor = egui::epaint::text::cursor::CCursor {
                    index: cursor_index,
                    prefer_next_row: false,
                };
                let cx = galley.pos_from_cursor(ccursor).min.x;
                let screen_cx = text_x + cx;

                painter.line_segment(
                    [
                        egui::pos2(screen_cx, text_y),
                        egui::pos2(screen_cx, text_y + text_height),
                    ],
                    Stroke::new(1.5, style.palette.text_color),
                );
            }
        }
    }

    // ── 渲染密码可见性切换按钮 ──
    if style.input_type == InputType::Password {
        let icon = if cstate.password_visible {
            crate::icon::ICONS.eye.clone()
        } else {
            crate::icon::ICONS.eye_slash.clone()
        };
        let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(icon_rect).layout(
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
        ));
        let btn_res = icon_button::render(&mut child_ui, icon, style.icon_button_style.clone());
        if btn_res.clicked() {
            cstate.password_visible = !cstate.password_visible;
            ui.ctx().request_repaint();
        }
        btn_res.on_hover_cursor(egui::CursorIcon::PointingHand);
    }

    // ── IME platform output: tell the OS where the cursor is ──
    if is_focused {
        // During IME, report the preedit-end position to keep the candidate
        // window aligned with what the user is composing.
        let track_index = if cstate.ime_active() && !cstate.ime_preedit.is_empty() {
            cstate.ime_start + cstate.ime_preedit.chars().count()
        } else {
            cstate.cursor
        };
        let cursor_screen_x = {
            let ccursor = egui::epaint::text::cursor::CCursor {
                index: track_index,
                prefer_next_row: false,
            };
            let galley_x = galley.pos_from_cursor(ccursor).min.x;
            let text_x = text_rect.left() - cstate.scroll_offset;
            (text_x + galley_x).clamp(text_rect.left(), text_rect.right())
        };
        let cursor_rect = egui::Rect::from_min_max(
            egui::pos2(cursor_screen_x, rect.top()),
            egui::pos2(cursor_screen_x + 1.0, rect.bottom()),
        );
        ui.output_mut(|o| {
            o.ime = Some(egui::output::IMEOutput { rect, cursor_rect });
            o.mutable_text_under_cursor = true;
        });
    }

    // ── Persist cursor state ──
    ui.data_mut(|d| d.insert_temp(id, cstate));

    // 鼠标指针
    response = response.on_hover_cursor(CursorIcon::Text);

    if text_changed {
        response.mark_changed();
    }

    response
}

pub fn match_text_edit_palette(
    theme: &Theme,
    color: Option<Color>,
    variant: &TextEditVariant,
) -> TextEditPalette {
    // focus 颜色：如果指定了 color 用 color.normal()，否则用 primary
    let focus_color = match color {
        Some(c) => c.normal(),
        None => theme.colors.primary.normal(),
    };

    match variant {
        TextEditVariant::Subtle => {
            // 有底色填充，无边框，focus 时出现 focus ring
            TextEditPalette {
                normal_bg: theme.colors.fg.base.gamma_multiply(0.06),
                hover_bg: theme.colors.fg.base.gamma_multiply(0.1),
                focus_bg: theme.colors.fg.base.gamma_multiply(0.06),
                text_color: theme.colors.fg.base,
                placeholder_color: theme.colors.fg.base.gamma_multiply(0.35),
                border_stroke: Stroke::new(1.0, Color32::TRANSPARENT),
                focus_stroke: Stroke::new(1.5, focus_color),
            }
        }
        TextEditVariant::Outline => {
            // 透明背景，有边框，focus 时边框变为 focus color
            TextEditPalette {
                normal_bg: Color32::TRANSPARENT,
                hover_bg: Color32::TRANSPARENT,
                focus_bg: Color32::TRANSPARENT,
                text_color: theme.colors.fg.base,
                placeholder_color: theme.colors.fg.base.gamma_multiply(0.35),
                border_stroke: Stroke::new(1.0, theme.colors.fg.base.gamma_multiply(0.25)),
                focus_stroke: Stroke::new(1.5, focus_color),
            }
        }
        TextEditVariant::Flushed => {
            // 透明背景，只有底部边线，focus 时底线变为 focus color
            TextEditPalette {
                normal_bg: Color32::TRANSPARENT,
                hover_bg: Color32::TRANSPARENT,
                focus_bg: Color32::TRANSPARENT,
                text_color: theme.colors.fg.base,
                placeholder_color: theme.colors.fg.base.gamma_multiply(0.35),
                border_stroke: Stroke::new(1.0, theme.colors.fg.base.gamma_multiply(0.25)),
                focus_stroke: Stroke::new(2.0, focus_color),
            }
        }
    }
}
