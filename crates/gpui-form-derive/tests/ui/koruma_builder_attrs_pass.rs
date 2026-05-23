use gpui_form_derive::GpuiForm;

struct NumericShape;
struct InputShape;
struct NumericState;
struct InputState;

impl gpui_form_component::custom::CustomComponentShape for NumericShape {
    type State = NumericState;

    fn new(
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<'_, Self::State>,
    ) -> Self::State {
        NumericState
    }
}

impl gpui_form_component::custom::CustomComponentShape for InputShape {
    type State = InputState;

    fn new(
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<'_, Self::State>,
    ) -> Self::State {
        InputState
    }
}

#[derive(GpuiForm, koruma::Koruma)]
#[gpui_form(koruma)]
struct Demo {
    #[gpui_form(component(custom(shape = crate::NumericShape)))]
    #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(18).max(167))]
    age: u32,

    #[gpui_form(component(custom(shape = crate::NumericShape)))]
    #[koruma(koruma_collection::numeric::PositiveValidation::<_>::builder())]
    score: u32,

    #[gpui_form(component(custom(shape = crate::InputShape)))]
    name: String,
}

fn main() {
    let _ = DemoFormValueHolder::default();
}
