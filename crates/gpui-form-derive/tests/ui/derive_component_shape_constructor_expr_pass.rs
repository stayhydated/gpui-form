use gpui_form_derive::ComponentShape;

struct TagsInput;

#[derive(ComponentShape)]
#[gpui_form_shape(
    new = |window, cx| Self::with_label(window, cx, "tags"),
    component = TagsInput,
    value_binding,
    field_suffix = "tags"
)]
struct TagsState;

impl TagsState {
    fn with_label(
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<'_, Self>,
        _label: &'static str,
    ) -> Self {
        Self
    }
}

fn main() {}
