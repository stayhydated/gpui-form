use gpui_form_derive::{ComponentShape, GpuiForm, component_shape};

struct State;

impl<Event: 'static> gpui::EventEmitter<Event> for State {}

impl State {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

#[derive(ComponentShape)]
#[gpui_form_shape(state = State)]
struct DerivedShape;

component_shape! {
    struct WrapperShape {
        type State = State;
    }
}

#[derive(GpuiForm)]
struct DerivedDemo {
    #[gpui_form(DerivedShape)]
    name: String,
}

#[derive(GpuiForm)]
struct WrapperDemo {
    #[gpui_form(WrapperShape.value_binding())]
    name: String,
}

fn main() {}
