use gpui_form_derive::GpuiForm;

struct InputShape;
struct InputState;

impl gpui_form_runtime::shape::ComponentShape for InputShape {
    type State = InputState;
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
    type ValueBindingPolicy = gpui_form_runtime::shape::NoComponentValueBinding;

    fn new(
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<'_, Self::State>,
    ) -> Self::State {
        InputState
    }
}

impl<T> gpui_form_runtime::shape::ComponentShapeFor<T> for InputShape {}

#[derive(GpuiForm)]
#[gpui_form(koruma)]
struct Demo {
    #[gpui_form(shape = crate::InputShape)]
    #[koruma]
    name: String,
}

fn main() {}
