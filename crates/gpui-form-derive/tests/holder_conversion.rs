use gpui_form_derive::GpuiForm;
use gpui_form_runtime::shape::{ComponentShape, NoComponentValueBinding, RequireValue};

#[derive(Clone, Debug, Eq, PartialEq)]
struct NonDefault(String);

struct State;

impl State {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

struct RequiredShape;

impl ComponentShape for RequiredShape {
    type State = State;
    type RequiredValuePolicy = RequireValue;
    type ValueBindingPolicy = NoComponentValueBinding;

    fn new(window: &mut gpui::Window, cx: &mut gpui::Context<'_, Self::State>) -> Self::State {
        State::new(window, cx)
    }
}

#[derive(Clone, Debug, Eq, GpuiForm, PartialEq)]
struct RequiredDemo {
    #[gpui_form(RequiredShape)]
    value: NonDefault,
}

#[derive(Clone, Debug, Eq, GpuiForm, PartialEq)]
struct DefaultedDemo {
    #[gpui_form(RequiredShape, default = NonDefault("fallback".to_string()))]
    value: NonDefault,
}

#[test]
fn missing_required_shape_value_reports_field_name() {
    let err = RequiredDemoFormValueHolder::try_from(RequiredDemoFormValueHolder { value: None })
        .expect_err("missing required shape value should be fallible");

    assert_eq!(err.field_name, "value");
    assert_eq!(err.to_string(), "Missing required value for field 'value'");
}

#[test]
fn defaulted_required_shape_value_remains_infallible() {
    let model = DefaultedDemo::from(DefaultedDemoFormValueHolder { value: None });

    assert_eq!(
        model,
        DefaultedDemo {
            value: NonDefault("fallback".to_string())
        }
    );
}
