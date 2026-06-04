use component_shape::ValueChange;
use component_shape_gpui::{GpuiComponentValueBinding, component_shape};
use gpui::{Context, Window};
use gpui_component::slider::{SliderEvent, SliderState, SliderValue};

component_shape! {
    /// Form component for a `gpui_component::slider::Slider` backed by `SliderState`.
    pub struct Slider {
        state = SliderState;
        new = |_window, _cx| SliderState::new();
        component = gpui_component::slider::Slider;
        values(SliderValue, f32);
        field_suffix = "slider";
        value_binding;

        impl GpuiComponentValueBinding<SliderValue> for Slider {
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

            fn value_change(
                _state: &Self::State,
                event: &Self::Event,
            ) -> ValueChange<SliderValue> {
                match event {
                    SliderEvent::Change(value) | SliderEvent::Release(value) => {
                        ValueChange::Set(*value)
                    },
                }
            }
        }

        impl GpuiComponentValueBinding<f32> for Slider {
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

            fn value_change(_state: &Self::State, event: &Self::Event) -> ValueChange<f32> {
                match event {
                    SliderEvent::Change(value) | SliderEvent::Release(value) => match value {
                        SliderValue::Single(value) => ValueChange::Set(*value),
                        SliderValue::Range(start, _) => ValueChange::Set(*start),
                    },
                }
            }
        }
    }
}

impl_form_component_shape!(Slider, gpui_form_runtime::shape::DirectValueStorage);
