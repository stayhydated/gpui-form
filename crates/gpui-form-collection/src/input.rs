use std::str::FromStr;

use component_shape::ValueChange;
use component_shape_gpui::{GpuiComponentValueBinding, component_shape};
use gpui::{Context, Window};
use gpui_component::input::{InputEvent, InputState};

/// Configuration for [`ParsedInput`].
///
/// Implement this trait in an application crate when a text input should store
/// a typed value but needs custom parsing or display formatting beyond
/// `FromStr` and `ToString`.
pub trait ParsedInputConfig<T>: 'static {
    type Error;

    /// Optional placeholder applied when the input state is created.
    const PLACEHOLDER: Option<&'static str> = None;

    /// Whether the input widget should reject text that [`ParsedInputConfig::parse`]
    /// cannot parse.
    ///
    /// Leave this as `false` when invalid user text should remain visible while
    /// generated value binding reports `ValueChange::Unchanged`.
    const VALIDATE_INPUT: bool = false;

    /// Parse text into the stored form value.
    fn parse(value: &str) -> Result<T, Self::Error>;

    /// Format a stored form value for display in the input.
    fn format(value: &T) -> String;

    /// Returns true when a change should clear optional storage.
    fn empty_as_clear(value: &str) -> bool {
        value.trim().is_empty()
    }

    /// Returns true when the input text should be accepted by widget-level
    /// validation.
    fn is_valid_input(value: &str) -> bool {
        Self::empty_as_clear(value) || Self::parse(value).is_ok()
    }
}

component_shape! {
    /// Form component for a parsed `gpui_component::input::Input`.
    ///
    /// `ParsedInput<T, C>` uses [`ParsedInputConfig`] for display formatting,
    /// parsing, placeholder text, empty-as-clear behavior, and optional
    /// widget-level validation.
    pub struct ParsedInput<T, C>
    where
        T: 'static,
        C: ParsedInputConfig<T>,
    {
        state = InputState;
        new = |window, cx| ParsedInput::<T, C>::new_state(window, cx);
        component = gpui_component::input::Input;
        value = T;
        field_suffix = "input";
        value_binding;

        impl<T, C> GpuiComponentValueBinding<T> for ParsedInput<T, C>
        where
            T: 'static,
            C: ParsedInputConfig<T>,
        {
            type Event = InputEvent;

            fn seed_value_binding_state(
                state: &mut Self::State,
                value: Option<&T>,
                window: &mut Window,
                cx: &mut Context<'_, Self::State>,
            ) {
                state.set_value(value.map(C::format).unwrap_or_default(), window, cx);
            }

            fn value_change(state: &Self::State, event: &Self::Event) -> ValueChange<T> {
                match event {
                    InputEvent::Change => Self::value_change_from_text(state.value().as_ref()),
                    _ => ValueChange::Unchanged,
                }
            }
        }
    }

}

impl<T, C> ParsedInput<T, C>
where
    T: 'static,
    C: ParsedInputConfig<T>,
{
    pub fn new_state(window: &mut Window, cx: &mut Context<'_, InputState>) -> InputState {
        let state = InputState::new(window, cx);
        let state = match C::PLACEHOLDER {
            Some(placeholder) => state.placeholder(placeholder),
            None => state,
        };

        if C::VALIDATE_INPUT {
            state.validate(|value, _| C::is_valid_input(value))
        } else {
            state
        }
    }

    pub fn value_change_from_text(value: &str) -> ValueChange<T> {
        if C::empty_as_clear(value) {
            ValueChange::Clear
        } else {
            C::parse(value)
                .map(ValueChange::Set)
                .unwrap_or(ValueChange::Unchanged)
        }
    }
}

impl_form_component_shape!(
    impl<T, C> ParsedInput<T, C>
    where [
        T: 'static,
        C: ParsedInputConfig<T>
    ];
    gpui_form_runtime::shape::DirectValueStorage
);

component_shape! {
    /// Form component for a `gpui_component::input::Input` backed by `InputState`.
    ///
    /// Use `Input::<_>` in `#[gpui_form(component(...))]` so the derive
    /// resolves `_` to the field's form-side type.
    pub struct Input<T = String>
    where
        T: FromStr + ToString + 'static,
    {
        state = InputState;
        new = |window, cx| InputState::new(window, cx)
            .validate(|value, _| value.parse::<T>().is_ok());
        component = gpui_component::input::Input;
        value = T;
        field_suffix = "input";
        value_binding;

        impl<T> GpuiComponentValueBinding<T> for Input<T>
        where
            T: FromStr + ToString + 'static,
        {
            type Event = InputEvent;

            fn seed_value_binding_state(
                state: &mut Self::State,
                value: Option<&T>,
                window: &mut Window,
                cx: &mut Context<'_, Self::State>,
            ) {
                state.set_value(
                    value.map(ToString::to_string).unwrap_or_default(),
                    window,
                    cx,
                );
            }

            fn value_change(state: &Self::State, event: &Self::Event) -> ValueChange<T> {
                match event {
                    InputEvent::Change => {
                        let value = state.value();
                        if value.is_empty() {
                            ValueChange::Clear
                        } else {
                            value
                                .parse::<T>()
                                .map(ValueChange::Set)
                                .unwrap_or(ValueChange::Unchanged)
                        }
                    },
                    _ => ValueChange::Unchanged,
                }
            }
        }
    }
}

impl_form_component_shape!(
    impl<T> Input<T>
    where [
        T: FromStr + ToString + 'static
    ];
    gpui_form_runtime::shape::DirectValueStorage
);

#[cfg(test)]
mod tests {
    use super::{ParsedInput, ParsedInputConfig};
    use component_shape::ValueChange;
    use component_shape_gpui::{GpuiComponentShape, GpuiComponentValueBinding};
    use gpui_component::input::{InputEvent, InputState};
    use gpui_form_runtime::shape::{
        DirectValueStorage, GpuiComponentShapeFor, GpuiFormComponentShapePolicy,
    };
    use std::num::ParseIntError;

    #[derive(Debug, Eq, PartialEq)]
    struct AccountCode(u32);

    struct AccountCodeInputConfig;

    impl ParsedInputConfig<AccountCode> for AccountCodeInputConfig {
        type Error = ParseIntError;

        const PLACEHOLDER: Option<&'static str> = Some("Account code");

        fn parse(value: &str) -> Result<AccountCode, Self::Error> {
            value.trim().parse().map(AccountCode)
        }

        fn format(value: &AccountCode) -> String {
            value.0.to_string()
        }
    }

    struct StrictNumberInputConfig;

    impl ParsedInputConfig<AccountCode> for StrictNumberInputConfig {
        type Error = ParseIntError;

        const VALIDATE_INPUT: bool = true;

        fn parse(value: &str) -> Result<AccountCode, Self::Error> {
            value.parse().map(AccountCode)
        }

        fn format(value: &AccountCode) -> String {
            value.0.to_string()
        }
    }

    type AccountCodeInput = ParsedInput<AccountCode, AccountCodeInputConfig>;

    fn assert_shape_contract<Shape, Value>()
    where
        Shape: component_shape_gpui::DeclaredGpuiComponentShape
            + GpuiComponentShape<State = InputState>
            + GpuiComponentShapeFor<Value>
            + GpuiFormComponentShapePolicy<ValueStoragePolicy = DirectValueStorage>
            + GpuiComponentValueBinding<Value, Event = InputEvent>,
    {
    }

    #[test]
    fn parsed_input_publishes_shape_contracts() {
        assert_shape_contract::<AccountCodeInput, AccountCode>();
    }

    #[test]
    fn parsed_input_config_maps_format_parse_empty_and_validation() {
        assert_eq!(
            AccountCodeInputConfig::format(&AccountCode(42)),
            "42".to_string()
        );
        assert_eq!(
            AccountCodeInputConfig::parse(" 42 ").unwrap(),
            AccountCode(42)
        );
        assert!(AccountCodeInputConfig::empty_as_clear("   "));
        assert!(AccountCodeInputConfig::is_valid_input(" 42 "));
        assert!(AccountCodeInputConfig::is_valid_input("   "));
        assert!(!AccountCodeInputConfig::is_valid_input("bad"));
    }

    #[test]
    fn parsed_input_config_can_request_widget_validation() {
        assert!(StrictNumberInputConfig::VALIDATE_INPUT);
        assert!(StrictNumberInputConfig::is_valid_input("42"));
        assert!(StrictNumberInputConfig::is_valid_input(""));
        assert!(!StrictNumberInputConfig::is_valid_input(" 42 "));
    }

    #[test]
    fn parsed_input_value_change_sets_clears_and_keeps_invalid_unchanged() {
        assert_eq!(
            AccountCodeInput::value_change_from_text(" 42 "),
            ValueChange::Set(AccountCode(42))
        );
        assert_eq!(
            AccountCodeInput::value_change_from_text("   "),
            ValueChange::Clear
        );
        assert_eq!(
            AccountCodeInput::value_change_from_text("bad"),
            ValueChange::Unchanged
        );
    }
}
