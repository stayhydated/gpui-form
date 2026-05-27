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

#[derive(ComponentShape)]
#[gpui_form_shape(new = DirectTagsState::with_label(window, cx, "direct"))]
struct DirectTagsState;

#[derive(ComponentShape)]
#[gpui_form_shape(new = BoolBindingTagsState::new, value_binding = true)]
struct BoolBindingTagsState;

impl TagsState {
    fn with_label(
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<'_, Self>,
        _label: &'static str,
    ) -> Self {
        Self
    }
}

impl DirectTagsState {
    fn with_label(
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<'_, Self>,
        _label: &'static str,
    ) -> Self {
        Self
    }
}

impl BoolBindingTagsState {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

fn main() {}
