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
                #[gpui_form(component(crate::Input))]
                #[cfg_attr(feature = "validation", koruma(koruma_collection::general::RequiredValidation::<Option<_>>::builder()))]
                name: String,

                #[gpui_form(component(crate::NumericShape))]
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
            derive_input,
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
                #[cfg_attr(feature = "ui", gpui_form(component(crate::NumericShape)))]
                #[cfg_attr(feature = "validation", koruma(newtype))]
                pub index: CommonVariableIndex,
            }
        };

        let derive_input: DeriveInput = syn::parse2(tokens).unwrap();

        if let syn::Data::Struct(data_struct) = &derive_input.data {
            let field = data_struct.fields.iter().next().unwrap();
            let result = koruma_derive_core::parse_field(field, 0);

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
                #[gpui_form(component(crate::NumericShape))]
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
                #[gpui_form(component(crate::NumericShape))]
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
                #[gpui_form(component(crate::Input))]
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
                "if<<crate::Inputas::gpui_form::runtime::shape::ComponentShape>::ValueStoragePolicyas::gpui_form::runtime::shape::ComponentValueStoragePolicy>::REQUIRES_VALUE{::gpui_form::schema::registry::FieldValuePresence::RequiresValue}else{::gpui_form::schema::registry::FieldValuePresence::DirectStorage}"
            ),
            "FieldVariant should inherit value-storage policy from the shape: {compact}"
        );
        assert!(
            compact.contains("__TestFormFormValueHolderValueStoragePolicyValidation")
                && compact.contains("ValueStorage<Value>>::is_present(value)")
                && compact
                    .contains("&[::gpui_form::schema::registry::ValidationRuleId::Required,]"),
            "shape-inherited requiredness should use the policy-aware validator and conditional metadata: {compact}"
        );
        assert!(
            compact.contains("impl::core::convert::TryFrom<TestFormFormValueHolder>forTestForm"),
            "Fallible holders should keep the standard TryFrom impl: {compact}"
        );
        assert!(
            compact.contains("with_holder_conversion_can_fail(true)")
                && compact.contains(
                    "with_holder_conversion_runtime_can_fail(false||<<crate::Inputas::gpui_form::runtime::shape::ComponentShape>::ValueStoragePolicyas::gpui_form::runtime::shape::ComponentValueStoragePolicy>::REQUIRES_VALUE)"
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
            "Generated holders should not expose the legacy inherent try_from helper"
        );
    }

    #[test]
    fn test_type_override_and_conversions() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    type = chrono::NaiveDate,
                    source_to_form = |ts| to_form(ts),
                    form_to_source = |dt| to_model(dt),
                    component(crate::DatePickerState)
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
            compact.contains(
                "FieldVariant::builder(\"birth_date\").with_value_type(::gpui_form::schema::registry::RustType::from_macro_tokens_unchecked(\"chrono::NaiveDate\")).with_value_presence(::gpui_form::schema::registry::FieldValuePresence::Optional).component("
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
            compact.contains("with_holder_conversion_can_fail(false)"),
            "Inventory metadata should publish infallible holder conversion API shape: {compact}"
        );
        assert!(
            !compact.contains("pubfntry_from("),
            "Generated holders should not expose the legacy inherent try_from helper"
        );
    }

    #[test]
    fn test_shape_override_keeps_full_type_path() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    type = rust_decimal::Decimal,
                    source_to_form = |value| value,
                    form_to_source = |value| value,
                    component(crate::NumericShape::<_>)
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
            compact.contains(
                "FieldVariant::builder(\"amount\").with_value_type(::gpui_form::schema::registry::RustType::from_macro_tokens_unchecked(\"rust_decimal::Decimal\")).with_value_presence(if<<crate::NumericShape<rust_decimal::Decimal>as::gpui_form::runtime::shape::ComponentShape>::ValueStoragePolicyas::gpui_form::runtime::shape::ComponentValueStoragePolicy>::REQUIRES_VALUE"
            ),
            "FieldVariant should keep the fully-qualified override type in metadata"
        );
        assert!(
            compact.contains(
                "<crate::NumericShape<rust_decimal::Decimal>as::gpui_form::runtime::shape::ComponentShape>::State"
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
    fn test_skipped_fields_still_generate_from_original() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    type = chrono::NaiveDate,
                    source_to_form = |ts| to_form(ts),
                    form_to_source = |dt| to_model(dt),
                    component(crate::DatePickerState)
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
            "skip + source_to_form should no longer emit a compile_error"
        );
        assert!(
            compact.contains("::core::convert::From<TestForm>forTestFormFormValueHolder",),
            "From<Original> for FormValueHolder should be generated even with skipped fields"
        );
        assert!(
            compact.contains("birth_date:from.birth_date.map(") && compact.contains("to_form"),
            "From<Original> for FormValueHolder should still apply `source_to_form` conversion"
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
    }

    #[test]
    fn test_present_fields_json_uses_into_converted_debug_values_for_skipped_forms() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    type = chrono::NaiveDate,
                    source_to_form = |ts| to_form(ts),
                    form_to_source = |dt| to_model(dt),
                    component(crate::DatePickerState)
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
            "present_fields_json() should apply `form_to_source` conversion for optional override fields"
        );
        assert!(
            compact.contains("format!(\"{:?}\",converted)"),
            "present_fields_json() should emit debug-formatted converted values"
        );
    }

    #[test]
    fn test_structured_field_attribute_grammar() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component(
                    crate::DatePickerState,
                    value(
                        type = chrono::NaiveDate,
                        from_source = |ts| to_form(ts),
                        into_source = |dt| to_model(dt)
                    ),
                    default = Timestamp::now()
                ))]
                birth_date: Timestamp,

                #[gpui_form(hidden(
                    value(type = String, from_source = |id| id.to_string(), into_source = |id| id.parse().unwrap()),
                    default = 1_u64
                ))]
                account_id: u64,
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
            compact.contains("RustType::from_macro_tokens_unchecked(\"chrono::NaiveDate\")")
                && compact.contains("pubaccount_id:String"),
            "structured value(type = ...) should drive component metadata and hidden holder storage: {compact}"
        );
        assert!(
            compact.contains("to_form") && compact.contains("to_model"),
            "structured from_source/into_source conversions should be emitted: {compact}"
        );
        assert!(
            compact.contains(".with_default(::gpui_form::schema::registry::RustExpr::from_macro_tokens_unchecked(\"Timestamp::now()\"))")
                && compact.contains("::core::convert::Into::into(1_u64)"),
            "structured default options should be emitted for inventory and value-holder defaults: {compact}"
        );
    }

    #[test]
    fn test_default_uses_into_conversion() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component(crate::Input), default = "test@example.com")]
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
                #[gpui_form(component(crate::Input), default = "test@example.com")]
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
            compact.contains("::core::convert::From<TestForm>forTestFormFormValueHolder",),
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
                #[gpui_form(component(crate::ui::BioInput))]
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
                    "<crate::ui::BioInputas::gpui_form::runtime::shape::ComponentShape>::State"
                ),
            "Component shape field should use the shape-derived suffix and shape state type"
        );

        assert!(
            compact.contains(
                "<crate::ui::BioInputas::gpui_form::runtime::shape::ComponentShape>::new(window,cx)"
            ),
            "Component shape constructor should delegate to shape::new"
        );

        assert!(
            compact.contains(
                "FieldVariant::builder(\"bio\").with_value_type(::gpui_form::schema::registry::RustType::from_macro_tokens_unchecked(\"String\")).with_value_presence(if<<crate::ui::BioInputas::gpui_form::runtime::shape::ComponentShape>::ValueStoragePolicy"
            ),
            "FieldVariant metadata should be shape-only and omit legacy behavior metadata"
        );

        assert!(
            compact.contains("with_render(if<<crate::ui::BioInputas::gpui_form::runtime::shape::ComponentShape>::RenderComponentas::gpui_form::runtime::shape::ComponentRender<"),
            "FieldVariant should inherit render component metadata from the shape: {compact}"
        );
        assert!(
            compact.contains(
                "FieldComponentVariant::new(::gpui_form::schema::registry::RustPath::from_macro_tokens_unchecked(\"crate::ui::BioInput\"))"
            ),
            "FieldVariant should carry the component shape path: {compact}"
        );
        assert!(
            compact.contains("with_value_binding(if<<crate::ui::BioInputas::gpui_form::runtime::shape::ComponentShape>::ValueBindingPolicyas::gpui_form::runtime::shape::ComponentValueBindingPolicy>::VALUE_BINDING"),
            "FieldVariant should inherit component value binding metadata from the shape: {compact}"
        );
        assert!(
            compact.contains("PROTOTYPING.field_suffix")
                && compact.contains("ComponentSuffix::new(\"input\")"),
            "FieldVariant should inherit shape prototyping metadata with a generated fallback: {compact}"
        );
    }

    #[test]
    fn test_component_shape_rejects_field_level_field_suffix() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component(crate::state::TagsState), field_suffix = "tags")]
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
            compact.contains("unknowngpui_formfieldoption`field_suffix`"),
            "field-level field_suffix should be rejected: {compact}"
        );
    }

    #[test]
    fn test_component_shape_rejects_associated_function_metadata_syntax() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(crate::state::TagsState::field_suffix("tags"))]
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
            compact.contains("component(MyShape)")
                || compact.contains("expectedgpui_formfieldoption"),
            "old positional metadata syntax should be rejected: {compact}"
        );
    }

    #[test]
    fn test_component_shape_generates_fields() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component(gpui_form_collection::input::Input::<_>), default = "test@example.com")]
                email: String,

                #[gpui_form(component(gpui_form_collection::switch::Switch))]
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
            "explicit component shape syntax should generate a component-suffixed field: {compact}"
        );
        assert!(
            compact.contains("Into::into(\"test@example.com\")"),
            "explicit component syntax should compose with key-value field helpers"
        );
        assert!(
            compact.contains("pubenabled:"),
            "explicit component syntax should generate a value-holder field: {compact}"
        );
        assert!(
            compact.contains(
                "if<<gpui_form_collection::switch::Switchas::gpui_form::runtime::shape::ComponentShape>::ValueStoragePolicyas::gpui_form::runtime::shape::ComponentValueStoragePolicy>::REQUIRES_VALUE{::gpui_form::schema::registry::FieldValuePresence::RequiresValue}else{::gpui_form::schema::registry::FieldValuePresence::DirectStorage}"
            ),
            "explicit component syntax should inherit required-value metadata from the shape"
        );
        assert!(
            compact.contains("PROTOTYPING.field_suffix")
                && compact.contains("ComponentSuffix::new(\"input\")"),
            "component inventory metadata should inherit shape prototyping metadata with a generated fallback"
        );
    }

    #[test]
    fn test_component_shape_accepts_shorthand_syntax() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component(gpui_form_collection::switch::Switch))]
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
            compact.contains("pubenabled:"),
            "shorthand component syntax should generate a value-holder field: {compact}"
        );
    }

    #[test]
    fn test_component_shape_rejects_duplicate_component_expression() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    component(gpui_form_collection::input::Input::<_>),
                    component(crate::Input)
                )]
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
            compact.contains("duplicate`component`option"),
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

                #[gpui_form(skip, skip)]
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
    fn test_component_value_binding_policy_assertion_is_generated() {
        let tokens = quote! {
            #[derive(GpuiForm)]
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
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("AssertComponentValueBindingPolicy<Shape,Value")
                && compact.contains(
                    "__gpui_form_assert_name_component_value_binding::<crate::Input,String"
                )
                && compact.contains("assert_component_value_binding_policy()"),
            "shape value-binding policy should emit a derive-time assertion through the runtime helper: {compact}"
        );
        assert!(
            compact.contains("__gpui_form_assert_name_declared_component_shape")
                && compact.contains("__gpui_form_assert_name_component_value_compatibility")
                && compact.contains("__gpui_form_assert_name_component_value_binding"),
            "component type checks should be split into field-named assertions: {compact}"
        );
    }

    #[test]
    fn test_field_variant_metadata_records_form_value_holder_conversion_shape() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    component(crate::Input),
                    type = crate::types::AccountCode,
                    source_to_form = crate::types::AccountCode::new,
                    form_to_source = crate::types::AccountCode::into_string
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
            compact.contains(
                "FieldVariant::builder(\"account_no\").with_value_type(::gpui_form::schema::registry::RustType::from_macro_tokens_unchecked(\"crate::types::AccountCode\")).with_value_presence(if<<crate::Inputas::gpui_form::runtime::shape::ComponentShape>::ValueStoragePolicy"
            ),
            "FieldVariant should store the form-side value type: {compact}"
        );
        assert!(
            compact.contains(
                "with_source_value_type(::gpui_form::schema::registry::RustType::from_macro_tokens_unchecked(\"String\"))"
            ),
            "FieldVariant should store the source model value type: {compact}"
        );
        assert!(
            compact.contains(
                "if<<crate::Inputas::gpui_form::runtime::shape::ComponentShape>::ValueStoragePolicyas::gpui_form::runtime::shape::ComponentValueStoragePolicy>::REQUIRES_VALUE{::gpui_form::schema::registry::FieldValuePresence::RequiresValue}else{::gpui_form::schema::registry::FieldValuePresence::DirectStorage}"
            ),
            "FieldVariant should inherit generated value-storage policy from the shape: {compact}"
        );
        assert!(
            compact.contains("with_conversions(Some(::gpui_form::schema::registry::RustExpr::from_macro_tokens_unchecked(\"crate::types::AccountCode::new\")),Some(::gpui_form::schema::registry::RustExpr::from_macro_tokens_unchecked(\"crate::types::AccountCode::into_string\")))"),
            "FieldVariant should store source/form conversion expressions: {compact}"
        );
    }

    #[test]
    fn test_component_shape_rejects_field_value_storage_override() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component(gpui_form_collection::switch::Switch), value_storage = direct)]
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
            compact.contains("unknowngpui_formfieldoption`value_storage`"),
            "field-level value_storage override should be rejected: {compact}"
        );
    }

    #[test]
    fn test_component_shape_infers_field_type_generic() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
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
                "<gpui_form_collection::input::Input<crate::types::AccountCode>as::gpui_form::runtime::shape::ComponentShape>::State"
            ),
            "component shape `_` should be resolved to the field type in FormFields: {compact}"
        );
        assert!(
            compact.contains("pubaccount_no_input:::gpui::Entity<"),
            "generated FormFields identifiers should use the component shape name suffix: {compact}"
        );
        assert!(
            compact.contains("PROTOTYPING.field_suffix")
                && compact.contains("ComponentSuffix::new(\"input\")"),
            "inventory metadata should use the resolved generated suffix: {compact}"
        );
        assert!(
            compact.contains(
                "FieldComponentVariant::new(::gpui_form::schema::registry::RustPath::from_macro_tokens_unchecked(\"gpui_form_collection::input::Input<crate::types::AccountCode>\"))"
            ),
            "component shape metadata should store the resolved shape path: {compact}"
        );
    }

    #[test]
    fn test_component_shape_generates_multiword_component_suffix() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component(gpui_form_collection::date_picker::DatePicker))]
                birth_date: chrono::NaiveDate,
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
            compact.contains("pubbirth_date_date_picker:::gpui::Entity<"),
            "multiword shape names should produce component-suffixed generated fields: {compact}"
        );
        assert!(
            compact.contains("PROTOTYPING.field_suffix")
                && compact.contains("ComponentSuffix::new(\"date_picker\")"),
            "inventory metadata should use the resolved generated suffix: {compact}"
        );
    }

    #[test]
    fn test_component_shape_rejects_component_specific_metadata_methods() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    component(gpui_form_collection::select::Select::<_>),
                    searchable = true
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
            compact.contains("unknowngpui_formfieldoption`searchable`"),
            "component-specific methods should fail in gpui_form attributes: {compact}"
        );
    }

    #[test]
    fn test_component_shape_rejects_unknown_infinite_select_helper() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(
                    component(gpui_form_component::infinite_select::InfiniteSelect::<_>),
                    searchable_with_max_depth = 3
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
            compact.contains("unknowngpui_formfieldoption`searchable_with_max_depth`"),
            "unsupported infinite-select helper should fail: {compact}"
        );
    }

    #[test]
    fn test_component_shape_rejects_invalid_field_suffix() {
        let tokens = quote! {
            #[derive(GpuiForm)]
            struct TestForm {
                #[gpui_form(component(crate::Input), field_suffix = "input-field")]
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
            compact.contains("unknowngpui_formfieldoption`field_suffix`"),
            "field-level field_suffix should be rejected before value parsing: {compact}"
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
            },
        );

        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("cannotregistergenericformsininventory")
                && compact.contains("gpui_form(no_inventory)"),
            "generic inventory registration should require an explicit opt-out: {compact}"
        );
    }
}
