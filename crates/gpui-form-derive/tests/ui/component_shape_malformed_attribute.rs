use gpui_form_derive::ComponentShape;

struct State;

#[derive(ComponentShape)]
#[gpui_form_shape(state)]
struct MalformedShape;

fn main() {}
