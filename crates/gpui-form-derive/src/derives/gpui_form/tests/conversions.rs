use super::support::*;

#[test]
fn test_type_override_and_conversions() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        struct TestForm {
            #[gpui_form(component(
                crate::DatePickerState,
                value(
                    type = chrono::NaiveDate,
                    from_source = |ts| to_form(ts),
                    into_source = |dt| to_model(dt),
                )
            ))]
            birth_date: Option<Timestamp>,
        }
    };

    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let expanded = expansion::expand_gpui_form(
        derive_input,
        structs::GpuiFormOptions {
            generate_shape: true,
            generate_mcp: false,
        },
    );

    let compact = compact_tokens(&expanded.to_string());

    assert!(
        compact.contains(
            "FieldVariant::component_for_field(\"birth_date\",::gpui_form::schema::registry::FieldValueSpec::new(::gpui_form::schema::registry::RustType::from_macro_tokens_unchecked(\"chrono::NaiveDate\"),::gpui_form::schema::registry::RustType::from_macro_tokens_unchecked(\"Timestamp\"),::gpui_form::schema::registry::FieldValuePresence::Optional)"
        ),
        "FieldVariant should use override type for metadata"
    );

    assert!(
        compact.contains("birth_date:from.birth_date.map(") && compact.contains("to_form"),
        "From<Original> for FormValueHolder should apply `from` conversion"
    );

    assert!(
        compact.contains("birth_date:from.birth_date.map(") && compact.contains("to_model"),
        "From<FormValueHolder> for Original should apply `into` conversion"
    );
    assert!(
        compact.contains("fnassert_source_to_form<F>(_:F)")
            && compact.contains("F:::core::ops::FnOnce(Timestamp)->chrono::NaiveDate")
            && compact.contains("fnassert_form_to_source<F>(_:F)")
            && compact.contains("F:::core::ops::FnOnce(chrono::NaiveDate)->Timestamp"),
        "Generated holders should assert explicit conversion signatures: {compact}"
    );
    assert!(
        compact.contains("pubfninto_original(self)->TestForm"),
        "Infallible holders should expose into_original(self): {compact}"
    );
    assert!(
        compact.contains("with_holder_conversion(::gpui_form::schema::registry::HolderConversionMetadata::new(::gpui_form::schema::registry::HolderConversionShape::Infallible,false))"),
        "Inventory metadata should publish infallible holder conversion API shape: {compact}"
    );
    assert!(
        !compact.contains("pubfntry_from("),
        "Generated holders should expose TryFrom through the standard trait only"
    );
}

#[test]
fn test_shape_override_keeps_full_type_path() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        struct TestForm {
            #[gpui_form(component(
                crate::NumericShape::<_>,
                value(
                    type = rust_decimal::Decimal,
                    from_source = |value| value,
                    into_source = |value| value,
                )
            ))]
            amount: f64,
        }
    };

    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let expanded = expansion::expand_gpui_form(
        derive_input,
        structs::GpuiFormOptions {
            generate_shape: true,
            generate_mcp: false,
        },
    );

    let compact = compact_tokens(&expanded.to_string());

    assert!(
        compact.contains(
            "FieldVariant::component_for_field(\"amount\",::gpui_form::schema::registry::FieldValueSpec::new(::gpui_form::schema::registry::RustType::from_macro_tokens_unchecked(\"rust_decimal::Decimal\"),::gpui_form::schema::registry::RustType::from_macro_tokens_unchecked(\"f64\"),if<<crate::NumericShape<rust_decimal::Decimal>as::gpui_form::runtime::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicyas::gpui_form::runtime::shape::ComponentValueStoragePolicy>::REQUIRES_VALUE"
        ),
        "FieldVariant should keep the fully-qualified override type in metadata"
    );
    assert!(
        compact.contains(
            "<crate::NumericShape<rust_decimal::Decimal>as::gpui_form::runtime::shape::GpuiComponentShape>::State"
        ),
        "Component shape `_` should resolve to the override type in state metadata"
    );
    assert!(
        compact.contains(
            "FieldComponentVariant::new(::gpui_form::schema::registry::RustPath::from_macro_tokens_unchecked(\"crate::NumericShape<rust_decimal::Decimal>\"))"
        ),
        "Component shape metadata should preserve the fully-qualified override type"
    );
}

#[test]
fn test_skipped_fields_generate_from_original() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        struct TestForm {
            #[gpui_form(component(
                crate::DatePickerState,
                value(
                    type = chrono::NaiveDate,
                    from_source = |ts| to_form(ts),
                    into_source = |dt| to_model(dt),
                )
            ))]
            birth_date: Option<Timestamp>,

            #[gpui_form(skip)]
            skip_me: bool,
        }
    };

    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let expanded = expansion::expand_gpui_form(
        derive_input,
        structs::GpuiFormOptions {
            generate_shape: true,
            generate_mcp: false,
        },
    );

    let compact = compact_tokens(&expanded.to_string());

    assert!(
        !compact.contains("compile_error!"),
        "skip plus a converted component field should not emit a compile_error"
    );
    assert!(
        compact.contains("::core::convert::From<TestForm>forTestFormFormValueHolder",),
        "From<Original> for FormValueHolder should be generated even with skipped fields"
    );
    assert!(
        compact.contains("birth_date:from.birth_date.map(") && compact.contains("to_form"),
        "From<Original> for FormValueHolder should apply `from_source` conversion"
    );
    assert!(
        !compact.contains("::core::convert::From<TestFormFormValueHolder>forTestForm",),
        "Reverse From<FormValueHolder> for Original should remain disabled when skipped fields exist"
    );
    assert!(
        compact.contains(
            "pubfninto_original(self,skip_me:bool)->Result<TestForm,TestFormFormValueHolderConversionError>"
        ),
        "Skipped-field forms should keep strict into_original(self, skipped...) conversion"
    );
    assert!(
        compact.contains("with_holder_conversion(::gpui_form::schema::registry::HolderConversionMetadata::new(::gpui_form::schema::registry::HolderConversionShape::NeedsSkippedFields,true))"),
        "Inventory metadata should record skipped-field holder conversion shape: {compact}"
    );
}

#[test]
fn test_present_fields_uses_typed_converted_values_for_skipped_forms() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        struct TestForm {
            #[gpui_form(component(
                crate::DatePickerState,
                value(
                    type = chrono::NaiveDate,
                    from_source = |ts| to_form(ts),
                    into_source = |dt| to_model(dt),
                )
            ))]
            birth_date: Option<Timestamp>,

            #[gpui_form(skip)]
            skip_me: bool,
        }
    };

    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let expanded = expansion::expand_gpui_form(
        derive_input,
        structs::GpuiFormOptions {
            generate_shape: true,
            generate_mcp: false,
        },
    );

    let compact = compact_tokens(&expanded.to_string());

    assert!(
        compact.contains("pubenumTestFormFormValueHolderPresentField"),
        "Skipped-field value holders should generate a typed present-field enum"
    );
    assert!(
        compact.contains("pubfnpresent_fields(&self)->Vec<TestFormFormValueHolderPresentField>"),
        "Skipped-field value holders should expose typed present_fields() snapshots: {compact}"
    );
    assert!(
        compact.contains("entries.push(TestFormFormValueHolderPresentField::BirthDate((|dt|to_model(dt))(value)));"),
        "present_fields() should apply `into_source` conversion for optional override fields"
    );
}

#[test]
fn test_default_uses_into_conversion() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        struct TestForm {
            #[gpui_form(component(crate::Input, default = "test@example.com"))]
            email: String,
        }
    };

    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let expanded = expansion::expand_gpui_form(
        derive_input,
        structs::GpuiFormOptions {
            generate_shape: true,
            generate_mcp: false,
        },
    );

    let compact = compact_tokens(&expanded.to_string());

    assert!(
        compact.contains("Into::into(\"test@example.com\")"),
        "Default should be wrapped in Into::into for string literals"
    );
}

#[test]
fn test_skipped_forms_with_string_default_emit_typed_default_comparison() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        struct TestForm {
            #[gpui_form(component(crate::Input, default = "test@example.com"))]
            email: String,

            #[gpui_form(skip)]
            skip_me: bool,
        }
    };

    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let expanded = expansion::expand_gpui_form(
        derive_input,
        structs::GpuiFormOptions {
            generate_shape: true,
            generate_mcp: false,
        },
    );

    let compact = compact_tokens(&expanded.to_string());

    assert!(
        compact.contains("::core::convert::From<TestForm>forTestFormFormValueHolder",),
        "Skipped-field forms should generate From<Original> for value holder"
    );
    assert!(
        compact.contains(
            "letdefault_original:String=::core::convert::Into::into(\"test@example.com\")"
        ),
        "From<Original> should emit a typed default value to avoid Into inference ambiguity"
    );
}
