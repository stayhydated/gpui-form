use gpui_form_derive::GpuiForm;

#[derive(Clone, Debug)]
struct NonDefault(String);

struct State;

impl State {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

component_shape_gpui::component_shape! {
    struct DirectStorageShape {
        type State = State;
        value = NonDefault;
    }
}

impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for DirectStorageShape {
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form(component(DirectStorageShape))]
    value: NonDefault,
}

fn main() {}
