use gpui_form_derive::GpuiForm;
use gpui_form_runtime::shape::{
    ComponentShape, ComponentShapeFor, NoComponentValueBinding, RequiredValueStorage,
};

struct State;

impl State {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

struct ManualShape;

impl ComponentShape for ManualShape {
    type State = State;
    type ValueStoragePolicy = RequiredValueStorage;
    type ValueBindingPolicy = NoComponentValueBinding;

    fn new(window: &mut gpui::Window, cx: &mut gpui::Context<'_, Self::State>) -> Self::State {
        State::new(window, cx)
    }
}

impl<T> ComponentShapeFor<T> for ManualShape {}

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form(component(shape = ManualShape))]
    value: String,
}

fn main() {}
