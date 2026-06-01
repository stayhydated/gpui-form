use component_shape::ValueChange;
use component_shape_gpui::{GpuiComponentValueBinding, component_shape};
use gpui::{Context, Hsla, Window};
use gpui_component::color_picker::{ColorPickerEvent, ColorPickerState};

component_shape! {
    /// Form component for a `gpui_component::color_picker::ColorPicker` backed by `ColorPickerState`.
    pub struct ColorPicker {
        type State = ColorPickerState;
        component = gpui_component::color_picker::ColorPicker;
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
                match event {
                    ColorPickerEvent::Change(Some(color)) => ValueChange::Set(*color),
                    ColorPickerEvent::Change(None) => ValueChange::Clear,
                }
            }
        }
    }
}

impl_form_component_shape!(ColorPicker, gpui_form_runtime::shape::RequiredValueStorage);
