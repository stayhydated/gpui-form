use std::str::FromStr;

use component_shape::ValueChange;
use component_shape_gpui::{GpuiComponentValueBinding, component_shape};
use gpui::{App, Context, Entity, IntoElement, RenderOnce, Window};
use gpui_component::input::{InputEvent, OtpInput as GpuiOtpInput, OtpState};

#[derive(IntoElement)]
pub struct OtpInputField {
    state: Entity<OtpState>,
}

impl OtpInputField {
    pub fn new(state: &Entity<OtpState>) -> Self {
        Self {
            state: state.clone(),
        }
    }
}

impl RenderOnce for OtpInputField {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        GpuiOtpInput::new(&self.state)
    }
}

component_shape! {
    /// Form component for a `gpui_component::input::OtpInput` backed by `OtpState`.
    pub struct OtpInput<T = String>
    where
        T: FromStr + ToString + 'static,
    {
        type State = OtpState;
        new = |window, cx| OtpState::new(6, window, cx);
        component = gpui_form_collection::otp_input::OtpInputField;
        value = T;
        field_suffix = "otp_input";
        value_binding;

        impl<T> GpuiComponentValueBinding<T> for OtpInput<T>
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
                let value = value.map(ToString::to_string).unwrap_or_default();
                state.set_value(value, window, cx);
            }

            fn value_change(state: &Self::State, event: &Self::Event) -> ValueChange<T> {
                match event {
                    InputEvent::Change => {
                        let value = state.value();
                        if value.is_empty() {
                            ValueChange::Clear
                        } else {
                            value
                                .as_ref()
                                .parse::<T>()
                                .map_or(ValueChange::Unchanged, ValueChange::Set)
                        }
                    },
                    _ => ValueChange::Unchanged,
                }
            }
        }
    }
}

impl_form_component_shape!(
    impl<T> OtpInput<T>
    where [
        T: FromStr + ToString + 'static
    ];
    gpui_form_runtime::shape::DirectValueStorage
);
