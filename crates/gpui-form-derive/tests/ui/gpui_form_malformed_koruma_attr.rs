use gpui_form_derive::GpuiForm;

struct InputState;

impl InputState {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

component_shape_gpui::component_shape! {
    struct InputShape {
        type State = InputState;
        value = String;
    }
}

impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for InputShape {
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}

#[derive(GpuiForm)]
#[gpui_form(koruma)]
struct Demo {
    #[gpui_form(component(crate::InputShape))]
    #[koruma]
    name: String,
}

fn main() {}
