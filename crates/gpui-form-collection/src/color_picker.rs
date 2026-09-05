use component_shape::ValueChange;
use component_shape_gpui::{GpuiComponentValueBinding, component_shape};
use gpui_kit::component::color_picker::{ColorPickerEvent, ColorPickerState};
use gpui_kit::{Context, Hsla, Window};

fn color_picker_value_change(event: &ColorPickerEvent) -> ValueChange<Hsla> {
    match event {
        ColorPickerEvent::Change(Some(color)) => ValueChange::Set(*color),
        ColorPickerEvent::Change(None) => ValueChange::Clear,
    }
}

component_shape! {
    /// Form component for a `gpui_kit::component::color_picker::ColorPicker` backed by `ColorPickerState`.
    pub struct ColorPicker {
        state = ColorPickerState;
        component = gpui_kit::component::color_picker::ColorPicker;
        value = Hsla;
        field_suffix = "color_picker";
        value_binding;

        impl GpuiComponentValueBinding<Hsla> for ColorPicker {
            type Event = ColorPickerEvent;

            fn seed_value_binding_state(
                state: &mut Self::State,
                value: Option<&Hsla>,
                window: &mut Window,
                cx: &mut Context<'_, Self::State>,
            ) {
                if let Some(value) = value {
                    state.set_value(*value, window, cx);
                }
            }

            fn value_change(_state: &Self::State, event: &Self::Event) -> ValueChange<Hsla> {
                color_picker_value_change(event)
            }
        }
    }
}

impl_form_component_shape!(ColorPicker, gpui_form_runtime::shape::RequiredValueStorage);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_picker_events_set_and_clear_form_values() {
        let color = Hsla::default();
        assert_eq!(
            color_picker_value_change(&ColorPickerEvent::Change(Some(color))),
            ValueChange::Set(color)
        );
        assert_eq!(
            color_picker_value_change(&ColorPickerEvent::Change(None)),
            ValueChange::Clear
        );
    }
}
