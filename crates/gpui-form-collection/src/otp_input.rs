use std::str::FromStr;

use gpui::{App, Context, Entity, IntoElement, RenderOnce, Window};
use gpui_component::input::{InputEvent, OtpInput as GpuiOtpInput, OtpState};
use gpui_form_component::shape::{ComponentValueBinding, FormValueChange};

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

gpui_form_derive::component_shape! {
    /// Form component for a `gpui_component::input::OtpInput` backed by `OtpState`.
    pub struct OtpInput<T = String>
    where
        T: FromStr + ToString + 'static,
    {
        type State = OtpState;
        new = |window, cx| OtpState::new(6, window, cx);
        component = gpui_form_collection::otp_input::OtpInputField;
        value_binding;
        field_suffix = "otp_input";
    }
}

impl<T> ComponentValueBinding<T> for OtpInput<T>
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

    fn form_value_change(state: &Self::State, event: &Self::Event) -> FormValueChange<T> {
        match event {
            InputEvent::Change => {
                let value = state.value();
                if value.is_empty() {
                    FormValueChange::Clear
                } else {
                    value
                        .as_ref()
                        .parse::<T>()
                        .map_or(FormValueChange::Unchanged, FormValueChange::Set)
                }
            }
            _ => FormValueChange::Unchanged,
        }
    }
}
