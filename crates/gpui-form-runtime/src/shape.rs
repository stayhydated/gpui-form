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

    /// Build the component state.
    fn new(window: &mut gpui::Window, cx: &mut gpui::Context<'_, Self::State>) -> Self::State;

    /// Optional path to the UI component type (e.g. `"TagsInput"`).
    ///
    /// When set here via `gpui_form_derive::component_shape!` or
    /// `#[gpui_form_shape(component = ...)]`, the prototyping code generator can
    /// emit `Component::new(&entity)` without requiring component UI metadata to
    /// be repeated on every field annotation.
    ///
    /// A `.component(...)` override on the field shape expression always takes
    /// precedence.
    const COMPONENT_PATH: Option<&'static str> = None;

    /// Whether generated prototyping code should wire this component shape
    /// through [`ComponentValueBinding`] by default.
    ///
    /// Field-level `Shape.value_binding()` still opts in explicitly. This
    /// shape-level flag is useful when the component's derive or reusable
    /// shape owns the metadata and each field should inherit it.
    const VALUE_BINDING: bool = false;

    /// Metadata used by prototyping generators.
    ///
    /// This is intentionally separate from runtime construction so reusable
    /// component crates can describe generated-code preferences once, and
    /// downstream fields can inherit those preferences.
    const PROTOTYPING: ComponentPrototyping = ComponentPrototyping::new();
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
/// component events. The form derive opts into this path either with
/// `Shape.value_binding()` or by inheriting [`ComponentShape::VALUE_BINDING`]
/// from the shape.
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
