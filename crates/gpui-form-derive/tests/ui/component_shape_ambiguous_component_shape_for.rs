use gpui_form_derive::component_shape;

struct State;

trait ComponentShapeFor<T> {}

component_shape! {
    struct AmbiguousShape {
        type State = State;

        impl ComponentShapeFor<String> for AmbiguousShape {}
    }
}

fn main() {}
