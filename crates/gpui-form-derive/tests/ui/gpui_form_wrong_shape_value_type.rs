use gpui_form_derive::GpuiForm;

struct State;

impl State {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

gpui_form_derive::component_shape! {
    struct StringShape {
        type State = State;
        value = String;
    }
}

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form(component(StringShape))]
    value: u64,
}

fn main() {}
