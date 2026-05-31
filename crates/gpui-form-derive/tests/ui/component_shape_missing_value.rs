use gpui_form_derive::component_shape;

struct State;

component_shape! {
    struct MissingValueShape {
        type State = State;
    }
}

fn main() {}
