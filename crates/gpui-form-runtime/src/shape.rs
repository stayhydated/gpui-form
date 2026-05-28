//! Runtime contract for component shapes used by `#[derive(GpuiForm)]`.
//!
//! Users define a zero-sized "shape" type that implements [`ComponentShape`].
//! The derive macro uses that shape to generate:
//! - `FormFields` entity state type
//! - `FormComponents` constructor function body
//!
//! Prefer using `gpui_form_derive::component_shape!` for reusable or generic
//! wrapper shapes, and `#[derive(gpui_form_derive::ComponentShape)]` on owned
//! rendered component types with explicit `state = ...` metadata.

/// Shape contract for user-defined components.
///
/// Implementations provide the component state type and how to construct it.
pub trait ComponentShape {
    /// Backing gpui component state type.
    type State: 'static;

    /// Shape-owned policy for whether non-optional source fields keep a
    /// missing-value state in the generated value holder.
    type RequiredValuePolicy: ComponentRequiredValuePolicy;

    /// Shape-owned policy for whether generated prototyping code should
    /// inherit value binding by default.
    type ValueBindingPolicy: ComponentValueBindingPolicy;

    /// Build the component state.
    fn new(window: &mut gpui::Window, cx: &mut gpui::Context<'_, Self::State>) -> Self::State;

    /// Optional UI component type (e.g. `"TagsInput"` or `"Combobox<_>"`).
    ///
    /// When set here via `gpui_form_derive::component_shape!` or
    /// `#[gpui_form_shape(component = ...)]`, the prototyping code generator can
    /// emit `Component::new(&entity)` without requiring component UI metadata to
    /// be repeated on every field annotation.
    ///
    /// A `.component(...)` override on the field shape expression always takes
    /// precedence.
    const COMPONENT_TYPE: Option<&'static str> = None;

    /// Metadata used by prototyping generators.
    ///
    /// This is intentionally separate from runtime construction so reusable
    /// component crates can describe generated-code preferences once, and
    /// downstream fields can inherit those preferences.
    const PROTOTYPING: ComponentPrototyping = ComponentPrototyping::new();
}

mod sealed {
    pub trait RequiredValuePolicy {}
    pub trait ValueBindingPolicy {}
}

/// Marker trait for a component shape's generated value-holder storage policy.
pub trait ComponentRequiredValuePolicy: sealed::RequiredValuePolicy {
    /// Whether a missing generated value-holder value is invalid by default.
    const REQUIRES_VALUE: bool;
}

/// Marker trait for a component shape's inherited value-binding policy.
pub trait ComponentValueBindingPolicy: sealed::ValueBindingPolicy {
    /// Whether generated prototyping metadata should use value binding.
    const VALUE_BINDING: bool;
}

/// Do not inherit value binding from the shape.
pub struct NoComponentValueBinding;

impl sealed::ValueBindingPolicy for NoComponentValueBinding {}

impl ComponentValueBindingPolicy for NoComponentValueBinding {
    const VALUE_BINDING: bool = false;
}

/// Inherit value binding from the shape.
pub struct InheritedComponentValueBinding;

impl sealed::ValueBindingPolicy for InheritedComponentValueBinding {}

impl ComponentValueBindingPolicy for InheritedComponentValueBinding {
    const VALUE_BINDING: bool = true;
}

/// Assert the trait requirements implied by a shape's value-binding policy.
pub trait AssertComponentValueBindingPolicy<Shape, Value>: ComponentValueBindingPolicy {
    fn assert_component_value_binding_policy();
}

impl<Shape, Value> AssertComponentValueBindingPolicy<Shape, Value> for NoComponentValueBinding
where
    Shape: ComponentShape,
{
    fn assert_component_value_binding_policy() {}
}

impl<Shape, Value> AssertComponentValueBindingPolicy<Shape, Value>
    for InheritedComponentValueBinding
where
    Shape: ComponentValueBinding<Value>,
    ComponentStateOf<Shape>: gpui::EventEmitter<ComponentEventOf<Shape, Value>>,
{
    fn assert_component_value_binding_policy() {}
}

/// Store non-optional source fields as `Option<T>` and treat `None` as missing.
pub struct RequireValue;

impl sealed::RequiredValuePolicy for RequireValue {}

impl ComponentRequiredValuePolicy for RequireValue {
    const REQUIRES_VALUE: bool = true;
}

/// Store non-optional source fields directly as `T`.
pub struct AllowMissingValue;

impl sealed::RequiredValuePolicy for AllowMissingValue {}

impl ComponentRequiredValuePolicy for AllowMissingValue {
    const REQUIRES_VALUE: bool = false;
}

/// Storage behavior used by generated form value holders.
///
/// This lets `#[derive(GpuiForm)]` defer the `T` vs `Option<T>` choice to the
/// component shape that owns the policy while still emitting concrete holder
/// conversion code.
pub trait ValueHolderStorage<T>: ComponentRequiredValuePolicy {
    /// Concrete generated value-holder field storage.
    type Storage;

    /// Construct a missing/default value-holder field.
    fn default_storage() -> Self::Storage;

    /// Construct storage from a present form value.
    fn present(value: T) -> Self::Storage;

    /// Construct storage from a present form value, allowing `RequireValue` to
    /// encode a value equal to the declared default as missing.
    fn present_unless_default(value: T, default: T) -> Self::Storage
    where
        T: PartialEq;

    /// Convert storage into an output value, using `missing` only for policies
    /// that can represent missing values.
    fn map_into_value<Output, Present, Missing>(
        storage: Self::Storage,
        present: Present,
        missing: Missing,
    ) -> Output
    where
        Present: FnOnce(T) -> Output,
        Missing: FnOnce() -> Output;

    /// Fallible variant of [`ValueHolderStorage::map_into_value`].
    fn try_map_into_value<Output, Error, Present>(
        storage: Self::Storage,
        present: Present,
        missing: Error,
    ) -> Result<Output, Error>
    where
        Present: FnOnce(T) -> Output;

    /// Clone and map a present value, returning `None` only for policies that
    /// can represent missing values.
    fn map_present_cloned<Output, Present>(
        storage: &Self::Storage,
        present: Present,
    ) -> Option<Output>
    where
        T: Clone,
        Present: FnOnce(T) -> Output;

    /// Return whether storage contains a source value.
    ///
    /// Generated validation uses this to make shape-owned requiredness visible
    /// to `validate()` without hard-coding whether the policy stores `Option<T>`
    /// or `T`.
    fn is_present(storage: &Self::Storage) -> bool;
}

impl<T> ValueHolderStorage<T> for RequireValue {
    type Storage = Option<T>;

    fn default_storage() -> Self::Storage {
        None
    }

    fn present(value: T) -> Self::Storage {
        Some(value)
    }

    fn present_unless_default(value: T, default: T) -> Self::Storage
    where
        T: PartialEq,
    {
        if value == default { None } else { Some(value) }
    }

    fn map_into_value<Output, Present, Missing>(
        storage: Self::Storage,
        present: Present,
        missing: Missing,
    ) -> Output
    where
        Present: FnOnce(T) -> Output,
        Missing: FnOnce() -> Output,
    {
        match storage {
            Some(value) => present(value),
            None => missing(),
        }
    }

    fn try_map_into_value<Output, Error, Present>(
        storage: Self::Storage,
        present: Present,
        missing: Error,
    ) -> Result<Output, Error>
    where
        Present: FnOnce(T) -> Output,
    {
        storage.map(present).ok_or(missing)
    }

    fn map_present_cloned<Output, Present>(
        storage: &Self::Storage,
        present: Present,
    ) -> Option<Output>
    where
        T: Clone,
        Present: FnOnce(T) -> Output,
    {
        storage.clone().map(present)
    }

    fn is_present(storage: &Self::Storage) -> bool {
        storage.is_some()
    }
}

impl<T: Default> ValueHolderStorage<T> for AllowMissingValue {
    type Storage = T;

    fn default_storage() -> Self::Storage {
        T::default()
    }

    fn present(value: T) -> Self::Storage {
        value
    }

    fn present_unless_default(value: T, _default: T) -> Self::Storage
    where
        T: PartialEq,
    {
        value
    }

    fn map_into_value<Output, Present, Missing>(
        storage: Self::Storage,
        present: Present,
        _missing: Missing,
    ) -> Output
    where
        Present: FnOnce(T) -> Output,
        Missing: FnOnce() -> Output,
    {
        present(storage)
    }

    fn try_map_into_value<Output, Error, Present>(
        storage: Self::Storage,
        present: Present,
        _missing: Error,
    ) -> Result<Output, Error>
    where
        Present: FnOnce(T) -> Output,
    {
        Ok(present(storage))
    }

    fn map_present_cloned<Output, Present>(
        storage: &Self::Storage,
        present: Present,
    ) -> Option<Output>
    where
        T: Clone,
        Present: FnOnce(T) -> Output,
    {
        Some(present(storage.clone()))
    }

    fn is_present(_storage: &Self::Storage) -> bool {
        true
    }
}

/// Shape-owned metadata for prototyping generators.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentPrototyping {
    /// Preferred generated field/helper suffix, such as `"input"` or `"select"`.
    pub field_suffix: Option<&'static str>,
}

impl ComponentPrototyping {
    pub const fn new() -> Self {
        Self { field_suffix: None }
    }

    pub const fn field_suffix(mut self, suffix: &'static str) -> Self {
        self.field_suffix = Some(suffix);
        self
    }
}

impl Default for ComponentPrototyping {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalized form-value change derived from a component event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FormValueChange<T> {
    /// The component event did not change the form value.
    Unchanged,
    /// Replace the form value with the supplied value.
    Set(T),
    /// Clear an optional form value.
    Clear,
}

impl<T> FormValueChange<T> {
    pub const fn set(value: T) -> Self {
        Self::Set(value)
    }

    pub const fn clear() -> Self {
        Self::Clear
    }

    pub const fn unchanged() -> Self {
        Self::Unchanged
    }
}

/// Optional value-binding contract for component shapes.
///
/// Implement this alongside [`ComponentShape`] when generated prototyping code
/// should seed the component from the form value holder and subscribe to
/// component events. The form derive opts into this path by inheriting
/// [`ComponentShape::ValueBindingPolicy`] from the shape.
pub trait ComponentValueBinding<T>: ComponentShape
where
    Self::State: gpui::EventEmitter<Self::Event>,
{
    /// Event emitted by the component state.
    type Event: 'static;

    /// Seed component state from the current form value.
    fn seed_value_binding_state(
        _state: &mut Self::State,
        _value: Option<&T>,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<'_, Self::State>,
    ) {
    }

    /// Convert an emitted component event into a form value change.
    fn form_value_change(state: &Self::State, event: &Self::Event) -> FormValueChange<T>;
}

/// Assert that a shape supports value binding for a form value type.
///
/// Generated form code uses this helper to keep missing binding diagnostics
/// anchored to the field attribute while reporting the public
/// [`ComponentValueBinding`] contract.
pub fn assert_component_value_binding<Shape, Value>()
where
    Shape: ComponentValueBinding<Value>,
    ComponentStateOf<Shape>: gpui::EventEmitter<ComponentEventOf<Shape, Value>>,
{
}

/// Value-binding contract implemented by backing component state.
///
/// Component-owned shapes can implement [`ComponentValueBinding`] by delegating
/// to this state-level contract, keeping render element types separate from
/// their GPUI entity state.
pub trait ComponentStateValueBinding<T>: gpui::EventEmitter<Self::Event> {
    /// Event emitted by the backing component state.
    type Event: 'static;

    /// Seed component state from the current form value.
    fn seed_value_binding_state(
        _state: &mut Self,
        _value: Option<&T>,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<'_, Self>,
    ) where
        Self: Sized,
    {
    }

    /// Convert an emitted component event into a form value change.
    fn form_value_change(state: &Self, event: &Self::Event) -> FormValueChange<T>;
}

/// State type for a component shape.
pub type ComponentStateOf<Shape> = <Shape as ComponentShape>::State;

/// Event type for a value-bound component shape and value.
pub type ComponentEventOf<Shape, Value> = <Shape as ComponentValueBinding<Value>>::Event;

/// Seed component state from the current form value without spelling out the
/// associated-type projection at every generated call site.
pub fn seed_value_binding_state<Shape, Value>(
    state: &mut ComponentStateOf<Shape>,
    value: Option<&Value>,
    window: &mut gpui::Window,
    cx: &mut gpui::Context<'_, ComponentStateOf<Shape>>,
) where
    Shape: ComponentValueBinding<Value>,
    ComponentStateOf<Shape>: gpui::EventEmitter<ComponentEventOf<Shape, Value>>,
{
    Shape::seed_value_binding_state(state, value, window, cx);
}

/// Convert a component event into a form value change without repeating UFCS
/// projections in generated code.
pub fn form_value_change<Shape, Value>(
    state: &ComponentStateOf<Shape>,
    event: &ComponentEventOf<Shape, Value>,
) -> FormValueChange<Value>
where
    Shape: ComponentValueBinding<Value>,
    ComponentStateOf<Shape>: gpui::EventEmitter<ComponentEventOf<Shape, Value>>,
{
    Shape::form_value_change(state, event)
}
