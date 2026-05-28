#[cfg(test)]
mod gpui_form_tests {
    use super::super::*;
    use crate::derives::gpui_form::koruma;
    use koruma_derive_core::{ParseFieldResult, ValidatorAttr};
    use quote::quote;
    use syn::DeriveInput;

    fn compact_tokens(tokens: &str) -> String {
        tokens.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn test_koruma_field_parsing_with_cfg_attr() {
        let tokens = quote! {
            struct Test {
                #[cfg_attr(feature = "validation", koruma(SomeValidator::<_>::builder()))]
                field: u32,
            }
        };
        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();

        if let syn::Data::Struct(data_struct) = &derive_input.data {
            let field = data_struct.fields.iter().next().unwrap();
            let result = koruma_derive_core::parse_field(field, 0);

            match result {
                ParseFieldResult::Valid(info) => {
                    assert!(
                        !info.validation.field_validators.is_empty(),
                        "Should find validators in cfg_attr"
                    );
                    assert_eq!(
                        info.validation.field_validators[0].name().to_string(),
                        "SomeValidator",
                        "Should extract correct validator name"
                    );
                },
                ParseFieldResult::Skip => {
                    panic!(
                        "parse_field returned Skip - koruma_derive_core may not be handling cfg_attr correctly"
                    );
                },
                ParseFieldResult::Error(e) => {
                    panic!("parse_field returned Error: {}", e);
                },
            }
        } else {
            panic!("Expected struct data");
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

        if let syn::Data::Struct(data_struct) = &derive_input.data {
            let field = data_struct.fields.iter().next().unwrap();
            let result = koruma_derive_core::parse_field(field, 0);

            match result {
                ParseFieldResult::Valid(info) => {
                    assert!(info.is_newtype(), "Should detect newtype in cfg_attr");
                },
                ParseFieldResult::Skip => {
                    panic!(
                        "parse_field returned Skip - koruma_derive_core may not be handling cfg_attr correctly for newtype"
                    );
                },
                ParseFieldResult::Error(e) => {
                    panic!("parse_field returned Error: {}", e);
                },
            }
        } else {
            panic!("Expected struct data");
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

        if let syn::Data::Struct(data_struct) = &derive_input.data {
            let field = data_struct.fields.iter().next().unwrap();
            let result = koruma_derive_core::parse_field(field, 0);

            match result {
                ParseFieldResult::Valid(info) => {
                    assert!(info.is_nested(), "Should detect nested in cfg_attr");
                },
                ParseFieldResult::Skip => {
                    panic!(
                        "parse_field returned Skip - koruma_derive_core may not be handling cfg_attr correctly for nested"
                    );
                },
                ParseFieldResult::Error(e) => {
                    panic!("parse_field returned Error: {}", e);
                },
            }
        } else {
            panic!("Expected struct data");
        }
    }

    #[test]
    fn test_cfg_attr_end_to_end_validation_generation() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            #[gpui_form(koruma(fluent))]
            struct TestForm {
                #[gpui_form(component = crate::Input)]
                #[cfg_attr(feature = "validation", koruma(koruma_collection::general::RequiredValidation::<Option<_>>::builder()))]
                name: String,

                #[gpui_form(component = crate::NumericShape)]
                #[cfg_attr(feature = "validation", koruma(koruma_collection::numeric::PositiveValidation::<_>::builder()))]
                age: u32,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();

        if let syn::Data::Struct(data_struct) = &derive_input.data {
            for (idx, field) in data_struct.fields.iter().enumerate() {
                let result = koruma_derive_core::parse_field(field, idx);
                match result {
                    ParseFieldResult::Valid(info) => {
                        assert!(
                            !info.validation.field_validators.is_empty(),
                            "Field {} should have validators detected from cfg_attr",
                            idx
                        );
                        let validator_names: Vec<String> = info
                            .validation
                            .field_validators
                            .iter()
                            .map(|v| v.name().to_string())
                            .collect();
                        assert!(
                            !validator_names.is_empty(),
                            "Field {} should have named validators",
                            idx
                        );
                    },
                    ParseFieldResult::Skip => {
                        panic!("Field {} should have validators, got Skip", idx);
                    },
                    ParseFieldResult::Error(e) => {
                        panic!("Field {} parsing failed: {}", idx, e);
                    },
                }
            }
        }

        let expanded = expansion::expand_gpui_form(
            derive_input.clone(),
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let expanded_str = expanded.to_string();

        assert!(
            expanded_str.contains("Koruma") || expanded_str.contains("koruma"),
            "Generated code should include Koruma-related types"
        );

        assert!(
            expanded_str.contains("RequiredValidation")
                && expanded_str.contains("PositiveValidation"),
            "Generated code should preserve parsed validation builders: {}",
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
                #[cfg_attr(feature = "ui", gpui_form(component = crate::NumericShape))]
                #[cfg_attr(feature = "validation", koruma(newtype))]
                pub index: CommonVariableIndex,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();

        if let syn::Data::Struct(data_struct) = &derive_input.data {
            let field = data_struct.fields.iter().next().unwrap();
            let result = koruma_derive_core::parse_field(field, 0);

            eprintln!("=== DEBUG: parse_field result for CommonVRead.index ===");
            match &result {
                ParseFieldResult::Valid(info) => {
                    eprintln!("  Result: Valid");
                    eprintln!("  is_newtype: {}", info.is_newtype());
                    eprintln!(
                        "  field_validators.len(): {}",
                        info.validation.field_validators.len()
                    );
                    for (idx, v) in info.validation.field_validators.iter().enumerate() {
                        eprintln!("    validator[{}]: {}", idx, v.name());
                    }
                },
                ParseFieldResult::Skip => {
                    eprintln!("  Result: Skip");
                },
                ParseFieldResult::Error(e) => {
                    eprintln!("  Result: Error({})", e);
                },
            }

            match result {
                ParseFieldResult::Valid(info) => {
                    assert!(
                        info.is_newtype(),
                        "Should detect newtype validation in nested cfg_attr"
                    );
                },
                ParseFieldResult::Skip => {
                    panic!(
                        "koruma_derive_core returned Skip for field with koruma(newtype) - cfg_attr not being handled!"
                    );
                },
                ParseFieldResult::Error(e) => {
                    panic!("koruma_derive_core returned Error: {}", e);
                },
            }
        }

        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let expanded_str = expanded.to_string();
        eprintln!("=== Generated code (first 1000 chars) ===");
        eprintln!("{}", &expanded_str[..expanded_str.len().min(1000)]);

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
                note: Option<String>,
                kind: Option<u8>,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
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
    fn test_validator_attr_to_tokens_normalizes_builder_chain() {
        let validator: ValidatorAttr = syn::parse_quote!(
            koruma_collection::numeric::RangeValidation::<_>::builder()
                .min(18)
                .max(167)
        );

        let tokens = koruma::validator_attr_to_tokens(&validator);
        let compact = compact_tokens(&tokens.to_string());

        assert_eq!(
            compact,
            compact_tokens(
                "koruma_collection::numeric::RangeValidation::<_>::builder().min(18).max(167)"
            )
        );
    }

    #[test]
    fn test_gpui_form_preserves_builder_chain_koruma_validators() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            #[gpui_form(koruma)]
            struct TestForm {
                #[gpui_form(component = crate::NumericShape)]
                #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(18).max(167))]
                age: u32,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains(&compact_tokens(
                "koruma_collection::numeric::RangeValidation::<_>::builder().min(18).max(167)"
            )),
            "Generated value holder should preserve builder-chain koruma validators: {compact}"
        );
    }

    #[test]
    fn test_gpui_form_preserves_zero_setter_koruma_builder_chains() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            #[gpui_form(koruma)]
            struct TestForm {
                #[gpui_form(component = crate::NumericShape)]
                #[koruma(koruma_collection::numeric::PositiveValidation::<_>::builder())]
                age: u32,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains(&compact_tokens(
                "koruma_collection::numeric::PositiveValidation::<_>::builder()"
            )),
            "Generated value holder should preserve zero-setter koruma builder chains: {compact}"
        );
    }

    #[test]
    fn test_gpui_form_inherits_required_policy_without_field_override() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            #[gpui_form(koruma)]
            struct TestForm {
                #[gpui_form(component = crate::Input)]
                name: String,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains(
                "with_requires_value(<crate::Inputas::gpui_form_runtime::shape::ComponentShape>::REQUIRES_VALUE)"
            ),
            "FieldVariant should inherit required-value policy from the shape: {compact}"
        );
        assert!(
            !compact.contains("RequiredValidation"),
            "shape-inherited requiredness should not depend on a field-level required override: {compact}"
        );
    }

    #[test]
    fn test_type_override_and_conversions() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    type = chrono::NaiveDate,
                    from = |ts| to_form(ts),
                    into = |dt| to_model(dt),
                    component = crate::DatePickerState
                )]
                birth_date: Option<Timestamp>,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("FieldVariant::new(\"birth_date\",\"chrono::NaiveDate\",true"),
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
    }

    #[test]
    fn test_shape_override_keeps_full_type_path() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    type = rust_decimal::Decimal,
                    from = |value| value,
                    into = |value| value,
                    component = crate::NumericShape::<_>
                )]
                amount: f64,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("FieldVariant::new(\"amount\",\"rust_decimal::Decimal\",false"),
            "FieldVariant should keep the fully-qualified override type in metadata"
        );
        assert!(
            compact.contains(
                "<crate::NumericShape<rust_decimal::Decimal>as::gpui_form_runtime::shape::ComponentShape>::State"
            ),
            "Component shape `_` should resolve to the override type in state metadata"
        );
        assert!(
            compact.contains("with_shape_path(\"crate::NumericShape<rust_decimal::Decimal>\")"),
            "Component shape metadata should preserve the fully-qualified override type"
        );
    }

    #[test]
    fn test_skipped_fields_still_generate_from_original() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    type = chrono::NaiveDate,
                    from = |ts| to_form(ts),
                    into = |dt| to_model(dt),
                    component = crate::DatePickerState
                )]
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
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            !compact.contains("compile_error!"),
            "skip + from should no longer emit a compile_error"
        );
        assert!(
            compact.contains("impl::core::convert::From<TestForm>forTestFormFormValueHolder"),
            "From<Original> for FormValueHolder should be generated even with skipped fields"
        );
        assert!(
            compact.contains("birth_date:from.birth_date.map(") && compact.contains("to_form"),
            "From<Original> for FormValueHolder should still apply `from` conversion"
        );
        assert!(
            !compact.contains("impl::core::convert::From<TestFormFormValueHolder>forTestForm"),
            "Reverse From<FormValueHolder> for Original should remain disabled when skipped fields exist"
        );
        assert!(
            compact.contains(
                "pubfninto_original(self,skip_me:bool)->Result<TestForm,TestFormFormValueHolderConversionError>"
            ),
            "Skipped-field forms should keep strict into_original(self, skipped...) conversion"
        );
    }

    #[test]
    fn test_present_fields_json_uses_into_converted_debug_values_for_skipped_forms() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    type = chrono::NaiveDate,
                    into = |dt| to_model(dt),
                    component = crate::DatePickerState
                )]
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
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("pubfnpresent_fields_json(&self)->String"),
            "Skipped-field value holders should generate present_fields_json()"
        );
        assert!(
            compact.contains(
                "letconverted=self.birth_date.clone().map(|value|(|dt|to_model(dt))(value));"
            ),
            "present_fields_json() should apply `into` conversion for optional override fields"
        );
        assert!(
            compact.contains("format!(\"{:?}\",converted)"),
            "present_fields_json() should emit debug-formatted converted values"
        );
    }

    #[test]
    fn test_default_uses_into_conversion() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component = crate::Input, default = "test@example.com")]
                email: String,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
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
                #[gpui_form(component = crate::Input, default = "test@example.com")]
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
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("impl::core::convert::From<TestForm>forTestFormFormValueHolder"),
            "Skipped-field forms should still generate From<Original> for value holder"
        );
        assert!(
            compact.contains(
                "letdefault_original:String=::core::convert::Into::into(\"test@example.com\")"
            ),
            "From<Original> should emit a typed default value to avoid Into inference ambiguity"
        );
    }

    #[test]
    fn test_component_shape_generates_shape_based_state_and_constructor() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component = crate::shapes::BioInputShape
                    .component(crate::ui::BioInput)
                    .value_binding())]
                bio: String,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("pubbio_input:::gpui::Entity<")
                && compact.contains(
                    "<crate::shapes::BioInputShapeas::gpui_form_runtime::shape::ComponentShape>::State"
                ),
            "Component shape field should use the shape-derived suffix and shape state type"
        );

        assert!(
            compact.contains(
                "<crate::shapes::BioInputShapeas::gpui_form_runtime::shape::ComponentShape>::new(window,cx)"
            ),
            "Component shape constructor should delegate to shape::new"
        );

        assert!(
            compact.contains("FieldVariant::new(\"bio\",\"String\",false)"),
            "FieldVariant metadata should be shape-only and omit behaviour metadata"
        );

        assert!(
            compact.contains("with_component_path("),
            "FieldVariant should carry the component path: {compact}"
        );
        assert!(
            compact.contains("with_shape_path(\"crate::shapes::BioInputShape\")"),
            "FieldVariant should carry the component shape path: {compact}"
        );
        assert!(
            compact.contains("with_value_binding(true)"),
            "FieldVariant should record opt-in component value binding: {compact}"
        );
        assert!(
            compact.contains(
                "with_prototyping_field_suffix(<crate::shapes::BioInputShapeas::gpui_form_runtime::shape::ComponentShape>::PROTOTYPING.field_suffix)"
            ),
            "FieldVariant should inherit component shape prototyping metadata: {compact}"
        );
    }

    #[test]
    fn test_component_shape_field_suffix_override() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component = crate::state::TagsState.field_suffix("tags"))]
                labels: Vec<String>,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("publabels_tags:::gpui::Entity<"),
            "field_suffix should control generated FormFields names: {compact}"
        );
        assert!(
            compact.contains("with_prototyping_field_suffix(Some(\"tags\"))"),
            "FieldVariant should record explicit prototyping suffix: {compact}"
        );
    }

    #[test]
    fn test_component_shorthand_generates_shape_fields() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(gpui_form_collection::input::Input::<_>, default = "test@example.com")]
                email: String,

                #[gpui_form(gpui_form_collection::switch::Switch)]
                enabled: bool,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("pubemail_input:::gpui::Entity<"),
            "positional component syntax should generate a shape-backed field: {compact}"
        );
        assert!(
            compact.contains("Into::into(\"test@example.com\")"),
            "positional component syntax should compose with key-value field helpers"
        );
        assert!(
            compact.contains("pubenabled:"),
            "positional component syntax should generate a value-holder field: {compact}"
        );
        assert!(
            compact.contains(
                "with_requires_value(<gpui_form_collection::switch::Switchas::gpui_form_runtime::shape::ComponentShape>::REQUIRES_VALUE)"
            ),
            "positional component syntax should inherit required-value metadata from the shape"
        );
    }

    #[test]
    fn test_component_shorthand_rejects_duplicate_component_expression() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(gpui_form_collection::input::Input::<_>, component = crate::Input)]
                email: String,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("multiplecomponentexpressionswereprovided"),
            "duplicate component syntax should produce an actionable error: {compact}"
        );
    }

    #[test]
    fn test_gpui_form_rejects_duplicate_field_options() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(type = String, type = std::string::String)]
                name: String,

                #[gpui_form(skip, skip = true)]
                hidden: bool,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("duplicate`type`option")
                && compact.contains("removetheduplicate`type`entry"),
            "duplicate type should produce an actionable error: {compact}"
        );
    }

    #[test]
    fn test_component_value_binding_assertion_is_generated_for_opt_in_field() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component = crate::Input.value_binding())]
                name: String,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains(
                "::gpui_form_runtime::shape::assert_component_value_binding::<crate::Input,String>()"
            ),
            "value_binding() should emit a derive-time ComponentValueBinding assertion through the runtime helper: {compact}"
        );
    }

    #[test]
    fn test_field_variant_metadata_records_form_value_holder_conversion_shape() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    component = crate::Input,
                    type = crate::types::AccountCode,
                    from = crate::types::AccountCode::new,
                    into = crate::types::AccountCode::into_string
                )]
                account_no: String,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact
                .contains("FieldVariant::new(\"account_no\",\"crate::types::AccountCode\",false"),
            "FieldVariant should store the form-side value type: {compact}"
        );
        assert!(
            compact.contains("with_source_value_type(\"String\")"),
            "FieldVariant should store the source model value type: {compact}"
        );
        assert!(
            compact.contains(
                "with_requires_value(<crate::Inputas::gpui_form_runtime::shape::ComponentShape>::REQUIRES_VALUE)"
            ),
            "FieldVariant should inherit generated required-value policy from the shape: {compact}"
        );
        assert!(
            compact.contains("with_conversions(Some(\"crate::types::AccountCode::new\"),Some(\"crate::types::AccountCode::into_string\"))"),
            "FieldVariant should store source/form conversion expressions: {compact}"
        );
    }

    #[test]
    fn test_component_shape_rejects_field_required_value_override() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component = gpui_form_collection::switch::Switch.requires_value(false))]
                enabled: bool,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("unknowncomponentmetadata`requires_value`"),
            "field-level requires_value override should be rejected: {compact}"
        );
    }

    #[test]
    fn test_component_shape_infers_field_type_generic() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component = gpui_form_collection::input::Input::<_>)]
                account_no: crate::types::AccountCode,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains(
                "<gpui_form_collection::input::Input<crate::types::AccountCode>as::gpui_form_runtime::shape::ComponentShape>::State"
            ),
            "component shape `_` should be resolved to the field type in FormFields: {compact}"
        );
        assert!(
            compact.contains("pubaccount_no_input:::gpui::Entity<"),
            "collection shape names should drive generated FormFields suffixes: {compact}"
        );
        assert!(
            compact.contains(
                "with_shape_path(\"gpui_form_collection::input::Input<crate::types::AccountCode>\")"
            ),
            "component shape metadata should store the resolved shape path: {compact}"
        );
    }

    #[test]
    fn test_component_shape_rejects_component_specific_metadata_methods() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    component = gpui_form_collection::select::Select::<_>::searchable(true)
                )]
                country: crate::types::Country,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("unknowncomponentmetadata`searchable`"),
            "component-specific methods should fail in gpui_form attributes: {compact}"
        );
    }

    #[test]
    fn test_component_shape_rejects_removed_combined_infinite_select_helper() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    component = gpui_form_component::infinite_select::InfiniteSelect::<_>::searchable_with_max_depth(3)
                )]
                location: crate::types::Country,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
        let expanded = expansion::expand_gpui_form(
            derive_input,
            structs::GpuiFormOptions {
                generate_shape: true,
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("unknowncomponentmetadata`searchable_with_max_depth`"),
            "removed pre-bon infinite-select helper should fail: {compact}"
        );
    }
}
