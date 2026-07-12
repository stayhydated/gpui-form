#![deny(non_snake_case)]

use gpui_form_derive::GpuiForm;

struct State;

impl State {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

component_shape_gpui::component_shape! {
    struct TypeShape {
        state = State;
        value = String;
    }
}

impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for TypeShape {
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}

#[derive(GpuiForm)]
#[gpui_form(no_inventory)]
struct Demo {
    #[gpui_form(component(TypeShape))]
    type_: String,
}

fn main() {}
