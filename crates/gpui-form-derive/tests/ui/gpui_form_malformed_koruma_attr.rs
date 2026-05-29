use gpui_form_derive::GpuiForm;

struct InputShape;
struct InputState;

impl gpui_form_runtime::shape::ComponentShape for InputShape {
    type State = InputState;
    type RequiredValuePolicy = gpui_form_runtime::shape::AllowMissingValue;
    type ValueBindingPolicy = gpui_form_runtime::shape::NoComponentValueBinding;

    fn new(
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<'_, Self::State>,
    ) -> Self::State {
        InputState
    }
}

#[derive(GpuiForm)]
#[gpui_form(koruma)]
struct Demo {
    #[gpui_form(crate::InputShape)]
    #[koruma]
    name: String,
}

fn main() {}
