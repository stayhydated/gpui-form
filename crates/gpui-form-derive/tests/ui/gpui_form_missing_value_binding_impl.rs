use gpui_form_derive::GpuiForm;

struct State;

impl<Event: 'static> gpui::EventEmitter<Event> for State {}

impl State {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

gpui_form_derive::component_shape! {
    struct InheritedBindingShape {
        type State = State;
        value_binding;

        impl<T> gpui_form_runtime::shape::ComponentShapeFor<T> for InheritedBindingShape {}
    }
}

#[derive(GpuiForm)]
struct InheritedDemo {
    #[gpui_form(component(InheritedBindingShape))]
    name: String,
}

fn main() {}
