//! A controlled editor for ordered typed rows.
//!
//! The caller supplies stable IDs, row creation and field rendering through
//! [`RowEditorConfig`](crate::row_editor::RowEditorConfig). Requests emit one complete proposed collection through
//! [`RowEditorEvent`](crate::row_editor::RowEditorEvent); the accepted collection changes only with
//! [`RowEditorState::set_rows`](crate::row_editor::RowEditorState::set_rows). The caller owns validation and persistence.
//!
//! ```
//! use gpui_form_component::row_editor::{
//!     RowEditorConfig, RowEditorRowContext,
//! };
//! use gpui_kit::{AnyElement, App, IntoElement, ParentElement, SharedString, Window, div};
//!
//! #[derive(Clone)]
//! struct Row { id: u64, title: String }
//!
//! #[derive(Clone, Default)]
//! struct Fields { next_id: u64 }
//!
//! impl Fields {
//!     fn fresh(&mut self, title: String) -> Row {
//!         self.next_id += 1;
//!         Row { id: self.next_id, title }
//!     }
//! }
//! impl RowEditorConfig<Row> for Fields {
//!     fn row_id(&self, row: &Row) -> SharedString { row.id.to_string().into() }
//!     fn create_row(&mut self) -> Option<Row> { Some(self.fresh(String::new())) }
//!     fn duplicate_row(&mut self, row: &Row) -> Option<Row> {
//!         Some(self.fresh(row.title.clone()))
//!     }
//!     fn render_row(&self, row: &Row, _: RowEditorRowContext<Row, Self>,
//!         _: &mut Window, _: &mut App) -> AnyElement {
//!         div().child(row.title.clone()).into_any_element()
//!     }
//! }
//! ```

use std::collections::{HashMap, HashSet};
use std::fmt;

use gpui_es_fluent::localize_message;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Selectable as _, Sizable as _, ThemeStyled as _,
    button::{Button, ButtonVariants as _},
    menu::DropdownMenu as _,
};
use gpui_kit::{
    AnyElement, App, Context, ElementId, Entity, EventEmitter, FocusHandle, Focusable, Global,
    InteractiveElement as _, IntoElement, KeyBinding, ParentElement as _, Render, RenderOnce, Role,
    SharedString, StatefulInteractiveElement as _, Styled as _, Subscription, Window, actions, div,
    prelude::FluentBuilder as _,
};

use crate::i18n::RowEditorText;

/// Keyboard commands shared by the toolbar and row header region.
pub mod action {
    use super::actions;
    actions!(
        gpui_form_row_editor,
        [
            #[derive(Eq)]
            InsertRow,
            #[derive(Eq)]
            RemoveRow,
            #[derive(Eq)]
            DuplicateRow,
            #[derive(Eq)]
            MoveRowUp,
            #[derive(Eq)]
            MoveRowDown,
            #[derive(Eq)]
            PreviousRow,
            #[derive(Eq)]
            NextRow
        ]
    );
}

struct Keybindings;
impl Global for Keybindings {}

fn init(cx: &mut App) {
    if cx.try_global::<Keybindings>().is_some() {
        return;
    }
    cx.set_global(Keybindings);
    let context = Some("RowEditor && !RowEditorField");
    cx.bind_keys([
        KeyBinding::new("insert", action::InsertRow, context),
        KeyBinding::new("delete", action::RemoveRow, context),
        KeyBinding::new("secondary-d", action::DuplicateRow, context),
        KeyBinding::new("alt-up", action::MoveRowUp, context),
        KeyBinding::new("alt-down", action::MoveRowDown, context),
        KeyBinding::new("up", action::PreviousRow, context),
        KeyBinding::new("down", action::NextRow, context),
    ]);
}

/// Caller-owned row construction, stable identity and field presentation.
///
/// IDs must be nonempty and unique within the collection. Creation and
/// duplication must return fresh IDs; returning `None` declines the command.
/// Keep field entities keyed by that ID. Rendered fields receive interaction
/// flags and an editor handle through [`RowEditorRowContext`].
pub trait RowEditorConfig<Row: Clone + 'static>: Default + Sized + 'static {
    /// Returns the caller's stable ID, independent of row position or values.
    fn row_id(&self, row: &Row) -> SharedString;
    /// Creates a row for insertion, or declines the request.
    fn create_row(&mut self) -> Option<Row>;
    /// Checks insertion availability without creating a row or allocating an ID.
    /// The editor applies the same check to its toolbar and requested changes.
    fn can_insert(&self, _rows: &[Row]) -> bool {
        true
    }
    /// Copies a row with a fresh stable ID, or declines the request.
    fn duplicate_row(&mut self, row: &Row) -> Option<Row>;
    /// Checks duplication availability without invoking the row factory.
    fn can_duplicate(&self, _row: &Row, _rows: &[Row]) -> bool {
        true
    }
    /// Returns an accessible row header label. The default is the stable ID.
    fn row_label(&self, row: &Row) -> SharedString {
        self.row_id(row)
    }
    /// Renders fields; apply the supplied disabled and readonly flags.
    fn render_row(
        &self,
        row: &Row,
        context: RowEditorRowContext<Row, Self>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement;
}

/// The editor and interaction flags supplied to caller-rendered row fields.
pub struct RowEditorRowContext<Row: Clone + 'static, Config: RowEditorConfig<Row>> {
    editor: Entity<RowEditorState<Row, Config>>,
    id: SharedString,
    disabled: bool,
    readonly: bool,
}

impl<Row: Clone + 'static, Config: RowEditorConfig<Row>> RowEditorRowContext<Row, Config> {
    /// Returns the editor handle for subscribing or requesting a field update.
    pub fn editor(&self) -> &Entity<RowEditorState<Row, Config>> {
        &self.editor
    }
    /// Returns this row's stable ID.
    pub fn id(&self) -> &SharedString {
        &self.id
    }
    /// Returns whether fields must reject all interaction.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
    /// Returns whether fields may navigate but must reject editing.
    pub fn is_readonly(&self) -> bool {
        self.readonly
    }
}

/// The semantic operation that produced a requested collection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RowEditorChange {
    /// A newly created row was inserted at this position.
    Insert { id: SharedString, ix: usize },
    /// A row was removed.
    Remove { id: SharedString, ix: usize },
    /// A row was duplicated with a fresh ID directly after its source.
    Duplicate {
        source: SharedString,
        id: SharedString,
        ix: usize,
    },
    /// An existing row moved to another position.
    Move {
        id: SharedString,
        from: usize,
        to: usize,
    },
    /// Caller-rendered fields requested a replacement of one row's values.
    Replace { id: SharedString },
}

/// One coherent requested change; accepting it is the caller's decision.
#[derive(Clone, Debug)]
pub struct RowEditorEvent<Row> {
    rows: Vec<Row>,
    change: RowEditorChange,
}
impl<Row> RowEditorEvent<Row> {
    /// Returns the complete requested ordered collection.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }
    /// Consumes the event and returns the requested collection.
    pub fn into_rows(self) -> Vec<Row> {
        self.rows
    }
    /// Returns the requested operation.
    pub fn change(&self) -> &RowEditorChange {
        &self.change
    }
}

/// Invalid caller identity; rejected collections leave accepted rows intact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RowEditorIdentityError {
    /// A row has an empty ID.
    Empty { ix: usize },
    /// Two rows have the same ID.
    Duplicate { id: SharedString },
    /// A field replacement tried to change the row's ID.
    Changed { id: SharedString },
}
impl fmt::Display for RowEditorIdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty { ix } => write!(f, "Row {ix} has an empty ID"),
            Self::Duplicate { id } => write!(f, "Duplicate row ID: {id}"),
            Self::Changed { id } => write!(f, "Replacement changed row ID: {id}"),
        }
    }
}
impl std::error::Error for RowEditorIdentityError {}

/// Construction options for a caller-configured row editor shape.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "component-shape", derive(bon::Builder))]
pub struct RowEditorOptions<Config: Default> {
    #[cfg_attr(feature = "component-shape", builder(default))]
    config: Config,
    #[cfg_attr(feature = "component-shape", builder(default))]
    disabled: bool,
    #[cfg_attr(feature = "component-shape", builder(default))]
    readonly: bool,
}

impl<Config: Default> Default for RowEditorOptions<Config> {
    fn default() -> Self {
        Self {
            config: Config::default(),
            disabled: false,
            readonly: false,
        }
    }
}

enum FocusRequest {
    Row(SharedString),
    AfterRemoval {
        id: SharedString,
        next: Option<SharedString>,
    },
}

/// Retained selection and focus around the caller's accepted row collection.
///
/// Requests never modify `rows()`. Subscribe to [`RowEditorEvent`], validate
/// its rows, then call `set_rows` to accept them. Programmatic updates emit no
/// change event. Reordering preserves keyed focus and values; deleting the
/// focused row moves focus to the following row, preceding row, or editor.
pub struct RowEditorState<Row: Clone + 'static, Config: RowEditorConfig<Row>> {
    config: Config,
    rows: Vec<Row>,
    focus: FocusHandle,
    row_focus: HashMap<SharedString, FocusHandle>,
    focus_subscriptions: HashMap<SharedString, Subscription>,
    selected: Option<SharedString>,
    pending_focus: Option<FocusRequest>,
    disabled: bool,
    readonly: bool,
    identity_error: Option<RowEditorIdentityError>,
}

impl<Row: Clone + 'static, Config: RowEditorConfig<Row>> EventEmitter<RowEditorEvent<Row>>
    for RowEditorState<Row, Config>
{
}
impl<Row: Clone + 'static, Config: RowEditorConfig<Row>> Focusable for RowEditorState<Row, Config> {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl<Row: Clone + 'static, Config: RowEditorConfig<Row>> RowEditorState<Row, Config> {
    /// Creates an empty editor with the default caller configuration.
    pub fn new(_: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::with_config(Config::default(), cx)
    }
    /// Creates an empty editor with caller configuration.
    pub fn with_config(config: Config, cx: &mut Context<Self>) -> Self {
        init(cx);
        Self {
            config,
            rows: Vec::new(),
            focus: cx.focus_handle(),
            row_focus: HashMap::new(),
            focus_subscriptions: HashMap::new(),
            selected: None,
            pending_focus: None,
            disabled: false,
            readonly: false,
            identity_error: None,
        }
    }
    /// Creates an editor with caller configuration and interaction options.
    pub fn with_options(options: RowEditorOptions<Config>, cx: &mut Context<Self>) -> Self {
        let mut state = Self::with_config(options.config, cx);
        state.disabled = options.disabled;
        state.readonly = options.readonly;
        state
    }
    /// Returns accepted rows in their current order.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }
    /// Returns the selected stable ID.
    pub fn selected_id(&self) -> Option<&SharedString> {
        self.selected.as_ref()
    }
    /// Returns the most recent identity error; a valid update clears it.
    pub fn identity_error(&self) -> Option<&RowEditorIdentityError> {
        self.identity_error.as_ref()
    }
    /// Returns whether interaction is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
    /// Returns whether navigation is allowed while editing is blocked.
    pub fn is_readonly(&self) -> bool {
        self.readonly
    }
    /// Sets disabled state for commands and caller-rendered fields.
    ///
    /// Availability also observes the caller's insertion and duplication checks.
    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.disabled = disabled;
        self.pending_focus = None;
        cx.notify();
    }
    /// Whether insertion is available under the current caller and interaction policy.
    pub fn can_insert(&self) -> bool {
        !self.disabled && !self.readonly && self.config.can_insert(&self.rows)
    }
    /// Whether duplication of this stable ID is currently available.
    pub fn can_duplicate(&self, id: &str) -> bool {
        !self.disabled
            && !self.readonly
            && self
                .position(id)
                .is_some_and(|ix| self.config.can_duplicate(&self.rows[ix], &self.rows))
    }
    /// Sets readonly state for commands and caller-rendered fields.
    pub fn set_readonly(&mut self, readonly: bool, cx: &mut Context<Self>) {
        self.readonly = readonly;
        self.pending_focus = None;
        cx.notify();
    }
    fn position(&self, id: &str) -> Option<usize> {
        self.rows
            .iter()
            .position(|row| self.config.row_id(row).as_ref() == id)
    }
    fn check_ids(&self, rows: &[Row]) -> Result<HashSet<SharedString>, RowEditorIdentityError> {
        let mut ids = HashSet::with_capacity(rows.len());
        for (ix, row) in rows.iter().enumerate() {
            let id = self.config.row_id(row);
            if id.is_empty() {
                return Err(RowEditorIdentityError::Empty { ix });
            }
            if !ids.insert(id.clone()) {
                return Err(RowEditorIdentityError::Duplicate { id });
            }
        }
        Ok(ids)
    }
    fn reject(
        &mut self,
        error: RowEditorIdentityError,
        cx: &mut Context<Self>,
    ) -> RowEditorIdentityError {
        self.identity_error = Some(error.clone());
        self.pending_focus = None;
        cx.notify();
        error
    }
    /// Replaces accepted rows without emitting a requested change.
    ///
    /// IDs retain focus across reordering and updates, including replacement
    /// collections with the same length. Invalid identities reject the update.
    pub fn set_rows(
        &mut self,
        rows: Vec<Row>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), RowEditorIdentityError> {
        let ids = self
            .check_ids(&rows)
            .map_err(|error| self.reject(error, cx))?;
        let focused_id = self
            .row_focus
            .iter()
            .find(|(_, focus)| focus.contains_focused(window, cx))
            .map(|(id, _)| id.clone());
        let had_focus = self.focus.contains_focused(window, cx);
        let old_ix = self
            .selected
            .as_ref()
            .and_then(|id| self.position(id))
            .unwrap_or(0);
        let preferred = match self.pending_focus.take() {
            Some(FocusRequest::Row(id)) => Some(id).filter(|id| ids.contains(id)),
            Some(FocusRequest::AfterRemoval { id, next })
                if !ids.contains(&id)
                    && (focused_id.as_ref() == Some(&id)
                        || (focused_id.is_none() && self.selected.as_ref() == Some(&id))) =>
            {
                next.filter(|id| ids.contains(id))
            },
            _ => None,
        };
        let selected = preferred
            .clone()
            .or_else(|| self.selected.clone().filter(|id| ids.contains(id)))
            .or_else(|| {
                rows.get(old_ix.min(rows.len().saturating_sub(1)))
                    .map(|row| self.config.row_id(row))
            });
        self.row_focus.retain(|id, _| ids.contains(id));
        self.focus_subscriptions.retain(|id, _| ids.contains(id));
        for id in ids {
            if !self.row_focus.contains_key(&id) {
                let focus = cx.focus_handle();
                let row_id = id.clone();
                let subscription = cx.on_focus_in(&focus, window, move |state, _, cx| {
                    if !state.disabled && state.selected.as_ref() != Some(&row_id) {
                        state.selected = Some(row_id.clone());
                        cx.notify();
                    }
                });
                self.row_focus.insert(id.clone(), focus);
                self.focus_subscriptions.insert(id, subscription);
            }
        }
        self.rows = rows;
        self.selected = selected;
        self.identity_error = None;
        if had_focus
            && !self.disabled
            && (preferred.is_some()
                || focused_id
                    .as_ref()
                    .is_some_and(|id| !self.row_focus.contains_key(id)))
        {
            self.focus_selected(window, cx);
        }
        cx.notify();
        Ok(())
    }
    /// Selects and focuses a stable row without requesting a value change.
    pub fn select_row(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.position(id).is_none() {
            return;
        }
        self.selected = Some(SharedString::from(id.to_owned()));
        self.focus_selected(window, cx);
        cx.notify();
    }
    fn focus_selected(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.selected
            .as_ref()
            .and_then(|id| self.row_focus.get(id))
            .unwrap_or(&self.focus)
            .focus(window, cx);
    }
    fn navigate(&mut self, forward: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.rows.is_empty() {
            return;
        }
        let ix = self
            .selected
            .as_ref()
            .and_then(|id| self.position(id))
            .unwrap_or(0);
        let ix = if forward {
            (ix + 1).min(self.rows.len() - 1)
        } else {
            ix.saturating_sub(1)
        };
        let id = self.config.row_id(&self.rows[ix]);
        self.select_row(&id, window, cx);
    }
    fn request(
        &mut self,
        rows: Vec<Row>,
        change: RowEditorChange,
        focus: Option<FocusRequest>,
        cx: &mut Context<Self>,
    ) -> Result<(), RowEditorIdentityError> {
        self.check_ids(&rows)
            .map_err(|error| self.reject(error, cx))?;
        self.identity_error = None;
        self.pending_focus = focus;
        cx.emit(RowEditorEvent { rows, change });
        cx.notify();
        Ok(())
    }
    /// Requests insertion after a stable ID, or appends when `after` is absent.
    pub fn request_insert(
        &mut self,
        after: Option<&str>,
        cx: &mut Context<Self>,
    ) -> Result<(), RowEditorIdentityError> {
        if !self.can_insert() {
            return Ok(());
        }
        let ix = match after {
            Some(id) => match self.position(id) {
                Some(ix) => ix + 1,
                None => return Ok(()),
            },
            None => self.rows.len(),
        };
        let Some(row) = self.config.create_row() else {
            return Ok(());
        };
        let id = self.config.row_id(&row);
        let mut rows = self.rows.clone();
        rows.insert(ix, row);
        self.request(
            rows,
            RowEditorChange::Insert { id: id.clone(), ix },
            Some(FocusRequest::Row(id)),
            cx,
        )
    }
    /// Requests removal of one stable row.
    pub fn request_remove(
        &mut self,
        id: &str,
        cx: &mut Context<Self>,
    ) -> Result<(), RowEditorIdentityError> {
        if self.disabled || self.readonly {
            return Ok(());
        }
        let Some(ix) = self.position(id) else {
            return Ok(());
        };
        let mut rows = self.rows.clone();
        rows.remove(ix);
        let focus = rows
            .get(ix.min(rows.len().saturating_sub(1)))
            .map(|row| self.config.row_id(row));
        self.request(
            rows,
            RowEditorChange::Remove {
                id: id.to_owned().into(),
                ix,
            },
            Some(FocusRequest::AfterRemoval {
                id: id.to_owned().into(),
                next: focus,
            }),
            cx,
        )
    }
    /// Requests duplication after its source with a caller-created fresh ID.
    pub fn request_duplicate(
        &mut self,
        id: &str,
        cx: &mut Context<Self>,
    ) -> Result<(), RowEditorIdentityError> {
        if !self.can_duplicate(id) {
            return Ok(());
        }
        let Some(ix) = self.position(id) else {
            return Ok(());
        };
        let Some(row) = self.config.duplicate_row(&self.rows[ix]) else {
            return Ok(());
        };
        let new_id = self.config.row_id(&row);
        let mut rows = self.rows.clone();
        rows.insert(ix + 1, row);
        self.request(
            rows,
            RowEditorChange::Duplicate {
                source: id.to_owned().into(),
                id: new_id.clone(),
                ix: ix + 1,
            },
            Some(FocusRequest::Row(new_id)),
            cx,
        )
    }
    /// Requests moving an existing row to a final zero-based position.
    pub fn request_move(
        &mut self,
        id: &str,
        to: usize,
        cx: &mut Context<Self>,
    ) -> Result<(), RowEditorIdentityError> {
        if self.disabled || self.readonly {
            return Ok(());
        }
        let Some(from) = self.position(id) else {
            return Ok(());
        };
        if from == to || to >= self.rows.len() {
            return Ok(());
        }
        let mut rows = self.rows.clone();
        let row = rows.remove(from);
        rows.insert(to, row);
        self.request(
            rows,
            RowEditorChange::Move {
                id: id.to_owned().into(),
                from,
                to,
            },
            None,
            cx,
        )
    }
    /// Requests changed field values while preserving the row ID.
    pub fn request_replace(
        &mut self,
        id: &str,
        row: Row,
        cx: &mut Context<Self>,
    ) -> Result<(), RowEditorIdentityError> {
        if self.disabled || self.readonly {
            return Ok(());
        }
        let Some(ix) = self.position(id) else {
            return Ok(());
        };
        if self.config.row_id(&row).as_ref() != id {
            return Err(self.reject(
                RowEditorIdentityError::Changed {
                    id: id.to_owned().into(),
                },
                cx,
            ));
        }
        let mut rows = self.rows.clone();
        rows[ix] = row;
        self.request(
            rows,
            RowEditorChange::Replace {
                id: id.to_owned().into(),
            },
            None,
            cx,
        )
    }
    fn insert_selected(&mut self, cx: &mut Context<Self>) {
        let id = self.selected.clone();
        let _ = self.request_insert(id.as_deref(), cx);
    }
    fn remove_selected(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected.clone() {
            let _ = self.request_remove(&id, cx);
        }
    }
    fn duplicate_selected(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected.clone() {
            let _ = self.request_duplicate(&id, cx);
        }
    }
    fn move_selected(&mut self, forward: bool, cx: &mut Context<Self>) {
        if let Some(id) = self.selected.clone()
            && let Some(ix) = self.position(&id)
        {
            let to = if forward {
                ix + 1
            } else {
                ix.saturating_sub(1)
            };
            let _ = self.request_move(&id, to, cx);
        }
    }
}

/// GPUI Kit presentation of a controlled ordered row collection.
///
/// Insert/Delete, secondary-D and Alt-Up/Down operate in the row header and
/// toolbar region. Up/Down navigate rows, including in readonly mode. These
/// shortcuts are excluded from caller-rendered fields, which keep their normal
/// text editing behavior. The caller owns scrolling around the editor.
#[cfg_attr(
    feature = "component-shape",
    derive(component_shape_gpui::GpuiComponentShape)
)]
#[cfg_attr(feature = "component-shape", gpui_component_shape(state = RowEditorState<Row, Config>, value = Vec<Row>, field_suffix = "row_editor", value_binding))]
#[derive(IntoElement)]
pub struct RowEditor<Row: Clone + 'static, Config: RowEditorConfig<Row>> {
    state: Entity<RowEditorState<Row, Config>>,
    id: ElementId,
}
impl<Row: Clone + 'static, Config: RowEditorConfig<Row>> RowEditor<Row, Config> {
    /// Renders the state with an identity derived from its retained entity.
    pub fn new(state: &Entity<RowEditorState<Row, Config>>) -> Self {
        Self {
            state: state.clone(),
            id: ("row-editor", state.entity_id()).into(),
        }
    }
    /// Supplies completed construction options in a form shape declaration.
    #[cfg(feature = "component-shape")]
    pub fn from(options: RowEditorOptions<Config>) -> RowEditorOptions<Config> {
        options
    }
    /// Sets a stable editor identity for application composition.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
}

impl<Row: Clone + 'static, Config: RowEditorConfig<Row>> RenderOnce for RowEditor<Row, Config> {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div().id(self.id).w_full().child(self.state)
    }
}

impl<Row: Clone + 'static, Config: RowEditorConfig<Row>> Render for RowEditorState<Row, Config> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let editor = cx.entity();
        let state = self;
        let disabled = state.disabled;
        let readonly = state.readonly;
        let locked = disabled || readonly;
        let can_insert = state.can_insert();
        let selected_ix = state.selected.as_ref().and_then(|id| state.position(id));
        let has_selection = selected_ix.is_some();
        let can_up = selected_ix.is_some_and(|ix| ix > 0);
        let can_down = selected_ix.is_some_and(|ix| ix + 1 < state.rows.len());
        let focus = state.focus.clone();
        let config = &state.config;
        let selected = &state.selected;
        let row_focus = &state.row_focus;
        let identity_error = state.identity_error.is_some();
        let accepted_rows = &state.rows;
        let rows = accepted_rows
            .iter()
            .map(|row| {
                let id = config.row_id(row);
                let row_focus = row_focus[&id].clone();
                let selected = selected.as_ref() == Some(&id);
                let row_state = editor.clone();
                let select_id = id.clone();
                let fields = config.render_row(
                    row,
                    RowEditorRowContext {
                        editor: editor.clone(),
                        id: id.clone(),
                        disabled,
                        readonly,
                    },
                    window,
                    cx,
                );
                div()
                    .id(id)
                    .role(Role::Group)
                    .aria_label(config.row_label(row))
                    .track_focus(&row_focus.clone().tab_stop(!disabled))
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_2()
                    .border_1()
                    .border_color(cx.theme().border)
                    .when(row_focus.is_focused(window), |this| {
                        this.focus_ring_style(window, cx)
                    })
                    .child(
                        Button::new("select-row")
                            .ghost()
                            .small()
                            .label(config.row_label(row))
                            .selected(selected)
                            .disabled(disabled)
                            .on_click(move |_, window, cx| {
                                row_state.update(cx, |state, cx| {
                                    state.select_row(&select_id, window, cx)
                                })
                            }),
                    )
                    .child(div().key_context("RowEditorField").child(fields))
            })
            .collect::<Vec<_>>();
        let empty = rows.is_empty();
        let menu_state = editor.clone();
        let insert_label: SharedString = localize_message(cx, &RowEditorText::InsertRow).into();
        let commands_label: SharedString = localize_message(cx, &RowEditorText::RowActions).into();
        let duplicate_label: SharedString = localize_message(cx, &RowEditorText::Duplicate).into();
        let remove_label: SharedString = localize_message(cx, &RowEditorText::Remove).into();
        let up_label: SharedString = localize_message(cx, &RowEditorText::MoveUp).into();
        let down_label: SharedString = localize_message(cx, &RowEditorText::MoveDown).into();
        div()
            .id("rows")
            .track_focus(&focus.tab_stop(!disabled))
            .key_context("RowEditor")
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .when(disabled, |this| this.opacity(0.5))
            .on_action(
                window.listener_for(&editor, |state, _: &action::InsertRow, _, cx| {
                    state.insert_selected(cx)
                }),
            )
            .on_action(
                window.listener_for(&editor, |state, _: &action::RemoveRow, _, cx| {
                    state.remove_selected(cx)
                }),
            )
            .on_action(
                window.listener_for(&editor, |state, _: &action::DuplicateRow, _, cx| {
                    state.duplicate_selected(cx)
                }),
            )
            .on_action(
                window.listener_for(&editor, |state, _: &action::MoveRowUp, _, cx| {
                    state.move_selected(false, cx)
                }),
            )
            .on_action(
                window.listener_for(&editor, |state, _: &action::MoveRowDown, _, cx| {
                    state.move_selected(true, cx)
                }),
            )
            .on_action(
                window.listener_for(&editor, |state, _: &action::PreviousRow, window, cx| {
                    state.navigate(false, window, cx)
                }),
            )
            .on_action(
                window.listener_for(&editor, |state, _: &action::NextRow, window, cx| {
                    state.navigate(true, window, cx)
                }),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("insert-row")
                            .small()
                            .outline()
                            .label(insert_label)
                            .disabled(!can_insert)
                            .on_click(window.listener_for(&editor, |state, _, _, cx| {
                                state.insert_selected(cx)
                            })),
                    )
                    .child(
                        Button::new("row-actions")
                            .small()
                            .ghost()
                            .label(commands_label)
                            .disabled(locked || !has_selection)
                            .dropdown_menu(move |menu, _, cx| {
                                // The menu dispatches the same commands as keyboard input.
                                // Each request rechecks these flags before emitting a change.
                                let state = menu_state.read(cx);
                                let locked = state.disabled || state.readonly;
                                let can_duplicate = state
                                    .selected
                                    .as_ref()
                                    .is_some_and(|id| state.can_duplicate(id));
                                menu.menu_with_disabled(
                                    duplicate_label.clone(),
                                    Box::new(action::DuplicateRow),
                                    !can_duplicate,
                                )
                                .menu_with_disabled(
                                    up_label.clone(),
                                    Box::new(action::MoveRowUp),
                                    locked || !can_up,
                                )
                                .menu_with_disabled(
                                    down_label.clone(),
                                    Box::new(action::MoveRowDown),
                                    locked || !can_down,
                                )
                                .separator()
                                .menu_with_disabled(
                                    remove_label.clone(),
                                    Box::new(action::RemoveRow),
                                    locked,
                                )
                            }),
                    ),
            )
            .when(empty, |this| {
                this.child(
                    div()
                        .text_color(cx.theme().muted_foreground)
                        .child(localize_message(cx, &RowEditorText::NoRows)),
                )
            })
            .child(
                div()
                    .id("row-list")
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(rows),
            )
            .when(readonly && !disabled, |this| {
                this.child(
                    div()
                        .text_color(cx.theme().muted_foreground)
                        .child(localize_message(cx, &RowEditorText::Readonly)),
                )
            })
            .when(identity_error, |this| {
                this.child(
                    div()
                        .text_color(cx.theme().danger)
                        .child(localize_message(cx, &RowEditorText::InvalidIdentity)),
                )
            })
    }
}

#[cfg(feature = "component-shape")]
impl<Row: Clone + 'static, Config: RowEditorConfig<Row>>
    gpui_form_runtime::shape::GpuiFormComponentShapePolicy for RowEditor<Row, Config>
{
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}
#[cfg(feature = "component-shape")]
impl<Row: Clone + 'static, Config: RowEditorConfig<Row>>
    gpui_form_runtime::shape::GpuiComponentStateValueBinding<Vec<Row>>
    for RowEditorState<Row, Config>
{
    type Event = RowEditorEvent<Row>;
    fn seed_value_binding_state(
        state: &mut Self,
        value: Option<&Vec<Row>>,
        window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        let _ = state.set_rows(value.cloned().unwrap_or_default(), window, cx);
    }
    fn value_change(
        _: &Self,
        event: &Self::Event,
    ) -> gpui_form_runtime::shape::ValueChange<Vec<Row>> {
        gpui_form_runtime::shape::ValueChange::Set(event.rows.clone())
    }
}

#[cfg(feature = "component-shape")]
impl<Row: Clone + 'static, Config: RowEditorConfig<Row>>
    component_shape_gpui::GpuiComponentShapeBuilder<RowEditor<Row, Config>>
    for RowEditorOptions<Config>
{
    fn build(
        self,
        _: &mut Window,
        cx: &mut Context<'_, RowEditorState<Row, Config>>,
    ) -> RowEditorState<Row, Config> {
        RowEditorState::with_options(self, cx)
    }
}
