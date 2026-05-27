use gpui_form_derive::GpuiForm;

struct NumericShape;
struct InputShape;
struct NumericState;
struct InputState;

impl gpui_form_runtime::shape::ComponentShape for NumericShape {
    type State = NumericState;
    type RequiredValuePolicy = gpui_form_runtime::shape::AllowMissingValue;

    fn new(
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<'_, Self::State>,
    ) -> Self::State {
        NumericState
    }
}

impl gpui_form_runtime::shape::ComponentShape for InputShape {
    type State = InputState;
    type RequiredValuePolicy = gpui_form_runtime::shape::AllowMissingValue;

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
    #[gpui_form(component = crate::NumericShape)]
    #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(18).max(167))]
    age: u32,

    #[gpui_form(component = crate::NumericShape)]
    #[koruma(koruma_collection::numeric::PositiveValidation::<_>::builder())]
    score: u32,

    #[gpui_form(component = crate::InputShape)]
    name: String,
}

fn main() {
    let _ = DemoFormValueHolder::default();
}
