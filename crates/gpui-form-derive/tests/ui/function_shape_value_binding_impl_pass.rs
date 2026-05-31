use gpui_form_derive::component_shape;
use gpui_form_runtime::shape::FormValueChange;

struct BindingEvent;
struct BindingState;

impl gpui::EventEmitter<BindingEvent> for BindingState {}

impl BindingState {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

component_shape! {
    struct BoundShape {
        type State = BindingState;
        value = String;
        value_binding;

        impl gpui_form_runtime::shape::ComponentValueBinding<String> for BoundShape {
            type Event = BindingEvent;

            fn form_value_change(
                _state: &Self::State,
                _event: &Self::Event,
            ) -> FormValueChange<String> {
                FormValueChange::unchanged()
            }
        }
    }
}

fn main() {
    gpui_form_runtime::shape::assert_component_value_binding::<BoundShape, String>();
}
