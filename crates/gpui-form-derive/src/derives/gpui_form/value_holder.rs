mod impls;
mod present;
mod storage;

use impls::{
    generate_clone_impl, generate_debug_impl, generate_default_impl, generate_from_wrapped_field,
    generate_to_wrapped_field,
};
pub(crate) use present::field_variant_ident;
use present::{
    generate_present_field_entry, generate_present_field_variant,
    required_validation_builder_tokens, value_storage_policy_validator_tokens,
};
use storage::{
    conversion_signature_assertions, form_base_type, form_field_type_tokens,
    generate_conversion_error_type, shape_policy_type_tokens, try_from_field_tokens,
};

use gpui_form_codegen::metadata::{
    apply_form_to_source_conversion_tokens, apply_source_to_form_conversion_tokens,
    source_default_tokens,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use std::collections::HashMap;
use syn::{DeriveInput, Type};

use crate::derives::gpui_form::holder_plan::{
    HolderConversionMode, HolderFieldPlan, HolderOriginalFieldPlan, PresentFieldPlan,
    ValueHolderPlan,
};
use crate::derives::gpui_form::ir::{DeriveContext, HolderFieldIr, HolderStoragePlan};
use crate::derives::gpui_form::koruma::validator_use_to_tokens;

type ValueHolderResult<T> = syn::Result<T>;

/// Generates the FormValueHolder struct and its implementations.
pub(super) fn generate_value_holder(
    context: &DeriveContext,
    original_input: &DeriveInput,
    holder_plan: &ValueHolderPlan<'_>,
    enable_koruma: bool,
    enable_koruma_fluent: bool,
    generate_mcp: bool,
) -> ValueHolderResult<TokenStream> {
    let has_skipped_fields = holder_plan.has_skipped_fields();
    let rendered_field_plans = holder_plan.rendered_fields();
    let rendered_fields: Vec<&HolderFieldIr> =
        rendered_field_plans.iter().map(|plan| plan.field).collect();
    let original_ident = &original_input.ident;
    let wrapped_ident = format_ident!("{}FormValueHolder", original_ident);
    let storage_ident = wrapped_ident.clone();
    let conversion_error_ident = format_ident!("{}ConversionError", wrapped_ident);
    let required_policy_validator_ident =
        format_ident!("__{}ValueStoragePolicyValidation", wrapped_ident);
    let required_policy_validator_builder_ident =
        format_ident!("__{}ValueStoragePolicyValidationBuilder", wrapped_ident);
    let required_policy_validator_target_ident =
        format_ident!("__{}ValueStoragePolicyValidationTarget", wrapped_ident);
    let required_policy_validator_target_trait_ident =
        format_ident!("__{}ValueStoragePolicyValidationTargetTrait", wrapped_ident);
    let has_any_required = holder_plan.has_any_required();
    let has_shape_policy_required = holder_plan.has_shape_policy_required();

    let mut field_attrs: HashMap<String, Vec<TokenStream>> = HashMap::new();

    for planned_field in rendered_field_plans {
        let f = planned_field.field;
        let field_name = f.field_name().to_string();

        if enable_koruma {
            let required_validation = required_validation_builder_tokens(
                context,
                f,
                &required_policy_validator_ident,
                &required_policy_validator_target_ident,
            )?;

            let validation = f.validation();
            let validation_plan = &planned_field.validation;
            let has_existing_validations = validation_plan.has_existing_validations;
            let has_newtype = validation_plan.is_newtype;
            let has_nested = validation_plan.is_nested;

            if required_validation.is_some()
                || has_existing_validations
                || has_newtype
                || has_nested
            {
                let mut koruma_items: Vec<TokenStream> = Vec::new();

                if let Some(required_validation) = required_validation {
                    koruma_items.push(required_validation);
                }

                let existing_field_validations: Vec<TokenStream> = validation
                    .field_validators
                    .iter()
                    .map(validator_use_to_tokens)
                    .collect();
                koruma_items.extend(existing_field_validations);

                let existing_element_validations: Vec<TokenStream> = validation
                    .element_validators
                    .iter()
                    .map(validator_use_to_tokens)
                    .collect();
                if !existing_element_validations.is_empty() {
                    koruma_items.push(quote! {
                        each(#(#existing_element_validations),*)
                    });
                }

                if validation_plan.is_newtype {
                    koruma_items.insert(0, quote! { newtype });
                }
                if validation_plan.is_nested {
                    koruma_items.insert(0, quote! { nested });
                }

                if !koruma_items.is_empty() {
                    field_attrs.insert(field_name, vec![quote! { #[koruma(#(#koruma_items),*)] }]);
                }
            }
        }
    }

    let needs_koruma_for_required = has_any_required && enable_koruma;
    // If koruma is enabled on the form, always derive Koruma so validate() is available,
    // even when there are no inferred validators.
    let needs_koruma_derive = enable_koruma || needs_koruma_for_required;

    let mut derives: Vec<TokenStream> = Vec::new();
    if has_skipped_fields {
        let facade_crate = &context.paths.gpui_form;
        derives.push(quote! { #facade_crate::bon::Builder });
    }
    if needs_koruma_derive {
        if enable_koruma_fluent {
            derives.push(quote! { ::koruma::Koruma });
            derives.push(quote! { ::koruma::KorumaAllFluent });
        } else {
            derives.push(quote! { ::koruma::Koruma });
        }
    }

    let derive_output = if derives.is_empty() {
        quote! {}
    } else {
        quote! { #[derive(#(#derives),*)] }
    };
    let builder_attr = if has_skipped_fields {
        let facade_crate = &context.paths.gpui_form;
        quote! { #[builder(crate = #facade_crate::bon)] }
    } else {
        quote! {}
    };

    let conversion_error_type = generate_conversion_error_type(&conversion_error_ident);
    let (original_impl_generics, ty_generics, where_clause) =
        original_input.generics.split_for_impl();
    let conversion_signature_assertions = if holder_plan.has_any_conversion() {
        conversion_signature_assertions(
            rendered_field_plans,
            &wrapped_ident,
            quote! { #original_impl_generics },
            quote! { #where_clause },
        )?
    } else {
        quote! {}
    };
    let holder_declaration_generics = original_input.generics.clone();
    let holder_impl_generics = original_input.generics.clone();
    let mut holder_where_clause = where_clause.cloned();
    if holder_plan.has_shape_policy_storage() {
        for planned_field in rendered_field_plans {
            let f = planned_field.field;
            if planned_field.storage.was_optional || !planned_field.storage.is_shape_policy {
                continue;
            }

            let base_type = form_base_type(f);
            let policy = shape_policy_type_tokens(context, f)?;
            let runtime_crate = context.paths.gpui_form_facade_runtime();
            let validation = &planned_field.validation;
            let wc = holder_where_clause.get_or_insert_with(|| syn::parse_quote!(where));
            if needs_koruma_derive
                && (validation.has_existing_validations
                    || validation.is_nested
                    || validation.is_newtype)
            {
                wc.predicates.push(syn::parse_quote! {
                    #policy: #runtime_crate::shape::ValueStorage<
                        #base_type,
                        Storage = #base_type
                    >
                });
            } else {
                wc.predicates.push(syn::parse_quote! {
                    #policy: #runtime_crate::shape::ValueStorage<#base_type>
                });
            }
        }
    }
    let (holder_impl_generics, holder_ty_generics, _) = holder_impl_generics.split_for_impl();
    let mut default_storage_assertions: Vec<TokenStream> = Vec::new();
    for planned_field in rendered_field_plans {
        if let Some(shape) = planned_field.default.shape_default_policy {
            let f = planned_field.field;
            let base_type = form_base_type(f);
            let runtime_crate = context.paths.gpui_form_facade_runtime();
            let assert_ident = format_ident!(
                "__{}_assert_{}_default_storage",
                wrapped_ident,
                f.field_name()
            );
            let mut assertion_where_clause = where_clause.cloned();
            let wc = assertion_where_clause.get_or_insert_with(|| syn::parse_quote!(where));
            wc.predicates.push(syn::parse_quote! {
                <#shape as #runtime_crate::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicy:
                    #runtime_crate::shape::DefaultValueStorage<#base_type>
            });
            default_storage_assertions.push(quote! {
                #[allow(dead_code, non_snake_case)]
                fn #assert_ident #original_impl_generics() #assertion_where_clause {}
            });
        }
    }

    let field_definitions: Vec<TokenStream> = rendered_fields
        .iter()
        .map(|f| -> ValueHolderResult<TokenStream> {
            let field_name = f.field_name();
            let field_type = form_field_type_tokens(context, f)?;
            let attrs = field_attrs
                .get(&field_name.to_string())
                .cloned()
                .unwrap_or_default();
            Ok(quote! {
                #(#attrs)* pub #field_name: #field_type
            })
        })
        .collect::<ValueHolderResult<Vec<_>>>()?;

    let to_wrapped_fields: Vec<TokenStream> = rendered_fields
        .iter()
        .map(|f| generate_to_wrapped_field(context, f))
        .collect::<ValueHolderResult<Vec<_>>>()?;

    let from_wrapped_fields: Vec<TokenStream> = rendered_fields
        .iter()
        .map(|f| generate_from_wrapped_field(context, f))
        .collect::<ValueHolderResult<Vec<_>>>()?;

    let try_from_fields: Vec<TokenStream> = rendered_fields
        .iter()
        .map(|f| try_from_field_tokens(context, f, quote! { from }, &conversion_error_ident))
        .collect::<ValueHolderResult<Vec<_>>>()?;

    let present_lifetime: syn::Lifetime = syn::parse_quote!('__gpui_form_present);
    let present_field_ident = format_ident!("{}PresentField", wrapped_ident);
    let present_field_borrows = rendered_field_plans
        .iter()
        .any(|plan| plan.present.borrows_source_value);
    let mut present_field_generics = original_input.generics.clone();
    if present_field_borrows {
        present_field_generics.params.insert(
            0,
            syn::GenericParam::Lifetime(syn::parse_quote!('__gpui_form_present)),
        );
    }
    let (_, present_field_ty_generics, _) = present_field_generics.split_for_impl();
    let present_field_variants: Vec<TokenStream> = rendered_field_plans
        .iter()
        .map(|plan| generate_present_field_variant(plan.field, &present_lifetime, &plan.present))
        .collect();
    let present_field_enum = quote! {
        #[derive(Debug)]
        pub enum #present_field_ident #present_field_generics #where_clause {
            #(#present_field_variants),*
        }
    };
    let present_field_entries: Vec<TokenStream> = rendered_fields
        .iter()
        .map(|f| generate_present_field_entry(context, f, &present_field_ident, &present_lifetime))
        .collect::<ValueHolderResult<Vec<_>>>()?;
    let present_fields_signature = if present_field_borrows {
        quote! {
            pub fn present_fields<'__gpui_form_present>(
                &'__gpui_form_present self
            ) -> Vec<#present_field_ident #present_field_ty_generics>
        }
    } else {
        quote! {
            pub fn present_fields(&self) -> Vec<#present_field_ident #present_field_ty_generics>
        }
    };

    let from_where_clause = holder_where_clause.clone();
    let skipped_params: Vec<TokenStream> = holder_plan
        .skipped_fields()
        .iter()
        .map(|f| {
            let field_name = &f.field.field_name;
            let ty = &f.field.original_type;
            quote! { #field_name: #ty }
        })
        .collect();

    let into_original_fields: Vec<TokenStream> = holder_plan
        .original_fields()
        .iter()
        .map(|field| -> ValueHolderResult<TokenStream> {
            match field {
                HolderOriginalFieldPlan::Skipped(skipped) => {
                    let field_name = &skipped.field_name;
                    Ok(quote! { #field_name })
                },
                HolderOriginalFieldPlan::Rendered(shared) => {
                    try_from_field_tokens(context, shared, quote! { self }, &conversion_error_ident)
                },
            }
        })
        .collect::<ValueHolderResult<Vec<_>>>()?;

    let skipped_fields_impl = match holder_plan.conversion_mode() {
        HolderConversionMode::SkippedFields => {
            quote! {
                #present_field_enum

                impl #holder_impl_generics #storage_ident #holder_ty_generics #holder_where_clause {
                    #present_fields_signature {
                        let mut entries = Vec::new();
                        #(#present_field_entries)*
                        entries
                    }

                    pub fn into_original(
                        self,
                        #(#skipped_params),*
                    ) -> Result<#original_ident #ty_generics, #conversion_error_ident> {
                        Ok(#original_ident {
                            #(#into_original_fields),*
                        })
                    }
                }
            }
        },
        HolderConversionMode::FallibleRequired => {
            quote! {
                impl #holder_impl_generics ::core::convert::TryFrom<#storage_ident #holder_ty_generics>
                    for #original_ident #ty_generics #holder_where_clause
                {
                    type Error = #conversion_error_ident;

                    fn try_from(from: #storage_ident #holder_ty_generics) -> Result<Self, Self::Error> {
                        Ok(Self {
                            #(#try_from_fields),*
                        })
                    }
                }

                impl #holder_impl_generics #storage_ident #holder_ty_generics #holder_where_clause {
                    pub fn try_into_original(
                        self
                    ) -> Result<#original_ident #ty_generics, #conversion_error_ident> {
                        <#original_ident #ty_generics as ::core::convert::TryFrom<
                            #storage_ident #holder_ty_generics
                        >>::try_from(self)
                    }
                }
            }
        },
        HolderConversionMode::Infallible => {
            quote! {
                impl #holder_impl_generics ::core::convert::From<#storage_ident #holder_ty_generics>
                    for #original_ident #ty_generics #from_where_clause
                {
                    fn from(from: #storage_ident #holder_ty_generics) -> Self {
                        Self {
                            #(#from_wrapped_fields),*
                        }
                    }
                }

                impl #holder_impl_generics #storage_ident #holder_ty_generics #holder_where_clause {
                    pub fn into_original(self) -> #original_ident #ty_generics {
                        ::core::convert::From::from(self)
                    }
                }
            }
        },
    };

    let required_policy_validator = if needs_koruma_derive && has_shape_policy_required {
        value_storage_policy_validator_tokens(
            context,
            &required_policy_validator_ident,
            &required_policy_validator_builder_ident,
            &required_policy_validator_target_ident,
            &required_policy_validator_target_trait_ident,
            enable_koruma_fluent,
        )
    } else {
        quote! {}
    };

    let facade_crate = &context.paths.gpui_form;
    let mcp_value_holder_impl = if generate_mcp {
        quote! {
            impl #holder_impl_generics #facade_crate::mcp::McpFormValueHolder
                for #storage_ident #holder_ty_generics #holder_where_clause
            {
                type Form = #original_ident #ty_generics;
            }
        }
    } else {
        quote! {}
    };

    let mut tokens = quote! {
        #conversion_error_type
        #required_policy_validator
        #conversion_signature_assertions
        #(#default_storage_assertions)*
        #mcp_value_holder_impl
        #derive_output
        #builder_attr
        pub struct #storage_ident #holder_declaration_generics #holder_where_clause {
            #(#field_definitions),*
        }

        impl #holder_impl_generics ::core::convert::From<#original_ident #ty_generics>
            for #storage_ident #holder_ty_generics #holder_where_clause
        {
            fn from(from: #original_ident #ty_generics) -> Self {
                Self {
                    #(#to_wrapped_fields),*
                }
            }
        }

        #skipped_fields_impl
    };

    let mut default_where_clause = holder_where_clause.clone();
    for planned_field in rendered_field_plans {
        if planned_field.default.shape_default_policy.is_some() {
            let f = planned_field.field;
            let base_type = form_base_type(f);
            let policy = shape_policy_type_tokens(context, f)?;
            let runtime_crate = context.paths.gpui_form_facade_runtime();
            let wc = default_where_clause.get_or_insert_with(|| syn::parse_quote!(where));
            wc.predicates.push(syn::parse_quote! {
                #policy: #runtime_crate::shape::DefaultValueStorage<#base_type>
            });
        }
    }

    let default_impl = generate_default_impl(
        context,
        &rendered_fields,
        &storage_ident,
        quote! { #holder_impl_generics },
        quote! { #holder_ty_generics },
        quote! { #default_where_clause },
    )?;
    tokens = quote! {
        #tokens
        #default_impl
    };
    let clone_impl = generate_clone_impl(
        context,
        &rendered_fields,
        &storage_ident,
        quote! { #holder_impl_generics },
        quote! { #holder_ty_generics },
        holder_where_clause.clone(),
    )?;
    let debug_impl = generate_debug_impl(
        context,
        &rendered_fields,
        &storage_ident,
        quote! { #holder_impl_generics },
        quote! { #holder_ty_generics },
        holder_where_clause,
    )?;
    tokens = quote! {
        #tokens
        #clone_impl
        #debug_impl
    };

    Ok(tokens)
}
