use gpui_form_derive::{ComponentShape, GpuiForm};

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

#[derive(GpuiForm)]
struct DerivedDemo {
    #[gpui_form(DerivedShape)]
    name: String,
}

fn main() {}
