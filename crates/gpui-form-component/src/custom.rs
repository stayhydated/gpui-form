//! Runtime contract for custom components used by `#[derive(GpuiForm)]`.
//!
//! Users define a zero-sized "shape" type that implements [`CustomComponentShape`].
//! The derive macro uses that shape to generate:
//! - `FormFields` entity state type
//! - `FormComponents` constructor function body
//!
//! Prefer using [`custom_component_shape!`] or `#[derive(gpui_form::CustomComponent)]`
//! to define shape types.

/// Shape contract for user-defined components.
///
/// Implementations provide the component state type and how to construct it.
pub trait CustomComponentShape {
    /// Backing gpui component state type.
    type State: 'static;

    /// Build the component state.
    fn new(window: &mut gpui::Window, cx: &mut gpui::Context<'_, Self::State>) -> Self::State;

    /// Optional path to the UI component type (e.g. `"TagsInput"`).
    ///
    /// When set here – via [`custom_component_shape!`] `component = …` or
    /// `#[gpui_form_custom(component = …)]` – the prototyping code generator
    /// can emit `Component::new(&entity)` without requiring `component = …`
    /// to be repeated on every field annotation.
    ///
    /// A `component = …` on the field attribute always takes precedence.
    const COMPONENT_PATH: Option<&'static str> = None;

    /// Whether generated prototyping code should wire this custom component
    /// through [`CustomComponentValueAdapter`] by default.
    ///
    /// Field-level `component(custom(..., value_binding))` still opts in
    /// explicitly. This shape-level flag is useful when the component's derive
    /// or reusable shape owns the metadata and each field should inherit it.
    const VALUE_BINDING: bool = false;
}

/// Value update emitted by a custom component adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CustomComponentValueChange<T> {
    /// The component event did not change the form value.
    Unchanged,
    /// Replace the form value with the supplied value.
    Set(T),
    /// Clear an optional form value.
    Clear,
}

impl<T> CustomComponentValueChange<T> {
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

/// Optional value-binding contract for user-defined custom components.
///
/// Implement this alongside [`CustomComponentShape`] when generated
/// prototyping code should seed the component from the form value holder and
/// subscribe to component events. The form derive opts into this path either
/// with `component(custom(..., value_binding))` or by inheriting
/// [`CustomComponentShape::VALUE_BINDING`] from the shape.
pub trait CustomComponentValueAdapter<T>: CustomComponentShape {
    /// Event emitted by the custom component state.
    type Event: 'static;

    /// Seed component state from the current form value.
    fn set_state_value(
        _state: &mut Self::State,
        _value: Option<&T>,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<'_, Self::State>,
    ) {
    }

    /// Convert a component event into a form value update.
    fn value_change(state: &Self::State, event: &Self::Event) -> CustomComponentValueChange<T>;
}

/// Define a custom component shape with minimal boilerplate.
///
/// # Example
///
/// ```ignore
/// gpui_form_component::custom_component_shape!(
///     pub EmailInputShape,
///     state = gpui_component::input::InputState,
///     new = gpui_component::input::InputState::new,
///     component = gpui_component::input::Input,
/// );
/// ```
#[macro_export]
macro_rules! custom_component_shape {
    // With explicit component path and value binding metadata
    ($vis:vis $shape:ident, state = $state:ty, new = $new:expr, component = $component:path, value_binding $(,)?) => {
        $vis struct $shape;

        impl $crate::custom::CustomComponentShape for $shape {
            type State = $state;

            fn new(
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::Context<'_, Self::State>,
            ) -> Self::State {
                ($new)(window, cx)
            }

            const COMPONENT_PATH: Option<&'static str> = Some(stringify!($component));
            const VALUE_BINDING: bool = true;
        }
    };
    // With explicit component path
    ($vis:vis $shape:ident, state = $state:ty, new = $new:expr, component = $component:path $(,)?) => {
        $vis struct $shape;

        impl $crate::custom::CustomComponentShape for $shape {
            type State = $state;

            fn new(
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::Context<'_, Self::State>,
            ) -> Self::State {
                ($new)(window, cx)
            }

            const COMPONENT_PATH: Option<&'static str> = Some(stringify!($component));
        }
    };
    // Without component path, with value binding metadata
    ($vis:vis $shape:ident, state = $state:ty, new = $new:expr, value_binding $(,)?) => {
        $vis struct $shape;

        impl $crate::custom::CustomComponentShape for $shape {
            type State = $state;

            fn new(
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::Context<'_, Self::State>,
            ) -> Self::State {
                ($new)(window, cx)
            }

            const VALUE_BINDING: bool = true;
        }
    };
    // Without component path (original form)
    ($vis:vis $shape:ident, state = $state:ty, new = $new:expr $(,)?) => {
        $vis struct $shape;

        impl $crate::custom::CustomComponentShape for $shape {
            type State = $state;

            fn new(
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::Context<'_, Self::State>,
            ) -> Self::State {
                ($new)(window, cx)
            }
        }
    };
}
