use gpui_form_derive::component_shape;

struct State;

component_shape! {
    struct UnknownOptionShape {
        type State = State;
        unknown = true;
    }
}

fn main() {}
