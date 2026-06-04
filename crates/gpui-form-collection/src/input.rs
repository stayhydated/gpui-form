use std::str::FromStr;

use component_shape::ValueChange;
use component_shape_gpui::{GpuiComponentValueBinding, component_shape};
use gpui::{Context, Window};
use gpui_component::input::{InputEvent, InputState};

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
