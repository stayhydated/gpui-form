use gpui::{Context, Window};
use gpui_component::slider::{SliderEvent, SliderState, SliderValue};
use gpui_form_runtime::shape::{ComponentValueBinding, FormValueChange};

gpui_form_derive::component_shape! {
    /// Form component for a `gpui_component::slider::Slider` backed by `SliderState`.
    pub struct Slider {
        type State = SliderState;
        new = |_window, _cx| SliderState::new();
        component = gpui_component::slider::Slider;
        value_binding;
        field_suffix = "slider";
    }
}

impl ComponentValueBinding<SliderValue> for Slider {
    type Event = SliderEvent;

    fn seed_value_binding_state(
        state: &mut Self::State,
        value: Option<&SliderValue>,
        window: &mut Window,
        cx: &mut Context<'_, Self::State>,
    ) {
        match value {
            Some(value) => state.set_value(*value, window, cx),
            None => state.set_value(0.0f32, window, cx),
        }
    }

    fn form_value_change(
        _state: &Self::State,
        event: &Self::Event,
    ) -> FormValueChange<SliderValue> {
        match event {
            SliderEvent::Change(value) | SliderEvent::Release(value) => {
                FormValueChange::Set(*value)
            },
        }
    }
}

impl ComponentValueBinding<f32> for Slider {
    type Event = SliderEvent;

    fn seed_value_binding_state(
        state: &mut Self::State,
        value: Option<&f32>,
        window: &mut Window,
        cx: &mut Context<'_, Self::State>,
    ) {
        let value = value.copied().unwrap_or(0.0);
        state.set_value(SliderValue::from(value), window, cx);
    }

    fn form_value_change(_state: &Self::State, event: &Self::Event) -> FormValueChange<f32> {
        match event {
            SliderEvent::Change(value) | SliderEvent::Release(value) => match value {
                SliderValue::Single(value) => FormValueChange::Set(*value),
                SliderValue::Range(start, _) => FormValueChange::Set(*start),
            },
        }
    }
}
