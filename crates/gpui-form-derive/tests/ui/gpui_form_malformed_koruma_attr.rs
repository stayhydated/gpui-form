use gpui_form_derive::GpuiForm;

struct InputState;

impl InputState {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

gpui_form_derive::component_shape! {
    struct InputShape {
        type State = InputState;
        compatibility<Value>
        where
            Value: 'static;
        value_storage = direct;
    }
}

#[derive(GpuiForm)]
#[gpui_form(koruma)]
struct Demo {
    #[gpui_form(component(crate::InputShape))]
    #[koruma]
    name: String,
}

fn main() {}
