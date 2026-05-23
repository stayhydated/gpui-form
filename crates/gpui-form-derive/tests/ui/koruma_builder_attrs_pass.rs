use gpui_form_derive::GpuiForm;

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
