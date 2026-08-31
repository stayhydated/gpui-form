use super::support::*;

#[test]
fn test_koruma_field_parsing_with_cfg_attr() {
    let tokens = quote! {
        struct Test {
            #[cfg_attr(feature = "validation", koruma(SomeValidator::<_>))]
            field: u32,
        }
    };
    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let result = parse_field_after_cfg_attr_flattening(derive_input, 0);

    match result {
        Ok(ParsedDataField::Participating(info)) => {
            assert!(
                !info.field_validators().is_empty(),
                "Should find validators after cfg_attr flattening"
            );
            assert_eq!(
                info.field_validators()[0].validator().name().to_string(),
                "SomeValidator",
                "Should extract correct validator name"
            );
        },
        Ok(ParsedDataField::Unannotated(_)) | Ok(ParsedDataField::Skipped { .. }) => {
            panic!("parse_field returned Skip after gpui-form cfg_attr flattening");
        },
        Err(e) => {
            panic!("parse_field returned Error: {}", e);
        },
    }
}

#[test]
fn test_koruma_field_parsing_newtype_in_cfg_attr() {
    let tokens = quote! {
        struct Test {
            #[cfg_attr(feature = "validation", koruma(newtype))]
            field: SomeNewtype,
        }
    };
    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let result = parse_field_after_cfg_attr_flattening(derive_input, 0);

    match result {
        Ok(ParsedDataField::Participating(info)) => {
            assert!(
                info.is_newtype(),
                "Should detect newtype after cfg_attr flattening"
            );
        },
        Ok(ParsedDataField::Unannotated(_)) | Ok(ParsedDataField::Skipped { .. }) => {
            panic!("parse_field returned Skip after gpui-form cfg_attr flattening for newtype");
        },
        Err(e) => {
            panic!("parse_field returned Error: {}", e);
        },
    }
}

#[test]
fn test_koruma_field_parsing_nested_in_cfg_attr() {
    let tokens = quote! {
        struct Test {
            #[cfg_attr(feature = "validation", koruma(nested))]
            field: NestedStruct,
        }
    };
    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let result = parse_field_after_cfg_attr_flattening(derive_input, 0);

    match result {
        Ok(ParsedDataField::Participating(info)) => {
            assert!(
                info.is_nested(),
                "Should detect nested after cfg_attr flattening"
            );
        },
        Ok(ParsedDataField::Unannotated(_)) | Ok(ParsedDataField::Skipped { .. }) => {
            panic!("parse_field returned Skip after gpui-form cfg_attr flattening for nested");
        },
        Err(e) => {
            panic!("parse_field returned Error: {}", e);
        },
    }
}

#[test]
fn test_cfg_attr_end_to_end_validation_generation() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        #[gpui_form(koruma(fluent))]
        struct TestForm {
            #[gpui_form(component(crate::Input))]
            #[cfg_attr(feature = "validation", koruma(koruma_collection::collection::NonEmptyValidation::<_>))]
            name: String,

            #[gpui_form(component(crate::NumericShape))]
            #[cfg_attr(feature = "validation", koruma(koruma_collection::numeric::PositiveValidation::<_>))]
            age: u32,
        }
    };

    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let flattened_derive_input = flatten_cfg_attr_in_derive_input(derive_input.clone());

    if let syn::Data::Struct(data_struct) = &flattened_derive_input.data {
        for (idx, field) in data_struct.fields.iter().enumerate() {
            let result = koruma_derive_core::parse_field(field, idx);
            match result {
                Ok(ParsedDataField::Participating(info)) => {
                    assert!(
                        !info.field_validators().is_empty(),
                        "Field {} should have validators detected from cfg_attr",
                        idx
                    );
                    let validator_names: Vec<String> = info
                        .field_validators()
                        .iter()
                        .map(|v| v.validator().name().to_string())
                        .collect();
                    assert!(
                        !validator_names.is_empty(),
                        "Field {} should have named validators",
                        idx
                    );
                },
                Ok(ParsedDataField::Unannotated(_)) | Ok(ParsedDataField::Skipped { .. }) => {
                    panic!("Field {} should have validators, got Skip", idx);
                },
                Err(e) => {
                    panic!("Field {} parsing failed: {}", idx, e);
                },
            }
        }
    }

    let expanded = expansion::expand_gpui_form(
        derive_input,
        structs::GpuiFormOptions {
            generate_shape: true,
            generate_mcp: false,
        },
    );

    let expanded_str = expanded.to_string();

    assert!(
        expanded_str.contains("Koruma") || expanded_str.contains("koruma"),
        "Generated code should include Koruma-related types"
    );

    assert!(
        expanded_str.contains("NonEmptyValidation") && expanded_str.contains("PositiveValidation"),
        "Generated code should preserve parsed validation chains: {}",
        &expanded_str[..expanded_str.len().min(500)]
    );

    assert!(
        expanded_str.contains("FieldVariant") || expanded_str.contains("validations"),
        "Generated code should include field variant metadata"
    );
}

#[test]
fn test_real_world_common_v_read_pattern() {
    let tokens = quote! {
        #[cfg_attr(feature = "ui", derive(GpuiForm))]
        #[cfg_attr(feature = "ui", gpui_form(koruma(fluent)))]
        pub struct CommonVRead {
            #[cfg_attr(feature = "ui", gpui_form(component(crate::NumericShape)))]
            #[cfg_attr(feature = "validation", koruma(newtype))]
            pub index: CommonVariableIndex,
        }
    };

    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let result = parse_field_after_cfg_attr_flattening(derive_input.clone(), 0);

    match result {
        Ok(ParsedDataField::Participating(info)) => {
            assert!(
                info.is_newtype(),
                "Should detect newtype validation after cfg_attr flattening"
            );
        },
        Ok(ParsedDataField::Unannotated(_)) | Ok(ParsedDataField::Skipped { .. }) => {
            panic!(
                "koruma_derive_core returned Skip for field with koruma(newtype) after gpui-form cfg_attr flattening"
            );
        },
        Err(e) => {
            panic!("koruma_derive_core returned Error: {}", e);
        },
    }

    let expanded = expansion::expand_gpui_form(
        derive_input,
        structs::GpuiFormOptions {
            generate_shape: true,
            generate_mcp: false,
        },
    );

    let expanded_str = expanded.to_string();

    assert!(
        expanded_str.contains("with_validations"),
        "Generated GpuiFormShape should call with_validations()"
    );
}

#[test]
fn test_koruma_enabled_without_validators_derives_validate() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        #[gpui_form(koruma(fluent))]
        struct OptionalOnlyForm {
            #[gpui_form(hidden)]
            note: Option<String>,
            #[gpui_form(hidden)]
            kind: Option<u8>,
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

    let expanded_str = compact_tokens(&expanded.to_string());

    assert!(
        expanded_str.contains("::koruma::Koruma"),
        "Koruma derive should be emitted when gpui_form(koruma) is enabled, even without validators"
    );
    assert!(
        expanded_str.contains("::koruma::KorumaAllFluent"),
        "KorumaAllFluent derive should be emitted when gpui_form(koruma(fluent)) is enabled"
    );
}

#[test]
fn test_validator_attr_to_tokens_normalizes_direct_chain() {
    let validator: ValidatorAttr = syn::parse_quote!(
        koruma_collection::numeric::RangeValidation::<_>
            .min(18)
            .max(167)
    );

    let tokens = koruma::validator_attr_to_tokens(&validator);
    let compact = compact_tokens(&tokens.to_string());

    assert_eq!(
        compact,
        compact_tokens("koruma_collection::numeric::RangeValidation::<_>.min(18).max(167)")
    );
}

#[test]
fn test_gpui_form_preserves_direct_chain_koruma_validators() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        #[gpui_form(koruma)]
        struct TestForm {
            #[gpui_form(component(crate::NumericShape))]
            #[koruma(koruma_collection::numeric::RangeValidation::<_>.min(18).max(167))]
            age: u32,
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
        compact.contains(&compact_tokens(
            "koruma_collection::numeric::RangeValidation::<_>.min(18).max(167)"
        )),
        "Generated value holder should preserve direct-chain koruma validators: {compact}"
    );
}

#[test]
fn test_gpui_form_preserves_zero_setter_koruma_direct_chains() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        #[gpui_form(koruma)]
        struct TestForm {
            #[gpui_form(component(crate::NumericShape))]
            #[koruma(koruma_collection::numeric::PositiveValidation::<_>)]
            age: u32,
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
        compact.contains(&compact_tokens(
            "koruma_collection::numeric::PositiveValidation::<_>"
        )),
        "Generated value holder should preserve zero-setter koruma direct chains: {compact}"
    );
}

#[test]
fn test_gpui_form_inherits_required_policy_without_field_override() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        #[gpui_form(koruma)]
        struct TestForm {
            #[gpui_form(component(crate::Input))]
            name: String,
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
            "if<<crate::Inputas::gpui_form::runtime::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicyas::gpui_form::runtime::shape::ComponentValueStoragePolicy>::REQUIRES_VALUE{::gpui_form::schema::registry::FieldValuePresence::RequiresValue}else{::gpui_form::schema::registry::FieldValuePresence::DirectStorage}"
        ),
        "FieldVariant should inherit value-storage policy from the shape: {compact}"
    );
    assert!(
        compact.contains("__TestFormFormValueHolderValueStoragePolicyValidation")
            && compact.contains("ValueStorage<Value>>::is_present(value)")
            && compact.contains("&[::gpui_form::schema::registry::ValidationRuleId::Required,]"),
        "shape-inherited requiredness should use the policy-aware validator and conditional metadata: {compact}"
    );
    assert!(
        compact.contains("impl::core::convert::TryFrom<TestFormFormValueHolder>forTestForm"),
        "Fallible holders should keep the standard TryFrom impl: {compact}"
    );
    assert!(
        compact.contains("with_holder_conversion(::gpui_form::schema::registry::HolderConversionMetadata::new(::gpui_form::schema::registry::HolderConversionShape::FallibleRequired")
            && compact.contains(
                "false||<<crate::Inputas::gpui_form::runtime::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicyas::gpui_form::runtime::shape::ComponentValueStoragePolicy>::REQUIRES_VALUE"
            ),
        "Inventory metadata should record the fallible API shape and runtime policy predicate: {compact}"
    );
    assert!(
        compact.contains(
            "pubfntry_into_original(self)->Result<TestForm,TestFormFormValueHolderConversionError>"
        ),
        "Fallible holders should expose try_into_original(self): {compact}"
    );
    assert!(
        !compact.contains("pubfntry_from("),
        "Generated holders should expose TryFrom through the standard trait only"
    );
}
