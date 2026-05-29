use gpui_form_derive::component_shape;

struct State;

component_shape! {
    struct UnknownOptionShape {
        type State = State;
        unknown = true;
    }
}

component_shape! {
    struct OldRequiresValueShape {
        type State = State;
        requires_value = false;
    }
}

component_shape! {
    struct InvalidValueStorageShape {
        type State = State;
        value_storage = synthesize;
    }
}

fn main() {}
