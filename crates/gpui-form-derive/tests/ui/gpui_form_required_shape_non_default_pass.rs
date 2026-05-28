use gpui_form_derive::GpuiForm;
use gpui_form_runtime::shape::{ComponentShape, NoComponentValueBinding, RequireValue};

#[derive(Clone, Debug)]
struct NonDefault(String);

struct State;

impl State {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

struct RequiredShape;

impl ComponentShape for RequiredShape {
    type State = State;
    type RequiredValuePolicy = RequireValue;
    type ValueBindingPolicy = NoComponentValueBinding;

    fn new(window: &mut gpui::Window, cx: &mut gpui::Context<'_, Self::State>) -> Self::State {
        State::new(window, cx)
    }
}

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form(RequiredShape)]
    value: NonDefault,

    #[gpui_form(default = String::new())]
    name: String,
}

fn main() {}
