use component_shape::ValueChange;
use component_shape_gpui::{GpuiComponentValueBinding, component_shape};
use gpui_kit::component::slider::{SliderEvent, SliderState, SliderValue};
use gpui_kit::{Context, Window};

fn slider_value_change(event: &SliderEvent) -> ValueChange<SliderValue> {
    match event {
        SliderEvent::Change(value) | SliderEvent::Release(value) => ValueChange::Set(*value),
    }
}

fn scalar_slider_value_change(event: &SliderEvent) -> ValueChange<f32> {
    match event {
        SliderEvent::Change(value) | SliderEvent::Release(value) => match value {
            SliderValue::Single(value) => ValueChange::Set(*value),
            SliderValue::Range(start, _) => ValueChange::Set(*start),
        },
    }
}

component_shape! {
    /// Form component for a `gpui_kit::component::slider::Slider` backed by `SliderState`.
    pub struct Slider {
        state = SliderState;
        new = |_window, _cx| SliderState::new();
        component = gpui_kit::component::slider::Slider;
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
                slider_value_change(event)
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
                scalar_slider_value_change(event)
            }
        }
    }
}

impl_form_component_shape!(Slider, gpui_form_runtime::shape::DirectValueStorage);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_events_preserve_ranges_and_extract_scalar_values() {
        let range = SliderValue::Range(2.0, 8.0);
        assert_eq!(
            slider_value_change(&SliderEvent::Change(range)),
            ValueChange::Set(range)
        );
        assert_eq!(
            scalar_slider_value_change(&SliderEvent::Change(range)),
            ValueChange::Set(2.0)
        );
        assert_eq!(
            scalar_slider_value_change(&SliderEvent::Release(SliderValue::Single(4.0))),
            ValueChange::Set(4.0)
        );
    }
}
