use gpui::{Context, Hsla, Window};
use gpui_component::color_picker::{ColorPickerEvent, ColorPickerState};
use gpui_form_runtime::shape::{ComponentValueBinding, FormValueChange};

gpui_form_derive::component_shape! {
    /// Form component for a `gpui_component::color_picker::ColorPicker` backed by `ColorPickerState`.
    pub struct ColorPicker {
        type State = ColorPickerState;
        component = gpui_component::color_picker::ColorPicker;
        field_suffix = "color_picker";
        value_binding;

        impl ComponentValueBinding<Hsla> for ColorPicker {
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

            fn form_value_change(_state: &Self::State, event: &Self::Event) -> FormValueChange<Hsla> {
                match event {
                    ColorPickerEvent::Change(Some(color)) => FormValueChange::Set(*color),
                    ColorPickerEvent::Change(None) => FormValueChange::Clear,
                }
            }
        }
    }
}
