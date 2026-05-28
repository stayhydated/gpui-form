use gpui_form_derive::component_shape;

struct State;

component_shape! {
    struct DuplicateMetadataShape {
        type State = State;
        field_suffix = "input";
        field_suffix = "field";
    }
}

fn main() {}
