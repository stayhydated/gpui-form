use std::str::FromStr;

use gpui::{Context, Window};
use gpui_component::input::{InputEvent, InputState};
use gpui_form_component::custom::{CustomComponentValueAdapter, CustomComponentValueChange};

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

    fn set_state_value(
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

    fn value_change(state: &Self::State, event: &Self::Event) -> CustomComponentValueChange<T> {
        match event {
            InputEvent::Change => {
                let value = state.value();
                if value.is_empty() {
                    CustomComponentValueChange::Clear
                } else {
                    value
                        .parse::<T>()
                        .map(CustomComponentValueChange::Set)
                        .unwrap_or(CustomComponentValueChange::Unchanged)
                }
            },
            _ => CustomComponentValueChange::Unchanged,
        }
    }
}
