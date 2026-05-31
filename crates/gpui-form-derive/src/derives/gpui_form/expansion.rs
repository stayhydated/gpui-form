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
use crate::derives::gpui_form::intent::ComponentStruct;
use crate::derives::gpui_form::ir::{
    ComponentFieldPlan, DeriveContext, FieldPlan, ValidationRule, holder_conversion_plan,
};
use crate::derives::gpui_form::planner::plan_form;
use crate::derives::gpui_form::structs::GpuiFormOptions;
use crate::derives::gpui_form::value_holder::generate_value_holder;

fn field_value_presence_tokens(
    context: &DeriveContext,
    was_optional: bool,
    required_value: &RequiredValue,
) -> TokenStream {
    let facade_crate = &context.paths.gpui_form;
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

fn component_state_type_tokens(context: &DeriveContext, field: &ComponentFieldPlan) -> TokenStream {
    let shape = field.component.shape.shape();
    let runtime_crate = context.paths.gpui_form_facade_runtime();
    quote! {
        <#shape as #runtime_crate::shape::ComponentShape>::State
    }
}

fn component_field_structure_tokens(
    context: &DeriveContext,
    field: &ComponentFieldPlan,
) -> TokenStream {
    let field_ident = &field.component.field_ident;
    let gpui_crate = &context.paths.gpui;
    let state_type = component_state_type_tokens(context, field);
    quote! {
        pub #field_ident: #gpui_crate::Entity<#state_type>,
    }
}

fn component_base_declaration_tokens(
    context: &DeriveContext,
    field: &ComponentFieldPlan,
) -> TokenStream {
    let field_ident = &field.component.field_ident;
    let shape = field.component.shape.shape();
    let runtime_crate = context.paths.gpui_form_facade_runtime();
    let gpui_crate = &context.paths.gpui;
    let state_type = component_state_type_tokens(context, field);

    quote! {
        pub fn #field_ident(
            window: &mut #gpui_crate::Window,
            cx: &mut #gpui_crate::Context<'_, #state_type>,
        ) -> #state_type {
            <#shape as #runtime_crate::shape::ComponentShape>::new(window, cx)
        }
    }
}

pub fn expand_gpui_form(
    derive_input: DeriveInput,
    options: GpuiFormOptions,
) -> proc_macro2::TokenStream {
    let original_input = derive_input.clone();
    let context = DeriveContext::new(original_input.ident.clone(), CratePaths::resolve());
    let facade_crate = &context.paths.gpui_form;
    let derive_input = flatten_cfg_attr_in_derive_input(derive_input);

    let parsed = match ComponentStruct::from_derive_input(&derive_input) {
        Ok(parsed) => parsed,
        Err(e) => return e.write_errors(),
    };

    let struct_name = &context.original_ident;
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
        let conversion_plan = match holder_conversion_plan(&empty_fields) {
            Ok(conversion_plan) => conversion_plan,
            Err(error) => return error.to_compile_error(),
        };
        let conversion_metadata = conversion_plan.metadata_tokens(&context);
        let value_holder_tokens = match generate_value_holder(
            &context,
            &original_input,
            &empty_fields,
            &conversion_plan,
            enable_koruma,
            enable_koruma_fluent,
        ) {
            Ok(value_holder_tokens) => value_holder_tokens,
            Err(error) => return error.to_compile_error(),
        };
        let shape_impl = if generate_inventory {
            quote! {
                #facade_crate::schema::registry::inventory::submit! {
                    #facade_crate::schema::registry::GpuiFormShape::new(
                        stringify!(#struct_name),
                        &[],
                        #facade_crate::schema::registry::RustPath::from_macro_tokens_unchecked(module_path!()),
                        #enable_koruma
                    ).with_holder_conversion(#conversion_metadata)
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
        _ => {
            return syn::Error::new_spanned(
                &derive_input,
                "`#[derive(GpuiForm)]` only supports named structs",
            )
            .to_compile_error();
        },
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
    let enable_koruma_fluent = form_plan.enable_koruma_fluent;
    let effective_enable_koruma = form_plan.effective_enable_koruma;

    let field_structure_tokens: Vec<TokenStream> = form_plan
        .component_fields()
        .map(|field| component_field_structure_tokens(&context, field))
        .collect();
    let field_base_declarations_tokens: Vec<TokenStream> = form_plan
        .component_fields()
        .map(|field| component_base_declaration_tokens(&context, field))
        .collect();
    let field_plans = form_plan.fields;
    let conversion_plan = match holder_conversion_plan(&field_plans) {
        Ok(conversion_plan) => conversion_plan,
        Err(error) => return error.to_compile_error(),
    };

    let value_holder_tokens = match generate_value_holder(
        &context,
        &original_input,
        &field_plans,
        &conversion_plan,
        effective_enable_koruma,
        enable_koruma_fluent,
    ) {
        Ok(value_holder_tokens) => value_holder_tokens,
        Err(error) => return error.to_compile_error(),
    };
    let conversion_metadata = conversion_plan.metadata_tokens(&context);

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
                &context,
                shared.was_optional,
                &shared.storage.required_value(),
            );

            let field_type_tokens = rust_type_tokens(&facade_crate, &shared.form_type);
            let source_value_type_tokens =
                rust_type_tokens(&facade_crate, &shared.source_value_type);
            let needs_inferred_required_rule = field.needs_required_validation()
                && !shared.validation_metadata.has_required();

            let validation_rules: Vec<ValidationRule> = if needs_inferred_required_rule
                && !shared.storage.is_shape_policy()
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
                && let Some(shape) = shared.storage.shape_policy()
            {
                let runtime_crate = context.paths.gpui_form_facade_runtime();
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

            let component_variant_tokens = component_def.component_variant_tokens();
            let from_expr_tokens =
                optional_rust_expr_tokens(&facade_crate, shared.source_to_form_expr());
            let into_expr_tokens =
                optional_rust_expr_tokens(&facade_crate, shared.form_to_source_expr());
            let value_spec_tokens = quote! {
                #facade_crate::schema::registry::FieldValueSpec::new(
                    #field_type_tokens,
                    #source_value_type_tokens,
                    #value_presence_tokens
                )
                .with_conversions(
                    #facade_crate::schema::registry::ConversionMetadata::new(
                        #from_expr_tokens,
                        #into_expr_tokens
                    )
                )
                .with_validations(#validation_rules_tokens)
                #default_expr_tokens
            };

            Some(quote! {
                #facade_crate::schema::registry::FieldVariant::component(
                    #field_name_str,
                    #value_spec_tokens,
                    #component_variant_tokens
                )
            })
        })
        .collect();

    let component_type_check_tokens: Vec<TokenStream> = field_plans
        .iter()
        .filter_map(|field| {
            if field.is_skipped() {
                return None;
            }

            field
                .component()
                .map(|component| component.type_check_tokens())
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
                        #facade_crate::schema::registry::RustPath::from_macro_tokens_unchecked(module_path!()),
                        #effective_enable_koruma
                    ).with_holder_conversion(#conversion_metadata)
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
