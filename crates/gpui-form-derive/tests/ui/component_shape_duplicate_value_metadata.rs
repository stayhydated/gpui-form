use gpui_form_derive::component_shape;

struct State;

component_shape! {
    struct DuplicateValueShape {
        type State = State;
        value = String;
        values(u32, String);
    }
}

fn main() {}
