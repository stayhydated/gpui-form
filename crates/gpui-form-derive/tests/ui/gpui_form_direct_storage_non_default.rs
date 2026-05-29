use gpui_form_derive::GpuiForm;
use gpui_form_runtime::shape::{
    DirectValueStorage, ComponentShape, ComponentShapeFor, NoComponentValueBinding,
};

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
    type ValueStoragePolicy = DirectValueStorage;
    type ValueBindingPolicy = NoComponentValueBinding;

    fn new(window: &mut gpui::Window, cx: &mut gpui::Context<'_, Self::State>) -> Self::State {
        State::new(window, cx)
    }
}

impl<T> ComponentShapeFor<T> for DirectStorageShape {}

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form(shape = DirectStorageShape)]
    value: NonDefault,
}

fn main() {}
