use gpui_form_derive::ComponentShape;

struct TagsInput;

impl TagsInput {
    fn new(_entity: &gpui::Entity<TagsState>) -> gpui::Div {
        gpui::div()
    }
}

impl DirectTagsInput {
    fn new(_entity: &gpui::Entity<DirectTagsState>) -> gpui::Div {
        gpui::div()
    }
}

impl BoolBindingTagsInput {
    fn new(_entity: &gpui::Entity<BoolBindingTagsState>) -> gpui::Div {
        gpui::div()
    }
}

#[derive(ComponentShape)]
#[gpui_form_shape(
    state = TagsState,
    new = |window, cx| TagsState::with_label(window, cx, "tags"),
    component = TagsInput,
    field_suffix = "tags"
)]
struct TagsInputShape;

#[derive(ComponentShape)]
#[gpui_form_shape(
    state = DirectTagsState,
    new = DirectTagsState::with_label(window, cx, "direct")
)]
struct DirectTagsInput;

#[derive(ComponentShape)]
#[gpui_form_shape(state = BoolBindingTagsState, new = BoolBindingTagsState::new)]
struct BoolBindingTagsInput;

impl TagsState {
    fn with_label(
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<'_, Self>,
        _label: &'static str,
    ) -> Self {
        Self
    }
}

struct TagsState;

impl DirectTagsState {
    fn with_label(
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<'_, Self>,
        _label: &'static str,
    ) -> Self {
        Self
    }
}

struct DirectTagsState;

impl BoolBindingTagsState {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

struct BoolBindingTagsState;

fn main() {}
