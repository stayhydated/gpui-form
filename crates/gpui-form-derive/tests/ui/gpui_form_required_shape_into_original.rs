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
    struct RequiredShape {
        type State = State;

        impl<T> gpui_form_runtime::shape::ComponentShapeFor<T> for RequiredShape {}
    }
}

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form(component(shape = RequiredShape))]
    value: NonDefault,
}

fn main() {
    let holder = DemoFormValueHolder { value: None };
    let _ = holder.into_original();
}
