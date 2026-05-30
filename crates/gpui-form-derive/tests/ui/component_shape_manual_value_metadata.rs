use gpui_form_derive::component_shape;

struct State;

component_shape! {
    struct ManualValueShape {
        type State = State;
        value = String;

        impl gpui_form_runtime::shape::ComponentShapeFor<String> for ManualValueShape {}
    }
}

fn main() {}
