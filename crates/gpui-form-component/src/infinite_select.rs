//! Infinite-select runtime support for nested enum trees.
//!
//! This module exposes both the low-level trait and path helpers used by
//! generated code and a higher-level `Select` entity that owns the
//! cascading `SelectState`s for a field.

use gpui::{
    App, AppContext as _, Context, Empty, Entity, EventEmitter, FocusHandle, Focusable,
    IntoElement, ParentElement as _, Render, RenderOnce, SharedString, Styled as _, Subscription,
    Window, div,
};
use gpui_component::{
    IndexPath,
    form::{Field, field},
    select::{Select as GpuiSelect, SelectDelegate, SelectEvent, SelectItem, SelectState},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use std::str::FromStr;
use std::{error::Error, fmt};

/// Trait for infinite-select enums that expose their nested structure.
///
/// This trait is derived using `#[derive(InfiniteSelect)]` and provides:
/// - the variant list at each level
/// - child variant names, keys, and labels for cascading selects
/// - index-based and key-based child selection to update nested values
/// - the current selection paths for a concrete value
///
/// Unlike a simple associated-type approach, this trait supports heterogeneous
/// inner types: each variant can contain a different inner type.
pub trait InfiniteSelectValue: Sized + Clone + Default + PartialEq + 'static {
    /// Returns all possible variants at this level with default inner values.
    fn variants() -> Vec<Self>;

    /// Returns the variant name/discriminant as a string.
    fn variant_name(&self) -> &'static str;

    /// Returns a stable key for this variant.
    ///
    /// The derived implementation currently mirrors `variant_name()`, which is
    /// stable across enum reordering and suitable for persisted paths.
    fn variant_key(&self) -> &'static str;

    /// Returns the localized label for this specific variant.
    fn variant_label(&self) -> SharedString;

    /// Returns true if this variant contains an inner value.
    fn has_inner(&self) -> bool;

    /// Returns the variant names of the children for this specific variant.
    /// Returns an empty vec for unit variants or variants without children.
    fn child_variant_names(&self) -> Vec<&'static str>;

    /// Returns the stable keys of the children for this specific variant.
    fn child_variant_keys(&self) -> Vec<&'static str>;

    /// Returns the localized labels of the children for this specific variant.
    fn child_variant_labels(&self) -> Vec<SharedString>;

    /// Creates a new instance with the child at the given index.
    /// Returns None if the variant doesn't have children or the index is out of bounds.
    fn set_child_by_index(&self, index: usize) -> Option<Self>;

    /// Creates a new instance with the child that matches the given key.
    fn set_child_by_key(&self, key: &str) -> Option<Self>;

    /// Sets the child at a given path depth recursively.
    /// `path[0]` is the child index at this level, `path[1]` the grandchild, and so on.
    fn set_child_by_path(&self, path: &[usize]) -> Option<Self>;

    /// Sets the child at a given key path recursively.
    fn set_child_by_key_path(&self, path: &[String]) -> Option<Self>;

    /// Returns the depth of nesting for this variant's children.
    /// Returns 0 for leaf variants.
    fn child_depth(&self) -> usize;

    /// Returns the maximum depth of nesting for this enum type.
    fn depth() -> usize;

    /// Returns the current selection path for this concrete value.
    ///
    /// The returned path includes the root variant index at depth 0.
    fn selection_path(&self) -> InfiniteSelectPath;

    /// Returns the current stable key path for this concrete value.
    fn selection_key_path(&self) -> InfiniteSelectKeyPath;

    /// Returns the variant names of the inner value's children.
    fn inner_child_variant_names(&self) -> Vec<&'static str>;

    /// Returns the stable keys of the inner value's children.
    fn inner_child_variant_keys(&self) -> Vec<&'static str>;

    /// Returns the localized labels of the inner value's children.
    fn inner_child_variant_labels(&self) -> Vec<SharedString>;

    /// Sets a child on the inner value and wraps it back.
    fn inner_set_child_by_index(&self, index: usize) -> Option<Self>;

    /// Sets a child on the inner value by stable key and wraps it back.
    fn inner_set_child_by_key(&self, key: &str) -> Option<Self>;

    /// Returns true if the inner value itself has children.
    fn inner_has_inner(&self) -> bool;

    /// Returns the localized label for this type (level).
    fn type_label(&self) -> SharedString;

    /// Returns the localized description for this type (level).
    fn type_description(&self) -> SharedString;

    /// Returns the localized label for the child at the given depth relative to this node.
    /// `depth = 0` is the immediate child.
    fn child_label_at_depth(&self, depth: usize) -> Option<SharedString>;

    /// Returns the localized description for the child at the given depth.
    fn child_description_at_depth(&self, depth: usize) -> Option<SharedString>;

    /// Internal method to delegate label lookup to the inner value.
    fn inner_child_label_at_depth(&self, depth: usize) -> Option<SharedString>;

    /// Internal method to delegate description lookup to the inner value.
    fn inner_child_description_at_depth(&self, depth: usize) -> Option<SharedString>;
}

/// A wrapper for infinite-select enum variants that implements `SelectItem`.
///
/// This allows infinite-select enum variants to be displayed in a select dropdown
/// while preserving access to the nested value.
#[derive(Clone)]
pub struct InfiniteSelectItem<T: InfiniteSelectValue> {
    value: T,
    title: SharedString,
}

impl<T: InfiniteSelectValue> InfiniteSelectItem<T> {
    /// Creates a new item with a custom title.
    pub fn new(value: T, title: impl Into<SharedString>) -> Self {
        Self {
            value,
            title: title.into(),
        }
    }

    /// Creates an item using `variant_label()` as the title.
    pub fn from_variant(value: T) -> Self {
        let title = value.variant_label();
        Self { value, title }
    }

    /// Returns a reference to the wrapped value.
    pub fn get_value(&self) -> &T {
        &self.value
    }

    /// Consumes the item and returns the wrapped value.
    pub fn into_value(self) -> T {
        self.value
    }

    /// Returns true if the wrapped value has nested inner values.
    pub fn has_inner(&self) -> bool {
        self.value.has_inner()
    }

    /// Returns the child variant names if the wrapped value has children.
    pub fn child_variant_names(&self) -> Vec<&'static str> {
        self.value.child_variant_names()
    }

    /// Returns the child variant keys if the wrapped value has children.
    pub fn child_variant_keys(&self) -> Vec<&'static str> {
        self.value.child_variant_keys()
    }

    /// Returns the child variant labels if the wrapped value has children.
    pub fn child_variant_labels(&self) -> Vec<SharedString> {
        self.value.child_variant_labels()
    }

    /// Returns a new item with a child selected at the given index.
    pub fn with_child_at(&self, index: usize) -> Option<Self> {
        let title = self.value.child_variant_labels().get(index).cloned()?;
        self.value
            .set_child_by_index(index)
            .map(|value| Self::new(value, title))
    }
}

impl<T: InfiniteSelectValue> SelectItem for InfiniteSelectItem<T> {
    type Value = T;

    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }
}

/// Creates root select items from `T::variants()`.
pub fn to_select_items<T>() -> Vec<InfiniteSelectItem<T>>
where
    T: InfiniteSelectValue,
{
    T::variants()
        .into_iter()
        .map(InfiniteSelectItem::from_variant)
        .collect()
}

/// Represents an index-based selection path through nested infinite-select enums.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InfiniteSelectPath {
    indices: Vec<usize>,
}

impl InfiniteSelectPath {
    /// Creates a new empty selection path.
    pub fn new() -> Self {
        Self {
            indices: Vec::new(),
        }
    }

    /// Creates a path with the given indices.
    pub fn with_indices(indices: Vec<usize>) -> Self {
        Self { indices }
    }

    /// Returns the selection index at a given depth.
    pub fn get(&self, depth: usize) -> Option<usize> {
        self.indices.get(depth).copied()
    }

    /// Sets the selection at a given depth, truncating deeper selections.
    pub fn set(&mut self, depth: usize, index: usize) {
        self.indices.truncate(depth);
        self.indices.push(index);
    }

    /// Clears selections from a given depth onwards.
    pub fn clear_from(&mut self, depth: usize) {
        self.indices.truncate(depth);
    }

    /// Truncates the path to the given length.
    pub fn truncate(&mut self, len: usize) {
        self.indices.truncate(len);
    }

    /// Returns the current depth of the selection.
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Returns all indices as a slice.
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }

    /// Returns true if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

/// Represents a key-based selection path through nested infinite-select enums.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InfiniteSelectKeyPath {
    keys: Vec<String>,
}

impl InfiniteSelectKeyPath {
    /// Creates a new empty key path.
    pub fn new() -> Self {
        Self { keys: Vec::new() }
    }

    /// Creates a key path with the given keys.
    pub fn with_keys(keys: Vec<String>) -> Self {
        Self { keys }
    }

    /// Returns the selected key at a given depth.
    pub fn get(&self, depth: usize) -> Option<&str> {
        self.keys.get(depth).map(String::as_str)
    }

    /// Sets the selected key at a given depth, truncating deeper selections.
    pub fn set(&mut self, depth: usize, key: impl Into<String>) {
        self.keys.truncate(depth);
        self.keys.push(key.into());
    }

    /// Clears selections from a given depth onwards.
    pub fn clear_from(&mut self, depth: usize) {
        self.keys.truncate(depth);
    }

    /// Truncates the path to the given length.
    pub fn truncate(&mut self, len: usize) {
        self.keys.truncate(len);
    }

    /// Returns the current depth of the selection.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Returns all keys as a slice.
    pub fn keys(&self) -> &[String] {
        &self.keys
    }

    /// Returns true if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

impl fmt::Display for InfiniteSelectKeyPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, key) in self.keys.iter().enumerate() {
            if index > 0 {
                write!(f, "/")?;
            }

            for ch in key.chars() {
                match ch {
                    '\\' => write!(f, "\\\\")?,
                    '/' => write!(f, "\\/")?,
                    _ => write!(f, "{ch}")?,
                }
            }
        }

        Ok(())
    }
}

/// A parse failure for the string form of `InfiniteSelectKeyPath`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InfiniteSelectKeyPathParseError {
    input: String,
    reason: InfiniteSelectKeyPathParseErrorReason,
}

impl InfiniteSelectKeyPathParseError {
    fn dangling_escape(input: &str) -> Self {
        Self {
            input: input.to_string(),
            reason: InfiniteSelectKeyPathParseErrorReason::DanglingEscape,
        }
    }

    /// Returns the original string input that failed to parse.
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Returns the typed parse failure reason.
    pub fn reason(&self) -> &InfiniteSelectKeyPathParseErrorReason {
        &self.reason
    }
}

impl fmt::Display for InfiniteSelectKeyPathParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.reason {
            InfiniteSelectKeyPathParseErrorReason::DanglingEscape => {
                write!(
                    f,
                    "infinite-select key path {:?} ends with an incomplete escape sequence",
                    self.input
                )
            },
        }
    }
}

impl Error for InfiniteSelectKeyPathParseError {}

/// The reason a string key path could not be parsed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InfiniteSelectKeyPathParseErrorReason {
    DanglingEscape,
}

impl FromStr for InfiniteSelectKeyPath {
    type Err = InfiniteSelectKeyPathParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty() {
            return Ok(Self::new());
        }

        let mut keys = Vec::new();
        let mut current = String::new();
        let mut escaped = false;

        for ch in value.chars() {
            if escaped {
                current.push(ch);
                escaped = false;
                continue;
            }

            match ch {
                '\\' => escaped = true,
                '/' => {
                    keys.push(std::mem::take(&mut current));
                },
                _ => current.push(ch),
            }
        }

        if escaped {
            return Err(InfiniteSelectKeyPathParseError::dangling_escape(value));
        }

        keys.push(current);
        Ok(Self::with_keys(keys))
    }
}

impl Serialize for InfiniteSelectKeyPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for InfiniteSelectKeyPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(D::Error::custom)
    }
}

/// Returns the current selection path for a concrete infinite-select value.
pub fn path_from_value<T: InfiniteSelectValue>(value: &T) -> InfiniteSelectPath {
    value.selection_path()
}

/// Returns the current key path for a concrete infinite-select value.
pub fn key_path_from_value<T: InfiniteSelectValue>(value: &T) -> InfiniteSelectKeyPath {
    value.selection_key_path()
}

/// The failing segment of an index- or key-based infinite-select path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InfiniteSelectPathSegment {
    Index(usize),
    Key(String),
}

impl InfiniteSelectPathSegment {
    /// Returns the segment as an index when this is an index-based path error.
    pub fn as_index(&self) -> Option<usize> {
        match self {
            Self::Index(index) => Some(*index),
            Self::Key(_) => None,
        }
    }

    /// Returns the segment as a key when this is a key-based path error.
    pub fn as_key(&self) -> Option<&str> {
        match self {
            Self::Index(_) => None,
            Self::Key(key) => Some(key),
        }
    }
}

/// The reason an infinite-select path failed to resolve.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InfiniteSelectPathErrorReason {
    EmptyPath,
    MissingSelectionOptions,
    InvalidIndex { option_count: usize },
    UnknownKey { available_keys: Vec<String> },
}

/// A typed path-resolution failure for infinite-select helpers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InfiniteSelectPathError {
    depth: usize,
    segment: Option<InfiniteSelectPathSegment>,
    reason: InfiniteSelectPathErrorReason,
}

impl InfiniteSelectPathError {
    fn empty() -> Self {
        Self {
            depth: 0,
            segment: None,
            reason: InfiniteSelectPathErrorReason::EmptyPath,
        }
    }

    fn missing_selection_options(depth: usize, segment: InfiniteSelectPathSegment) -> Self {
        Self {
            depth,
            segment: Some(segment),
            reason: InfiniteSelectPathErrorReason::MissingSelectionOptions,
        }
    }

    fn invalid_index(depth: usize, index: usize, option_count: usize) -> Self {
        Self {
            depth,
            segment: Some(InfiniteSelectPathSegment::Index(index)),
            reason: InfiniteSelectPathErrorReason::InvalidIndex { option_count },
        }
    }

    fn unknown_key(depth: usize, key: &str, available_keys: Vec<String>) -> Self {
        Self {
            depth,
            segment: Some(InfiniteSelectPathSegment::Key(key.to_string())),
            reason: InfiniteSelectPathErrorReason::UnknownKey { available_keys },
        }
    }

    /// Returns the depth where path resolution failed.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Returns the failing path segment, when available.
    pub fn segment(&self) -> Option<&InfiniteSelectPathSegment> {
        self.segment.as_ref()
    }

    /// Returns the typed failure reason.
    pub fn reason(&self) -> &InfiniteSelectPathErrorReason {
        &self.reason
    }
}

impl fmt::Display for InfiniteSelectPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.segment, &self.reason) {
            (None, InfiniteSelectPathErrorReason::EmptyPath) => {
                write!(f, "infinite-select path is empty")
            },
            (
                Some(InfiniteSelectPathSegment::Index(index)),
                InfiniteSelectPathErrorReason::MissingSelectionOptions,
            ) => {
                write!(
                    f,
                    "no selectable options exist at depth {} for index {}",
                    self.depth, index
                )
            },
            (
                Some(InfiniteSelectPathSegment::Key(key)),
                InfiniteSelectPathErrorReason::MissingSelectionOptions,
            ) => {
                write!(
                    f,
                    "no selectable options exist at depth {} for key {:?}",
                    self.depth, key
                )
            },
            (
                Some(InfiniteSelectPathSegment::Index(index)),
                InfiniteSelectPathErrorReason::InvalidIndex { option_count },
            ) => write!(
                f,
                "index {} is out of bounds at depth {} ({} options available)",
                index, self.depth, option_count
            ),
            (
                Some(InfiniteSelectPathSegment::Key(key)),
                InfiniteSelectPathErrorReason::UnknownKey { available_keys },
            ) => write!(
                f,
                "key {:?} is not valid at depth {} (available keys: {:?})",
                key, self.depth, available_keys
            ),
            _ => write!(f, "invalid infinite-select path at depth {}", self.depth),
        }
    }
}

impl Error for InfiniteSelectPathError {}

/// Rebuilds a value from an index-based selection path.
pub fn build_from_path<T: InfiniteSelectValue>(
    path: &InfiniteSelectPath,
) -> Result<T, InfiniteSelectPathError> {
    if path.is_empty() {
        return Err(InfiniteSelectPathError::empty());
    }

    let variants = T::variants();
    let root_index = path
        .get(0)
        .expect("non-empty paths include the root selection");
    let Some(mut current_value) = variants.get(root_index).cloned() else {
        return Err(InfiniteSelectPathError::invalid_index(
            0,
            root_index,
            variants.len(),
        ));
    };

    for depth in 1..path.len() {
        let index = path
            .get(depth)
            .expect("path length guarantees a selection at each iterated depth");
        let items = child_items_for_level(&current_value, depth - 1);

        if items.is_empty() {
            return Err(InfiniteSelectPathError::missing_selection_options(
                depth,
                InfiniteSelectPathSegment::Index(index),
            ));
        }

        let Some(item) = items.get(index) else {
            return Err(InfiniteSelectPathError::invalid_index(
                depth,
                index,
                items.len(),
            ));
        };

        current_value = item.get_value().clone();
    }

    Ok(current_value)
}

/// Rebuilds a value from a key-based selection path.
pub fn build_from_key_path<T: InfiniteSelectValue>(
    path: &InfiniteSelectKeyPath,
) -> Result<T, InfiniteSelectPathError> {
    if path.is_empty() {
        return Err(InfiniteSelectPathError::empty());
    }

    let root_key = path
        .get(0)
        .expect("non-empty key paths include the root selection");
    let variants = T::variants();
    let Some(mut current_value) = variants
        .iter()
        .find(|variant| variant.variant_key() == root_key)
        .cloned()
    else {
        return Err(InfiniteSelectPathError::unknown_key(
            0,
            root_key,
            variants
                .iter()
                .map(|variant| variant.variant_key().to_string())
                .collect(),
        ));
    };

    for depth in 1..path.len() {
        let key = path
            .get(depth)
            .expect("path length guarantees a selection at each iterated depth");
        let items = child_items_for_level(&current_value, depth - 1);

        if items.is_empty() {
            return Err(InfiniteSelectPathError::missing_selection_options(
                depth,
                InfiniteSelectPathSegment::Key(key.to_string()),
            ));
        }

        let available_keys: Vec<String> = items
            .iter()
            .filter_map(|item| {
                item.get_value()
                    .selection_key_path()
                    .get(depth)
                    .map(str::to_string)
            })
            .collect();

        let Some(item) = items.iter().find(|item| {
            item.get_value()
                .selection_key_path()
                .get(depth)
                .is_some_and(|candidate| candidate == key)
        }) else {
            return Err(InfiniteSelectPathError::unknown_key(
                depth,
                key,
                available_keys,
            ));
        };

        current_value = item.get_value().clone();
    }

    Ok(current_value)
}

/// Options for the runtime `Select`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InfiniteSelectOptions {
    searchable: bool,
    max_depth: Option<usize>,
}

impl InfiniteSelectOptions {
    /// Creates runtime options for `Select`.
    pub const fn new(searchable: bool, max_depth: Option<usize>) -> Self {
        Self {
            searchable,
            max_depth,
        }
    }
}

/// Event emitted by `Select` whenever the selection changes.
#[derive(Clone)]
pub struct InfiniteSelectEvent<T: InfiniteSelectValue> {
    previous_value: T,
    previous_path: InfiniteSelectPath,
    previous_key_path: InfiniteSelectKeyPath,
    value: T,
    path: InfiniteSelectPath,
    key_path: InfiniteSelectKeyPath,
    changed_depth: usize,
}

impl<T: InfiniteSelectValue> InfiniteSelectEvent<T> {
    /// Returns the concrete selection value before this change.
    pub fn previous_value(&self) -> &T {
        &self.previous_value
    }

    /// Returns the index path before this change.
    pub fn previous_path(&self) -> &InfiniteSelectPath {
        &self.previous_path
    }

    /// Returns the key path before this change.
    pub fn previous_key_path(&self) -> &InfiniteSelectKeyPath {
        &self.previous_key_path
    }

    /// Returns the rebuilt concrete selection value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Consumes the event and returns the rebuilt concrete selection value.
    pub fn into_value(self) -> T {
        self.value
    }

    /// Returns the current index path.
    pub fn path(&self) -> &InfiniteSelectPath {
        &self.path
    }

    /// Returns the current key path.
    pub fn key_path(&self) -> &InfiniteSelectKeyPath {
        &self.key_path
    }

    /// Returns the first depth that changed for this selection event.
    pub fn changed_depth(&self) -> usize {
        self.changed_depth
    }
}

/// A single rendered level of an infinite-select field.
#[derive(Clone)]
pub struct InfiniteSelectLevel<D>
where
    D: SelectDelegate + 'static,
{
    depth: usize,
    label: SharedString,
    description: SharedString,
    select: Entity<SelectState<D>>,
    selected_index: Option<usize>,
    selected_key: Option<String>,
}

impl<D> InfiniteSelectLevel<D>
where
    D: SelectDelegate + 'static,
{
    /// Returns the rendered depth for this select level.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Returns true when this level is the root select.
    pub fn is_root(&self) -> bool {
        self.depth == 0
    }

    /// Returns the field label for this level.
    pub fn label(&self) -> &SharedString {
        &self.label
    }

    /// Returns the field description for this level.
    pub fn description(&self) -> &SharedString {
        &self.description
    }

    /// Returns the backing select entity for this level.
    pub fn select(&self) -> Entity<SelectState<D>> {
        self.select.clone()
    }

    /// Returns the selected index for this level, when available.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Returns the selected stable key for this level, when available.
    pub fn selected_key(&self) -> Option<&str> {
        self.selected_key.as_deref()
    }

    /// Builds a `gpui_component::form::Field` for this select level.
    pub fn to_form_field(&self) -> Field {
        let label = self.label.clone();
        let description = self.description.clone();
        let select = self.select.clone();

        field()
            .label(label)
            .description_fn(move |_, _| {
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().child(description.clone()))
            })
            .child(GpuiSelect::new(&select))
    }
}

/// An owned snapshot of the current infinite-select runtime state.
#[derive(Clone)]
pub struct InfiniteSelectSnapshot<T, D>
where
    D: SelectDelegate + 'static,
{
    value: T,
    path: InfiniteSelectPath,
    key_path: InfiniteSelectKeyPath,
    levels: Vec<InfiniteSelectLevel<D>>,
}

impl<T, D> InfiniteSelectSnapshot<T, D>
where
    T: InfiniteSelectValue,
    D: SelectDelegate + 'static,
{
    /// Returns the concrete selected value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns the current index-based selection path.
    pub fn path(&self) -> &InfiniteSelectPath {
        &self.path
    }

    /// Returns the current key-based selection path.
    pub fn key_path(&self) -> &InfiniteSelectKeyPath {
        &self.key_path
    }

    /// Returns the rendered select levels in root-to-leaf order.
    pub fn levels(&self) -> &[InfiniteSelectLevel<D>] {
        &self.levels
    }

    /// Consumes the snapshot and returns the owned level list.
    pub fn into_levels(self) -> Vec<InfiniteSelectLevel<D>> {
        self.levels
    }

    /// Returns render-ready GPUI form fields for each select level.
    pub fn form_fields(&self) -> Vec<Field> {
        self.levels
            .iter()
            .map(InfiniteSelectLevel::to_form_field)
            .collect()
    }
}

/// Runtime state for a cascading infinite-select field.
pub struct InfiniteSelectState<T, D = Vec<InfiniteSelectItem<T>>>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    value: T,
    path: InfiniteSelectPath,
    key_path: InfiniteSelectKeyPath,
    master_select: Entity<SelectState<D>>,
    child_selects: Vec<Entity<SelectState<D>>>,
    options: InfiniteSelectOptions,
    _master_subscription: Subscription,
    _child_subscriptions: Vec<Subscription>,
}

impl<T, D> Focusable for InfiniteSelectState<T, D>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.master_select.read(cx).focus_handle(cx)
    }
}

impl<T, D> EventEmitter<InfiniteSelectEvent<T>> for InfiniteSelectState<T, D>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
}

#[bon::bon]
impl<T, D> InfiniteSelectState<T, D>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    /// Starts a bon-style option chain for `#[gpui_form(...)]`.
    #[builder(start_fn = builder, finish_fn = build)]
    pub fn options(
        #[builder(default)] searchable: bool,
        max_depth: Option<usize>,
    ) -> InfiniteSelectOptions {
        InfiniteSelectOptions {
            searchable,
            max_depth,
        }
    }

    /// Creates a new state from `T::default()`.
    pub fn new_default(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::new(T::default(), window, cx)
    }

    /// Creates a new state from the given initial value.
    pub fn new(initial_value: T, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::new_with_options(initial_value, InfiniteSelectOptions::default(), window, cx)
    }

    /// Creates a new state with explicit options.
    pub fn new_with_options(
        initial_value: T,
        options: InfiniteSelectOptions,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let path = path_from_value(&initial_value);
        let key_path = key_path_from_value(&initial_value);
        let root_selected = path.get(0);
        let master_select = cx.new(|cx| {
            build_select_state::<T, D>(
                to_select_items::<T>(),
                root_selected,
                options.searchable,
                window,
                cx,
            )
        });
        let master_subscription = cx.subscribe_in(&master_select, window, Self::on_select_event);

        let mut this = Self {
            value: initial_value,
            path,
            key_path,
            master_select,
            child_selects: Vec::new(),
            options,
            _master_subscription: master_subscription,
            _child_subscriptions: Vec::new(),
        };
        this.rebuild_child_selects(window, cx);
        this
    }

    /// Returns the current concrete selection.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns the current index-based selection path.
    pub fn path(&self) -> &InfiniteSelectPath {
        &self.path
    }

    /// Returns the current key-based selection path.
    pub fn key_path(&self) -> &InfiniteSelectKeyPath {
        &self.key_path
    }

    /// Returns the selected index at the given depth.
    pub fn selected_index_at_depth(&self, depth: usize) -> Option<usize> {
        self.path.get(depth)
    }

    /// Returns the selected key at the given depth.
    pub fn selected_key_at_depth(&self, depth: usize) -> Option<&str> {
        self.key_path.get(depth)
    }

    /// Returns the current rendered select levels in root-to-leaf order.
    pub fn levels(&self) -> Vec<InfiniteSelectLevel<D>> {
        build_levels(
            &self.value,
            &self.path,
            &self.key_path,
            &self.master_select,
            &self.child_selects,
        )
    }

    /// Returns an owned snapshot of the value, paths, and rendered levels.
    pub fn snapshot(&self) -> InfiniteSelectSnapshot<T, D> {
        InfiniteSelectSnapshot {
            value: self.value.clone(),
            path: self.path.clone(),
            key_path: self.key_path.clone(),
            levels: self.levels(),
        }
    }

    /// Returns render-ready GPUI form fields for each visible select level.
    pub fn form_fields(&self) -> Vec<Field> {
        self.levels()
            .into_iter()
            .map(|level| level.to_form_field())
            .collect()
    }

    /// Returns the root select entity.
    pub fn master_select(&self) -> Entity<SelectState<D>> {
        self.master_select.clone()
    }

    /// Returns the currently visible child selects.
    pub fn child_selects(&self) -> Vec<Entity<SelectState<D>>> {
        self.child_selects.clone()
    }

    /// Programmatically sets the current selection.
    pub fn set_value(&mut self, value: T, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_selection(value, None, false, window, cx);
    }

    /// Programmatically sets the current selection from an index path.
    pub fn set_path(
        &mut self,
        path: &InfiniteSelectPath,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), InfiniteSelectPathError> {
        let value = build_from_path::<T>(path)?;
        self.apply_selection(value, None, false, window, cx);
        Ok(())
    }

    /// Programmatically sets the current selection from a key path.
    pub fn set_key_path(
        &mut self,
        key_path: &InfiniteSelectKeyPath,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), InfiniteSelectPathError> {
        let value = build_from_key_path::<T>(key_path)?;
        self.apply_selection(value, None, false, window, cx);
        Ok(())
    }

    /// Programmatically changes one selection index and resets deeper levels to defaults.
    pub fn set_selected_index_at_depth(
        &mut self,
        depth: usize,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), InfiniteSelectPathError> {
        if depth > self.path.len() {
            return Err(InfiniteSelectPathError::missing_selection_options(
                depth,
                InfiniteSelectPathSegment::Index(index),
            ));
        }

        let mut path = self.path.clone();
        path.set(depth, index);
        self.set_path(&path, window, cx)
    }

    /// Programmatically changes one selection key and resets deeper levels to defaults.
    pub fn set_selected_key_at_depth(
        &mut self,
        depth: usize,
        key: impl Into<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), InfiniteSelectPathError> {
        let key = key.into();
        if depth > self.key_path.len() {
            return Err(InfiniteSelectPathError::missing_selection_options(
                depth,
                InfiniteSelectPathSegment::Key(key),
            ));
        }

        let mut key_path = self.key_path.clone();
        key_path.set(depth, key);
        self.set_key_path(&key_path, window, cx)
    }

    fn resolved_max_depth(&self) -> usize {
        match self.options.max_depth {
            Some(max_depth) => max_depth.clamp(1, T::depth()),
            None => T::depth(),
        }
    }

    fn on_select_event(
        &mut self,
        this: &Entity<SelectState<D>>,
        event: &SelectEvent<D>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let SelectEvent::Confirm(Some(selected)) = event else {
            return;
        };

        let changed_depth = self
            .select_depth(this)
            .or_else(|| Some(first_changed_depth(&self.path, &selected.selection_path())));

        self.apply_selection(selected.clone(), changed_depth, true, window, cx);
    }

    fn select_depth(&self, this: &Entity<SelectState<D>>) -> Option<usize> {
        if &self.master_select == this {
            Some(0)
        } else {
            self.child_selects
                .iter()
                .position(|child| child == this)
                .map(|position| position + 1)
        }
    }

    fn apply_selection(
        &mut self,
        value: T,
        changed_depth: Option<usize>,
        emit: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let previous_value = self.value.clone();
        let previous_path = self.path.clone();
        let previous_key_path = self.key_path.clone();
        let new_path = path_from_value(&value);
        let new_key_path = key_path_from_value(&value);
        let changed_depth =
            changed_depth.unwrap_or_else(|| first_changed_depth(&previous_path, &new_path));

        self.value = value.clone();
        self.path = new_path.clone();
        self.key_path = new_key_path.clone();
        self.sync_master_select(window, cx);
        self.rebuild_child_selects(window, cx);
        if emit {
            cx.emit(InfiniteSelectEvent {
                previous_value,
                previous_path,
                previous_key_path,
                value,
                path: new_path,
                key_path: new_key_path,
                changed_depth,
            });
        }
        cx.notify();
    }

    fn sync_master_select(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let selected_index = selected_index(self.path.get(0).unwrap_or(0));
        self.master_select.update(cx, |state, cx| {
            state.set_selected_index(selected_index, window, cx);
        });
    }

    fn rebuild_child_selects(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let child_selects = build_child_selects::<T, D>(
            &self.value,
            &self.path,
            self.resolved_max_depth(),
            self.options.searchable,
            window,
            cx,
        );

        self._child_subscriptions = child_selects
            .iter()
            .map(|child| cx.subscribe_in(child, window, Self::on_select_event))
            .collect();
        self.child_selects = child_selects;
    }
}

#[allow(unnameable_types)]
impl<T, D> InfiniteSelectState<T, D>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    /// Starts an option chain with search enabled.
    pub fn searchable(
        value: bool,
    ) -> InfiniteSelectStateOptionsBuilder<T, D, infinite_select_state_options_builder::SetSearchable>
    {
        Self::builder().searchable(value)
    }

    /// Starts an option chain with a max depth.
    pub fn max_depth(
        value: usize,
    ) -> InfiniteSelectStateOptionsBuilder<T, D, infinite_select_state_options_builder::SetMaxDepth>
    {
        Self::builder().max_depth(value)
    }
}

/// Render wrapper used by generated form code for infinite-select fields.
#[cfg_attr(feature = "component-shape", derive(gpui_form_derive::ComponentShape))]
#[cfg_attr(
    feature = "component-shape",
    gpui_form_shape(
        state = InfiniteSelectState<T, D>,
        new = InfiniteSelectState::<T, D>::new_default,
        requires_value = false,
        field_suffix = "infinite_select"
    )
)]
#[derive(IntoElement)]
pub struct InfiniteSelect<T, D = Vec<InfiniteSelectItem<T>>>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    state: Entity<InfiniteSelectState<T, D>>,
}

impl<T, D> InfiniteSelect<T, D>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    pub fn new(state: &Entity<InfiniteSelectState<T, D>>) -> Self {
        Self {
            state: state.clone(),
        }
    }
}

impl<T, D> RenderOnce for InfiniteSelect<T, D>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div().children(self.state.read(cx).form_fields())
    }
}

#[cfg(feature = "component-shape")]
#[gpui_form_derive::component_value_binding]
impl<T, D> gpui_form_runtime::shape::ComponentValueBinding<T> for InfiniteSelectState<T, D>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    type Event = InfiniteSelectEvent<T>;

    fn seed_value_binding_state(
        state: &mut Self::State,
        value: Option<&T>,
        window: &mut Window,
        cx: &mut Context<'_, Self::State>,
    ) {
        if let Some(value) = value {
            state.set_value(value.clone(), window, cx);
        }
    }

    fn form_value_change(
        _state: &Self::State,
        event: &Self::Event,
    ) -> gpui_form_runtime::shape::FormValueChange<T> {
        gpui_form_runtime::shape::FormValueChange::Set(event.value().clone())
    }
}

impl<T, D> Render for InfiniteSelectState<T, D>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

fn build_child_selects<T, D>(
    parent: &T,
    path: &InfiniteSelectPath,
    max_depth: usize,
    searchable: bool,
    window: &mut Window,
    cx: &mut Context<InfiniteSelectState<T, D>>,
) -> Vec<Entity<SelectState<D>>>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    let mut current_value = parent.clone();
    let mut selects = Vec::new();

    for level in 0..max_depth.saturating_sub(1) {
        let items = child_items_for_level(&current_value, level);
        if items.is_empty() {
            break;
        }

        let selected_row = path.get(level + 1).unwrap_or(0).min(items.len() - 1);
        let child_select = cx.new(|cx| {
            build_select_state::<T, D>(items.clone(), Some(selected_row), searchable, window, cx)
        });
        current_value = items[selected_row].get_value().clone();
        selects.push(child_select);
    }

    selects
}

fn build_levels<T, D>(
    value: &T,
    path: &InfiniteSelectPath,
    key_path: &InfiniteSelectKeyPath,
    master_select: &Entity<SelectState<D>>,
    child_selects: &[Entity<SelectState<D>>],
) -> Vec<InfiniteSelectLevel<D>>
where
    T: InfiniteSelectValue,
    D: SelectDelegate + 'static,
{
    let mut levels = Vec::with_capacity(child_selects.len() + 1);
    levels.push(InfiniteSelectLevel {
        depth: 0,
        label: value.type_label(),
        description: value.type_description(),
        select: master_select.clone(),
        selected_index: path.get(0),
        selected_key: key_path.get(0).map(str::to_string),
    });

    levels.extend(child_selects.iter().enumerate().map(|(index, select)| {
        let depth = index + 1;
        InfiniteSelectLevel {
            depth,
            label: value
                .child_label_at_depth(index)
                .unwrap_or_else(|| "".into()),
            description: value
                .child_description_at_depth(index)
                .unwrap_or_else(|| "".into()),
            select: select.clone(),
            selected_index: path.get(depth),
            selected_key: key_path.get(depth).map(str::to_string),
        }
    }));

    levels
}

fn child_items_for_level<T: InfiniteSelectValue>(
    current_value: &T,
    level: usize,
) -> Vec<InfiniteSelectItem<T>> {
    let (has_more, child_labels) = if level == 0 {
        (
            current_value.has_inner(),
            current_value.child_variant_labels(),
        )
    } else {
        (
            current_value.inner_has_inner(),
            current_value.inner_child_variant_labels(),
        )
    };

    if !has_more || child_labels.is_empty() {
        return Vec::new();
    }

    child_labels
        .into_iter()
        .enumerate()
        .filter_map(|(index, title)| {
            let value = if level == 0 {
                current_value.set_child_by_index(index)
            } else {
                current_value.inner_set_child_by_index(index)
            };
            value.map(|value| InfiniteSelectItem::new(value, title))
        })
        .collect()
}

fn build_select_state<T, D>(
    items: Vec<InfiniteSelectItem<T>>,
    selected_row: Option<usize>,
    searchable: bool,
    window: &mut Window,
    cx: &mut Context<SelectState<D>>,
) -> SelectState<D>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    let mut state = SelectState::new(
        items.into(),
        selected_row.and_then(selected_index),
        window,
        cx,
    );
    if searchable {
        state = state.searchable(true);
    }
    state
}

fn selected_index(row: usize) -> Option<IndexPath> {
    Some(IndexPath {
        section: 0,
        row,
        column: 0,
    })
}

fn first_changed_depth(previous: &InfiniteSelectPath, next: &InfiniteSelectPath) -> usize {
    let max_depth = previous.len().max(next.len());
    for depth in 0..max_depth {
        if previous.get(depth) != next.get(depth) {
            return depth;
        }
    }

    max_depth.saturating_sub(1)
}
