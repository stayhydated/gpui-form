use gpui_form_derive::GpuiForm;
use gpui_form_runtime::shape::{ComponentShape, InheritedComponentValueBinding, RequireValue};

struct State;

impl<Event: 'static> gpui::EventEmitter<Event> for State {}

impl State {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

struct InheritedBindingShape;

impl ComponentShape for InheritedBindingShape {
    type State = State;
    type RequiredValuePolicy = RequireValue;
    type ValueBindingPolicy = InheritedComponentValueBinding;

    fn new(window: &mut gpui::Window, cx: &mut gpui::Context<'_, Self::State>) -> Self::State {
        State::new(window, cx)
    }
}

#[derive(GpuiForm)]
struct InheritedDemo {
    #[gpui_form(InheritedBindingShape)]
    name: String,
}

fn main() {}
