//! Infinite-select runtime support for nested enum trees.
//!
//! This module exposes both the low-level trait and path helpers used by
//! generated code and a higher-level `Select` entity that owns the
//! cascading `SelectState`s for a field.

mod path;
mod state_helpers;
mod value;

pub use path::{
    InfiniteSelectKeyPath, InfiniteSelectKeyPathParseError, InfiniteSelectKeyPathParseErrorReason,
    InfiniteSelectPath, InfiniteSelectPathError, InfiniteSelectPathErrorReason,
    InfiniteSelectPathSegment, build_from_key_path, build_from_path, key_path_from_value,
    path_from_value,
};
use state_helpers::{
    build_child_selects, build_levels, build_select_state, first_changed_depth, selected_index,
};
pub use value::{InfiniteSelectItem, InfiniteSelectValue, to_select_items};

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

/// Options for the runtime `Select`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "component-shape", derive(bon::Builder))]
pub struct InfiniteSelectOptions {
    #[cfg_attr(feature = "component-shape", builder(default))]
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

impl<T, D> InfiniteSelectState<T, D>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
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
                to_select_items::<T>(cx),
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
    pub fn levels(&self, cx: &impl std::borrow::Borrow<App>) -> Vec<InfiniteSelectLevel<D>> {
        build_levels(
            &self.value,
            &self.path,
            &self.key_path,
            &self.master_select,
            &self.child_selects,
            cx,
        )
    }

    /// Returns an owned snapshot of the value, paths, and rendered levels.
    pub fn snapshot(&self, cx: &impl std::borrow::Borrow<App>) -> InfiniteSelectSnapshot<T, D> {
        InfiniteSelectSnapshot {
            value: self.value.clone(),
            path: self.path.clone(),
            key_path: self.key_path.clone(),
            levels: self.levels(cx),
        }
    }

    /// Returns render-ready GPUI form fields for each visible select level.
    pub fn form_fields(&self, cx: &impl std::borrow::Borrow<App>) -> Vec<Field> {
        self.levels(cx)
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

/// Render wrapper used by generated form code for infinite-select fields.
#[cfg_attr(
    feature = "component-shape",
    derive(component_shape_gpui::GpuiComponentShape)
)]
#[cfg_attr(
    feature = "component-shape",
    gpui_component_shape(
        state = InfiniteSelectState<T, D>,
        new = InfiniteSelectState::<T, D>::new_default,
        value = T,
        field_suffix = "infinite_select",
        value_binding
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

#[cfg(feature = "component-shape")]
impl<T, D> gpui_form_runtime::shape::GpuiFormComponentShapePolicy for InfiniteSelect<T, D>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}

impl<T, D> InfiniteSelect<T, D>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    #[cfg(feature = "component-shape")]
    pub fn searchable(searchable: bool) -> InfiniteSelectOptions {
        InfiniteSelectOptions::builder()
            .searchable(searchable)
            .build()
    }

    #[cfg(feature = "component-shape")]
    pub fn from(options: InfiniteSelectOptions) -> InfiniteSelectOptions {
        options
    }

    pub fn new(state: &Entity<InfiniteSelectState<T, D>>) -> Self {
        Self {
            state: state.clone(),
        }
    }
}

#[cfg(feature = "component-shape")]
impl<T, D> component_shape_gpui::GpuiComponentShapeBuilder<InfiniteSelect<T, D>>
    for InfiniteSelectOptions
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    fn build(
        self,
        window: &mut Window,
        cx: &mut Context<'_, InfiniteSelectState<T, D>>,
    ) -> InfiniteSelectState<T, D> {
        InfiniteSelectState::<T, D>::new_with_options(T::default(), self, window, cx)
    }
}

impl<T, D> RenderOnce for InfiniteSelect<T, D>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div().children(self.state.read(cx).form_fields(cx))
    }
}

#[cfg(feature = "component-shape")]
impl<T, D> gpui_form_runtime::shape::GpuiComponentStateValueBinding<T> for InfiniteSelectState<T, D>
where
    T: InfiniteSelectValue,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    type Event = InfiniteSelectEvent<T>;

    fn seed_value_binding_state(
        state: &mut Self,
        value: Option<&T>,
        window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        if let Some(value) = value {
            state.set_value(value.clone(), window, cx);
        }
    }

    fn value_change(
        _state: &Self,
        event: &Self::Event,
    ) -> gpui_form_runtime::shape::ValueChange<T> {
        gpui_form_runtime::shape::ValueChange::Set(event.value().clone())
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
