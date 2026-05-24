use std::str::FromStr;

use gpui::{Context, Window};
use gpui_component::input::{InputEvent, InputState};
use gpui_form_component::custom::{CustomComponentValueAdapter, ValueBindingChange};

gpui_form_derive::custom_component! {
    /// Form component for a `gpui_component::input::Input` backed by `InputState`.
    ///
    /// Use `Input::<_>` in `#[gpui_form(component = ...)]` so the derive
    /// resolves `_` to the field's form-side type.
    pub struct Input<T = String>
    where
        T: FromStr + ToString + 'static,
    {
        type State = InputState;
        new = |window, cx| InputState::new(window, cx)
            .validate(|value, _| value.parse::<T>().is_ok());
        component = gpui_component::input::Input;
        value_binding;
        field_suffix = "input";
    }
}

impl<T> CustomComponentValueAdapter<T> for Input<T>
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

    fn value_binding_change(state: &Self::State, event: &Self::Event) -> ValueBindingChange<T> {
        match event {
            InputEvent::Change => {
                let value = state.value();
                if value.is_empty() {
                    ValueBindingChange::Clear
                } else {
                    value
                        .parse::<T>()
                        .map(ValueBindingChange::Set)
                        .unwrap_or(ValueBindingChange::Unchanged)
                }
            },
            _ => ValueBindingChange::Unchanged,
        }
    }
}
