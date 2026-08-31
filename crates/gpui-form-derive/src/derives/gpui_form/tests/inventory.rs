use super::support::*;

#[test]
fn test_hidden_inventory_emits_validations_and_omits_skipped_fields() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        #[gpui_form(koruma)]
        struct TestForm {
            #[gpui_form(component(crate::Input))]
            visible: String,

            #[gpui_form(hidden(default = HiddenId::new()))]
            #[koruma(newtype)]
            account_id: HiddenId,

            #[gpui_form(skip)]
            skipped_secret: bool,
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
        compact.contains("FieldVariant::hidden_for_field(\"account_id\",::gpui_form::schema::registry::FieldValueSpec::new(::gpui_form::schema::registry::RustType::from_macro_tokens_unchecked(\"HiddenId\"),::gpui_form::schema::registry::RustType::from_macro_tokens_unchecked(\"HiddenId\"),::gpui_form::schema::registry::FieldValuePresence::DirectStorage)")
            && compact.contains(".with_validations(&[::gpui_form::schema::registry::ValidationRuleId::Newtype])")
            && compact.contains(".with_default(::gpui_form::schema::registry::RustExpr::from_macro_tokens_unchecked(\"HiddenId::new()\"))"),
        "hidden fields should publish validation IDs and defaults in inventory: {compact}"
    );
    assert!(
        compact.contains("FieldVariant::component_for_field(\"visible\""),
        "component field inventory should remain emitted: {compact}"
    );
    assert!(
        !compact.contains("FieldVariant::hidden_for_field(\"skipped_secret\")")
            && !compact.contains("FieldVariant::component_for_field(\"skipped_secret\")"),
        "skipped fields should stay out of field inventory: {compact}"
    );
    assert!(
        compact.contains("with_holder_conversion(::gpui_form::schema::registry::HolderConversionMetadata::new(::gpui_form::schema::registry::HolderConversionShape::NeedsSkippedFields,true))"),
        "skipped fields should be represented by holder conversion metadata: {compact}"
    );
}

#[test]
fn test_field_variant_metadata_records_form_value_holder_conversion_shape() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        struct TestForm {
            #[gpui_form(component(
                crate::Input,
                value(
                    type = crate::types::AccountCode,
                    from_source = crate::types::AccountCode::new,
                    into_source = crate::types::AccountCode::into_string
                )
            ))]
            account_no: String,
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
            "FieldVariant::component_for_field(\"account_no\",::gpui_form::schema::registry::FieldValueSpec::new(::gpui_form::schema::registry::RustType::from_macro_tokens_unchecked(\"crate::types::AccountCode\"),::gpui_form::schema::registry::RustType::from_macro_tokens_unchecked(\"String\"),if<<crate::Inputas::gpui_form::runtime::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicy"
        ),
        "FieldVariant should store the form-side value type: {compact}"
    );
    assert!(
        compact.contains("RustType::from_macro_tokens_unchecked(\"String\")"),
        "FieldVariant should store the source model value type: {compact}"
    );
    assert!(
        compact.contains(
            "if<<crate::Inputas::gpui_form::runtime::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicyas::gpui_form::runtime::shape::ComponentValueStoragePolicy>::REQUIRES_VALUE{::gpui_form::schema::registry::FieldValuePresence::RequiresValue}else{::gpui_form::schema::registry::FieldValuePresence::DirectStorage}"
        ),
        "FieldVariant should inherit generated value-storage policy from the shape: {compact}"
    );
    assert!(
        compact.contains("with_conversions(::gpui_form::schema::registry::ConversionMetadata::new(Some(::gpui_form::schema::registry::RustExpr::from_macro_tokens_unchecked(\"crate::types::AccountCode::new\")),Some(::gpui_form::schema::registry::RustExpr::from_macro_tokens_unchecked(\"crate::types::AccountCode::into_string\"))))"),
        "FieldVariant should store source/form conversion expressions: {compact}"
    );
}

#[test]
fn test_generic_forms_require_inventory_opt_out() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        struct TestForm<T>
        where
            T: Default,
        {
            #[gpui_form(hidden)]
            value: T,
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
        compact.contains("cannotregistergenericformsininventory")
            && compact.contains("gpui_form(no_inventory)"),
        "generic inventory registration should require an explicit opt-out: {compact}"
    );
}
