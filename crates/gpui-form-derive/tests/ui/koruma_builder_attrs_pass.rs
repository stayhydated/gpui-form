use gpui_form_derive::GpuiForm;

struct NumericState;
struct InputState;

impl NumericState {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

impl InputState {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

gpui_form_derive::component_shape! {
    struct NumericShape {
        type State = NumericState;
        value_storage = direct;

        impl<T> gpui_form_runtime::shape::ComponentShapeFor<T> for NumericShape {}
    }
}

gpui_form_derive::component_shape! {
    struct InputShape {
        type State = InputState;
        value_storage = direct;

        impl<T> gpui_form_runtime::shape::ComponentShapeFor<T> for InputShape {}
    }
}

#[derive(GpuiForm, koruma::Koruma)]
#[gpui_form(koruma)]
struct Demo {
    #[gpui_form(shape = crate::NumericShape)]
    #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(18).max(167))]
    age: u32,

    #[gpui_form(shape = crate::NumericShape)]
    #[koruma(koruma_collection::numeric::PositiveValidation::<_>::builder())]
    score: u32,

    #[gpui_form(shape = crate::InputShape)]
    name: String,
}

fn main() {
    let _ = DemoFormValueHolder::default();
}
