use gpui_form_derive::GpuiForm;

struct State;

impl State {
    fn new(_window: &mut gpui_kit::Window, _cx: &mut gpui_kit::Context<'_, Self>) -> Self {
        Self
    }
}

component_shape_gpui::component_shape! {
    struct StringShape {
        state = State;
        value = String;
    }
}

impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for StringShape {
    type ValueStoragePolicy = gpui_form_runtime::shape::RequiredValueStorage;
}

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form(component(StringShape))]
    value: u64,
}

fn main() {}
