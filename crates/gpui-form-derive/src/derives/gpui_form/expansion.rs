use darling::FromDeriveInput as _;
use gpui_form_codegen::{
    CratePaths,
    components::RequiredValue,
    metadata::{optional_rust_expr_tokens, rust_expr_tokens, rust_type_tokens},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;
use syn::GenericParam;

use crate::derives::gpui_form::cfg_attr::flatten_cfg_attr_in_derive_input;
use crate::derives::gpui_form::planner::plan_form;
use crate::derives::gpui_form::structs::{
    ComponentStruct, FieldPlan, GpuiFormOptions, StoragePlan, ValidationRule,
};
use crate::derives::gpui_form::value_holder::{
    generate_value_holder, holder_conversion_can_fail_metadata_tokens,
};

fn field_value_presence_tokens(was_optional: bool, required_value: &RequiredValue) -> TokenStream {
    let facade_crate = CratePaths::resolve().gpui_form;
    if was_optional {
        quote! {
            #facade_crate::schema::registry::FieldValuePresence::Optional
        }
    } else {
        let requires_value_tokens = required_value.metadata_tokens();
        quote! {
            if #requires_value_tokens {
                #facade_crate::schema::registry::FieldValuePresence::RequiresValue
            } else {
                #facade_crate::schema::registry::FieldValuePresence::DirectStorage
            }
        }
    }
}

fn phantom_type_tokens(generics: &syn::Generics) -> TokenStream {
    let params: Vec<TokenStream> = generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(param) => {
                let ident = &param.ident;
                Some(quote! { #ident })
            },
            GenericParam::Lifetime(param) => {
                let lifetime = &param.lifetime;
                Some(quote! { &#lifetime () })
            },
            GenericParam::Const(_) => None,
        })
        .collect();

    if params.is_empty() {
        quote! { () }
    } else {
        quote! { (#(#params),*) }
    }
}

fn marker_field_tokens(generics: &syn::Generics) -> TokenStream {
    if generics
        .params
        .iter()
        .any(|param| matches!(param, GenericParam::Type(_) | GenericParam::Lifetime(_)))
    {
        let phantom_type = phantom_type_tokens(generics);
        quote! {
            #[doc(hidden)]
            pub __gpui_form_marker: ::core::marker::PhantomData<fn() -> #phantom_type>,
        }
    } else {
        quote! {}
    }
}

pub fn expand_gpui_form(
    derive_input: DeriveInput,
    options: GpuiFormOptions,
) -> proc_macro2::TokenStream {
    let facade_crate = CratePaths::resolve().gpui_form;
    let original_input = derive_input.clone();
    let derive_input = flatten_cfg_attr_in_derive_input(derive_input);

    let parsed = match ComponentStruct::from_derive_input(&derive_input) {
        Ok(parsed) => parsed,
        Err(e) => return e.write_errors(),
    };

    let struct_name = &parsed.ident;
    let generate_inventory = options.generate_shape && parsed.no_inventory.is_none();
    if generate_inventory && !original_input.generics.params.is_empty() {
        return syn::Error::new_spanned(
            struct_name,
            "`#[derive(GpuiForm)]` cannot register generic forms in inventory; add `#[gpui_form(no_inventory)]` to opt out",
        )
        .to_compile_error();
    }
    let components_holder_name = format_ident!("{}FormFields", struct_name);
    let components_base_declarations_name = format_ident!("{}FormComponents", struct_name);
    let (impl_generics, ty_generics, where_clause) = original_input.generics.split_for_impl();
    let declaration_generics = &original_input.generics;
    let form_fields_marker = marker_field_tokens(&original_input.generics);

    let koruma_options = parsed.koruma.as_ref().map(|k| k.0.clone());

    if parsed.empty.is_some() {
        let enable_koruma = koruma_options.is_some();
        let enable_koruma_fluent = koruma_options.as_ref().map(|k| k.fluent).unwrap_or(false);
        let empty_fields: Vec<FieldPlan> = Vec::new();
        let value_holder_tokens = generate_value_holder(
            &original_input,
            &empty_fields,
            enable_koruma,
            enable_koruma_fluent,
        );
        let shape_impl = if generate_inventory {
            quote! {
                #facade_crate::schema::registry::inventory::submit! {
                    #facade_crate::schema::registry::GpuiFormShape::new(
                        stringify!(#struct_name),
                        &[],
                        #facade_crate::schema::registry::RustPath::new_unchecked(module_path!()),
                        #enable_koruma
                    ).with_skipped_fields(false)
                    .with_holder_conversion_can_fail(false)
                }
            }
        } else {
            quote! {}
        };
        let empty_struct_declarations = if original_input.generics.params.is_empty() {
            quote! {
                pub struct #components_holder_name;
                pub struct #components_base_declarations_name;
            }
        } else {
            quote! {
                pub struct #components_holder_name #declaration_generics #where_clause {
                    #form_fields_marker
                }

                pub struct #components_base_declarations_name #declaration_generics #where_clause {
                    #form_fields_marker
                }
            }
        };

        return quote! {
            #value_holder_tokens

            #shape_impl

            #empty_struct_declarations
        };
    }

    let fields_iter = match &parsed.data {
        darling::ast::Data::Struct(s) => &s.fields,
        _ => unreachable!("GpuiForm derive only supports named structs"),
    };

    if fields_iter.is_empty() {
        return syn::Error::new_spanned(
            &derive_input,
            format!(
                "Struct `{}` has no fields. Add `#[gpui_form(empty)]` attribute to explicitly mark it as an empty form.",
                struct_name
            ),
        )
        .to_compile_error();
    }

    let form_plan = match plan_form(&parsed, &derive_input, koruma_options.as_ref()) {
        Ok(plan) => plan,
        Err(error) => return error.to_compile_error(),
    };
    let field_plans = form_plan.fields;
    let has_skipped_fields = form_plan.has_skipped_fields;
    let enable_koruma_fluent = form_plan.enable_koruma_fluent;
    let effective_enable_koruma = form_plan.effective_enable_koruma;

    let field_structure_tokens: Vec<TokenStream> = field_plans
        .iter()
        .filter_map(|field| field.component_layout())
        .map(|layout| layout.field_structure_tokens.clone())
        .collect();
    let field_base_declarations_tokens: Vec<TokenStream> = field_plans
        .iter()
        .filter_map(|field| field.component_layout())
        .map(|layout| layout.field_base_declarations_tokens.clone())
        .collect();

    let value_holder_tokens = generate_value_holder(
        &original_input,
        &field_plans,
        effective_enable_koruma,
        enable_koruma_fluent,
    );
    let conversion_can_fail = holder_conversion_can_fail_metadata_tokens(&field_plans);

    let field_variant_construction_code: Vec<TokenStream> = field_plans
        .iter()
        .filter_map(|field| {
            if field.is_skipped() {
                return None;
            }

            let component_def = field.component()?;
            let shared = field.shared()?;
            let field_name_str = field.field_name().to_string();
            let value_presence_tokens = field_value_presence_tokens(
                shared.was_optional,
                &shared.storage.required_value(),
            );

            let field_type_tokens = rust_type_tokens(&facade_crate, &shared.form_type);
            let source_value_type_tokens =
                rust_type_tokens(&facade_crate, &shared.source_value_type);
            let needs_inferred_required_rule = field.needs_required_validation()
                && !shared.validation_metadata.has_required();

            let validation_rules: Vec<ValidationRule> = if needs_inferred_required_rule
                && !matches!(shared.storage, StoragePlan::ShapePolicy { .. })
            {
                std::iter::once(ValidationRule::Required)
                    .chain(shared.validation_metadata.iter().cloned())
                    .collect()
            } else {
                shared.validation_metadata.iter().cloned().collect()
            };
            let validation_rule_tokens: Vec<_> = validation_rules
                .iter()
                .map(|rule| {
                    let registry = &facade_crate;
                    match rule {
                        ValidationRule::Required => {
                            quote! { #registry::schema::registry::ValidationRuleId::Required }
                        },
                        ValidationRule::Newtype => {
                            quote! { #registry::schema::registry::ValidationRuleId::Newtype }
                        },
                        ValidationRule::Nested => {
                            quote! { #registry::schema::registry::ValidationRuleId::Nested }
                        },
                        ValidationRule::Validator(name) => {
                            let literal =
                                syn::LitStr::new(name, proc_macro2::Span::call_site());
                            quote! {
                                #registry::schema::registry::ValidationRuleId::Custom(#literal)
                            }
                        },
                    }
                })
                .collect();
            let validation_rules_tokens = if needs_inferred_required_rule
                && let StoragePlan::ShapePolicy { shape, .. } = &shared.storage
            {
                let runtime_crate = CratePaths::resolve().gpui_form_facade_runtime();
                quote! {
                    {
                        const __GPUI_FORM_FIELD_VALIDATIONS: &[#facade_crate::schema::registry::ValidationRuleId] =
                            if <<#shape as #runtime_crate::shape::ComponentShape>::ValueStoragePolicy
                                as #runtime_crate::shape::ComponentValueStoragePolicy>::REQUIRES_VALUE
                            {
                                &[
                                    #facade_crate::schema::registry::ValidationRuleId::Required,
                                    #( #validation_rule_tokens ),*
                                ]
                            } else {
                                &[
                                    #( #validation_rule_tokens ),*
                                ]
                            };
                        __GPUI_FORM_FIELD_VALIDATIONS
                    }
                }
            } else {
                quote! {
                    &[
                        #( #validation_rule_tokens ),*
                    ]
                }
            };

            let default_expr_tokens = shared.default_expr.as_ref().map(|expr| {
                let expr_tokens = rust_expr_tokens(&facade_crate, expr);
                quote! {
                    .with_default(#expr_tokens)
                }
            });

            let component_variant_tokens =
                component_def.component_variant_tokens(&shared.form_type, &field_name_str);
            let from_expr_tokens = optional_rust_expr_tokens(&facade_crate, &shared.from_expr);
            let into_expr_tokens = optional_rust_expr_tokens(&facade_crate, &shared.into_expr);

            Some(quote! {
                #facade_crate::schema::registry::FieldVariant::component(
                    #field_name_str,
                    #field_type_tokens,
                    #value_presence_tokens,
                    #component_variant_tokens
                )
                .with_source_value_type(
                    #source_value_type_tokens
                )
                .with_conversions(#from_expr_tokens, #into_expr_tokens)
                .with_validations(#validation_rules_tokens)
                #default_expr_tokens
            })
        })
        .collect();

    let component_type_check_tokens: Vec<TokenStream> = field_plans
        .iter()
        .filter_map(|field| {
            if field.is_skipped() {
                return None;
            }

            let shared = field.shared()?;
            field
                .component()
                .map(|component| component.type_check_tokens(&shared.form_type))
        })
        .collect();

    let component_type_checks = if component_type_check_tokens.is_empty() {
        quote! {}
    } else {
        quote! {
            impl #impl_generics #components_holder_name #ty_generics #where_clause {
                #[allow(dead_code)]
                fn __gpui_form_component_type_checks() {
                    #(#component_type_check_tokens)*
                }
            }
        }
    };

    let shape_impl = if generate_inventory {
        quote! {
            #facade_crate::schema::registry::inventory::submit! {
                {
                    const __GPUI_FORM_FIELDS: &[#facade_crate::schema::registry::FieldVariant] = &[
                        #(#field_variant_construction_code),*
                    ];

                    #facade_crate::schema::registry::GpuiFormShape::new(
                        stringify!(#struct_name),
                        __GPUI_FORM_FIELDS,
                        #facade_crate::schema::registry::RustPath::new_unchecked(module_path!()),
                        #effective_enable_koruma
                    ).with_skipped_fields(#has_skipped_fields)
                    .with_holder_conversion_can_fail(#conversion_can_fail)
                }
            }
        }
    } else {
        quote! {}
    };

    let components_base_declarations = if original_input.generics.params.is_empty() {
        quote! {
            pub struct #components_base_declarations_name;

            impl #components_base_declarations_name {
              #(#field_base_declarations_tokens)*
            }
        }
    } else {
        let phantom_type = phantom_type_tokens(&original_input.generics);
        quote! {
            pub struct #components_base_declarations_name #declaration_generics #where_clause {
                #[doc(hidden)]
                pub __gpui_form_marker: ::core::marker::PhantomData<fn() -> #phantom_type>,
            }

            impl #impl_generics #components_base_declarations_name #ty_generics #where_clause {
              #(#field_base_declarations_tokens)*
            }
        }
    };

    let expanded = quote! {
        #value_holder_tokens

        pub struct #components_holder_name #declaration_generics #where_clause {
            #(#field_structure_tokens)*
            #form_fields_marker
        }

        #component_type_checks

        #shape_impl

        #components_base_declarations
    };

    expanded
}
