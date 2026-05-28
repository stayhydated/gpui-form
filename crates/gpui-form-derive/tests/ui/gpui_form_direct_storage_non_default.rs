use gpui_form_derive::GpuiForm;
use gpui_form_runtime::shape::{AllowMissingValue, ComponentShape, NoComponentValueBinding};

#[derive(Clone, Debug)]
struct NonDefault(String);

struct State;

impl State {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

struct DirectStorageShape;

impl ComponentShape for DirectStorageShape {
    type State = State;
    type RequiredValuePolicy = AllowMissingValue;
    type ValueBindingPolicy = NoComponentValueBinding;

    fn new(window: &mut gpui::Window, cx: &mut gpui::Context<'_, Self::State>) -> Self::State {
        State::new(window, cx)
    }
}

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form(DirectStorageShape)]
    value: NonDefault,
}

fn main() {}
