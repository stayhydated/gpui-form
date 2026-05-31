use gpui_form_derive::component_shape;

struct MultiValueState;

impl MultiValueState {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

component_shape! {
    struct MultiValueShape {
        type State = MultiValueState;
        values(String, u32);
    }
}

fn assert_shape_for_string<T: gpui_form_runtime::shape::ComponentShapeFor<String>>() {}

fn assert_shape_for_u32<T: gpui_form_runtime::shape::ComponentShapeFor<u32>>() {}

fn main() {
    assert_shape_for_string::<MultiValueShape>();
    assert_shape_for_u32::<MultiValueShape>();
}
