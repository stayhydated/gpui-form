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

component_shape_gpui::component_shape! {
    struct NumericShape {
        type State = NumericState;
        value = u32;
    }
}

impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for NumericShape {
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}

component_shape_gpui::component_shape! {
    struct InputShape {
        type State = InputState;
        value = String;
    }
}

impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for InputShape {
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}

#[derive(GpuiForm, koruma::Koruma)]
#[gpui_form(koruma)]
struct Demo {
    #[gpui_form(component(crate::NumericShape))]
    #[koruma(koruma_collection::numeric::RangeValidation::<_>::min(18).max(167))]
    age: u32,

    #[gpui_form(component(crate::NumericShape))]
    #[koruma(koruma_collection::numeric::PositiveValidation::<_>)]
    score: u32,

    #[gpui_form(component(crate::InputShape))]
    name: String,
}

fn main() {
    let _ = DemoFormValueHolder::default();
}
