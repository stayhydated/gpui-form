use gpui_form_derive::GpuiForm;
use koruma::ValidationError as _;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NonDefault(String);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceCode(String);

pub struct State;

impl State {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

component_shape_gpui::component_shape! {
    struct RequiredShape {
        state = State;
        values(NonDefault, String);
    }
}

impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for RequiredShape {
    type ValueStoragePolicy = gpui_form_runtime::shape::RequiredValueStorage;
}

component_shape_gpui::component_shape! {
    struct AllowShape {
        state = State;
        value = String;
    }
}

impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for AllowShape {
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}

#[derive(Clone, Debug, Eq, GpuiForm, PartialEq)]
#[gpui_form(koruma)]
pub struct RequiredDemo {
    #[gpui_form(component(RequiredShape))]
    value: NonDefault,
}

#[derive(Clone, Debug, Eq, GpuiForm, PartialEq)]
pub struct DefaultedDemo {
    #[gpui_form(component(
        RequiredShape,
        default = NonDefault("fallback".to_string())
    ))]
    value: NonDefault,
}

#[derive(Clone, Debug, Eq, GpuiForm, PartialEq)]
pub struct ConvertedDefaultDemo {
    #[gpui_form(component(
        RequiredShape,
        value(
            type = String,
            from_source = |value: SourceCode| value.0,
            into_source = SourceCode,
        ),
        default = SourceCode("fallback".to_string())
    ))]
    value: SourceCode,
}

#[derive(Clone, Debug, Eq, GpuiForm, PartialEq)]
pub struct DirectStorageDemo {
    #[gpui_form(component(AllowShape))]
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
fn defaulted_converted_required_shape_uses_storage_strategy() {
    let holder = ConvertedDefaultDemoFormValueHolder::default();
    assert_eq!(holder.value, Some("fallback".to_string()));

    let defaulted = ConvertedDefaultDemoFormValueHolder { value: None }.into_original();
    assert_eq!(
        defaulted,
        ConvertedDefaultDemo {
            value: SourceCode("fallback".to_string())
        }
    );

    let converted = ConvertedDefaultDemoFormValueHolder {
        value: Some("custom".to_string()),
    }
    .into_original();
    assert_eq!(
        converted,
        ConvertedDefaultDemo {
            value: SourceCode("custom".to_string())
        }
    );
}

#[test]
fn direct_storage_shape_value_try_into_original_succeeds() {
    let model = DirectStorageDemoFormValueHolder {
        value: "ready".to_string(),
    }
    .try_into_original()
    .expect("direct storage shape always contains a value");

    assert_eq!(
        model,
        DirectStorageDemo {
            value: "ready".to_string()
        }
    );
}
