use gpui_form_derive::GpuiForm;
use gpui_form_runtime::shape::{ComponentShape, RequireValue};

struct State;

impl<Event: 'static> gpui::EventEmitter<Event> for State {}

impl State {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

struct MissingBindingShape;

impl ComponentShape for MissingBindingShape {
    type State = State;
    type RequiredValuePolicy = RequireValue;

    fn new(window: &mut gpui::Window, cx: &mut gpui::Context<'_, Self::State>) -> Self::State {
        State::new(window, cx)
    }
}

struct InheritedBindingShape;

impl ComponentShape for InheritedBindingShape {
    type State = State;
    type RequiredValuePolicy = RequireValue;

    const VALUE_BINDING: bool = true;

    fn new(window: &mut gpui::Window, cx: &mut gpui::Context<'_, Self::State>) -> Self::State {
        State::new(window, cx)
    }
}

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form(MissingBindingShape.value_binding())]
    name: String,
}

#[derive(GpuiForm)]
struct InheritedDemo {
    #[gpui_form(InheritedBindingShape)]
    name: String,
}

fn main() {}
