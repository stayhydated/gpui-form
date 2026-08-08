use gpui::{App, AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, Render, Window};
use gpui::{InteractiveElement, ParentElement as _, Styled};
use gpui_component::form::v_form;
use gpui_component::separator::Separator;
use gpui_component::v_flex;
use some_lib::structs::empty::*;
const CONTEXT: &str = "EmptyForm";
#[gpui_storybook::story_init]
pub fn init(_cx: &mut App) {}
#[derive(gpui_storybook::StoryControls)]
#[gpui_storybook::story]
pub struct EmptyForm {
    _fields: EmptyFormFields,
    focus_handle: FocusHandle,
}
impl Focusable for EmptyForm {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
impl gpui_storybook::Story for EmptyForm {
    fn title(cx: &gpui::App) -> String {
        gpui_es_fluent::localize_label::<Empty>(cx)
    }
    fn new_view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
}
impl EmptyForm {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            _fields: EmptyFormFields,
            focus_handle: cx.focus_handle(),
        }
    }
}
impl Render for EmptyForm {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context(CONTEXT)
            .id("empty-form")
            .size_full()
            .p_4()
            .justify_start()
            .gap_3()
            .child(Separator::horizontal())
            .child(v_form())
            .child(Separator::horizontal())
    }
}
