use gpui_form_derive::GpuiForm;
use koruma::ValidationError as _;
use koruma::{NewtypeTryFromInner as _, NewtypeValue as _};
use koruma_collection::collection::LenValidation;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NonDefault(String);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceCode(String);

fn source_code_to_form(value: SourceCode) -> String {
    value.0
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FallibleCode(String);

#[derive(Clone, Copy)]
pub struct FallibleCodeError;

impl fmt::Debug for FallibleCodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("expected at least two characters")
    }
}

fn fallible_code_to_form(value: FallibleCode) -> String {
    value.0
}

fn try_form_to_fallible_code(value: String) -> Result<FallibleCode, FallibleCodeError> {
    if value.len() >= 2 {
        Ok(FallibleCode(value))
    } else {
        Err(FallibleCodeError)
    }
}

#[derive(Clone, Debug, Eq, koruma::Koruma, PartialEq)]
#[koruma(newtype)]
pub struct ValidatedCode(#[koruma(LenValidation::<_>.min(2).max(8))] String);

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
            from_source = source_code_to_form,
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

#[derive(Clone, Debug, Eq, GpuiForm, PartialEq)]
pub struct FallibleConvertedDemo {
    #[gpui_form(hidden(value(
        type = String,
        from_source = fallible_code_to_form,
        try_into_source = try_form_to_fallible_code,
    )))]
    value: FallibleCode,
}

#[derive(Clone, Debug, Eq, GpuiForm, PartialEq)]
pub struct KorumaNewtypeConvertedDemo {
    #[gpui_form(hidden(value(koruma_newtype)))]
    value: ValidatedCode,
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

#[test]
fn fallible_value_conversion_uses_try_into_original() {
    let holder = FallibleConvertedDemoFormValueHolder::from(FallibleConvertedDemo {
        value: FallibleCode("AB".to_string()),
    });
    assert_eq!(holder.value, "AB");

    let model = holder
        .try_into_original()
        .expect("valid form value should convert");
    assert_eq!(
        model,
        FallibleConvertedDemo {
            value: FallibleCode("AB".to_string())
        }
    );

    let err = FallibleConvertedDemoFormValueHolder {
        value: "A".to_string(),
    }
    .try_into_original()
    .expect_err("invalid form value should fail conversion");

    assert_eq!(err.field_name, "value");
    assert_eq!(err.validator_name(), "conversion");
    assert_eq!(err.detail(), Some("expected at least two characters"));
    assert_eq!(
        err.to_string(),
        "Invalid value for field 'value': expected at least two characters"
    );
}

#[test]
fn koruma_newtype_value_conversion_uses_inner_value_holder() {
    let source_value =
        ValidatedCode::try_from_inner("ZX-81".to_string()).expect("fixture should validate");
    let holder = KorumaNewtypeConvertedDemoFormValueHolder::from(KorumaNewtypeConvertedDemo {
        value: source_value,
    });
    assert_eq!(holder.value, "ZX-81");

    let model = holder
        .try_into_original()
        .expect("valid inner value should reconstruct newtype");
    assert_eq!(
        model.value.into_inner(),
        "ZX-81",
        "newtype reconstruction should preserve the edited inner value"
    );

    let err = KorumaNewtypeConvertedDemoFormValueHolder {
        value: "Z".to_string(),
    }
    .try_into_original()
    .expect_err("invalid inner value should fail newtype validation");

    assert_eq!(err.field_name, "value");
    assert_eq!(err.validator_name(), "conversion");
    assert!(
        err.detail()
            .is_some_and(|detail| detail.contains("LenValidation"))
    );
}
