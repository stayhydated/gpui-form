use gpui_form_derive::component_shape;

struct State;

component_shape! {
    struct InvalidComponentShape {
        type State = State;
        component = _;
    }
}

fn main() {}
