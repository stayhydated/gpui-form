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
    struct RequiredShape {
        state = State;
        value = NonDefault;
    }
}

impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for RequiredShape {
    type ValueStoragePolicy = gpui_form_runtime::shape::RequiredValueStorage;
}

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form(component(RequiredShape))]
    value: NonDefault,

    #[gpui_form(hidden(default = String::new()))]
    name: String,
}

fn main() {}
