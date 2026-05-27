use gpui_form_derive::{ComponentShape, component_shape};

struct Widget;

struct ExternalState;

impl ExternalState {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

#[derive(ComponentShape)]
#[gpui_form_shape(value_binding = true)]
struct DerivedState;

impl DerivedState {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

component_shape! {
    pub struct ExternalShape {
        type State = ExternalState;
        component = Widget;
        value_binding = false;
    }
}

fn main() {}
