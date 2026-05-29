use gpui_form::runtime::shape::{
    AllowMissingValue, ComponentShape, NoComponentValueBinding, RequireValue,
};
use gpui_form_derive::GpuiForm;
use koruma::ValidationError as _;

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

struct AllowShape;

impl ComponentShape for AllowShape {
    type State = State;
    type RequiredValuePolicy = AllowMissingValue;
    type ValueBindingPolicy = NoComponentValueBinding;

    fn new(window: &mut gpui::Window, cx: &mut gpui::Context<'_, Self::State>) -> Self::State {
        State::new(window, cx)
    }
}

#[derive(Clone, Debug, Eq, GpuiForm, PartialEq)]
#[gpui_form(koruma)]
struct RequiredDemo {
    #[gpui_form(RequiredShape)]
    value: NonDefault,
}

#[derive(Clone, Debug, Eq, GpuiForm, PartialEq)]
struct DefaultedDemo {
    #[gpui_form(RequiredShape, default = NonDefault("fallback".to_string()))]
    value: NonDefault,
}

#[derive(Clone, Debug, Eq, GpuiForm, PartialEq)]
struct AllowMissingDemo {
    #[gpui_form(AllowShape)]
    value: String,
}

#[test]
fn missing_required_shape_value_reports_field_name() {
    let err = RequiredDemoFormValueHolder { value: None }
        .try_into_original()
        .expect_err("missing required shape value should be fallible");

    assert_eq!(err.field_name, "value");
    assert_eq!(err.to_string(), "Missing required value for field 'value'");
}

#[test]
fn validate_reports_missing_required_shape_value() {
    let err = RequiredDemoFormValueHolder { value: None }
        .validate()
        .expect_err("shape-required missing value should fail Koruma validation");

    assert!(err.has_errors());
    assert!(
        format!("{err:?}").contains("value"),
        "validation error should identify the missing field: {err:?}"
    );
}

#[test]
fn defaulted_required_shape_value_remains_infallible() {
    let model = DefaultedDemoFormValueHolder { value: None }.into_original();

    assert_eq!(
        model,
        DefaultedDemo {
            value: NonDefault("fallback".to_string())
        }
    );
}

#[test]
fn allow_missing_shape_value_exposes_infallible_into_original() {
    let model = AllowMissingDemoFormValueHolder {
        value: "ready".to_string(),
    }
    .into_original();

    assert_eq!(
        model,
        AllowMissingDemo {
            value: "ready".to_string()
        }
    );
}
