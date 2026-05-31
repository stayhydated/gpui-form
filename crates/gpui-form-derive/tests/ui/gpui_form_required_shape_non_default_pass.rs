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
        compatibility<Value>
        where
            Value: 'static;
    }
}

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form(component(RequiredShape))]
    value: NonDefault,

    #[gpui_form(hidden(default = String::new()))]
    name: String,
}

fn main() {}
