use gpui_form_derive::GpuiForm;

#[derive(Clone, Debug)]
struct NonDefault(String);

struct State;

impl State {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

gpui_form_derive::component_shape! {
    struct DirectStorageShape {
        type State = State;
        value_storage = direct;

        impl<T> gpui_form_runtime::shape::ComponentShapeFor<T> for DirectStorageShape {}
    }
}

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form(component(shape = DirectStorageShape))]
    value: NonDefault,
}

fn main() {}
