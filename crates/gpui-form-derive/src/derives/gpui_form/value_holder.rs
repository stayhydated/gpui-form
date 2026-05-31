use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use std::collections::HashMap;
use syn::{DeriveInput, Type};

use crate::derives::gpui_form::ir::{
    DeriveContext, FieldPlan, HolderConversionMode, HolderConversionPlan, HolderFieldIr,
    HolderStoragePlan,
};
use crate::derives::gpui_form::koruma::validator_attr_to_tokens;

type ValueHolderResult<T> = syn::Result<T>;

fn form_base_type(field: &HolderFieldIr) -> Type {
    field.form_type().clone()
}

trait HolderStorageStrategy {
    fn shape_policy_type_tokens(
        &self,
        context: &DeriveContext,
        field_name: &syn::Ident,
    ) -> ValueHolderResult<TokenStream>;
    fn storage_type_tokens(
        &self,
        context: &DeriveContext,
        base_type: &Type,
        field_name: &syn::Ident,
    ) -> ValueHolderResult<TokenStream>;
    fn present_tokens(
        &self,
        context: &DeriveContext,
        value: TokenStream,
        base_type: &Type,
        field_name: &syn::Ident,
    ) -> ValueHolderResult<TokenStream>;
    fn default_storage_tokens(
        &self,
        context: &DeriveContext,
        base_type: &Type,
        field_name: &syn::Ident,
    ) -> ValueHolderResult<TokenStream>;
    fn try_into_source_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        access: TokenStream,
        error_type: &syn::Ident,
    ) -> ValueHolderResult<TokenStream>;
    fn from_source_storage_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        access: TokenStream,
    ) -> ValueHolderResult<TokenStream>;
    fn into_source_infallible_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        access: TokenStream,
    ) -> ValueHolderResult<TokenStream>;
    fn present_field_entry_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        present_field_ident: &syn::Ident,
        _present_lifetime: &syn::Lifetime,
    ) -> ValueHolderResult<TokenStream>;
    fn required_validation_builder_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        validator_ident: &syn::Ident,
        target_ident: &syn::Ident,
    ) -> ValueHolderResult<Option<TokenStream>>;
}

impl HolderStorageStrategy for HolderStoragePlan {
    fn shape_policy_type_tokens(
        &self,
        context: &DeriveContext,
        field_name: &syn::Ident,
    ) -> ValueHolderResult<TokenStream> {
        let Some(shape) = self.shape_policy() else {
            return Err(syn::Error::new_spanned(
                field_name,
                "only shape-policy storage has a generated policy type",
            ));
        };
        let runtime_crate = context.paths.gpui_form_facade_runtime();
        Ok(quote! {
            <#shape as #runtime_crate::shape::ComponentShape>::ValueStoragePolicy
        })
    }

    fn storage_type_tokens(
        &self,
        context: &DeriveContext,
        base_type: &Type,
        field_name: &syn::Ident,
    ) -> ValueHolderResult<TokenStream> {
        match self {
            HolderStoragePlan::OriginallyOptional | HolderStoragePlan::RequiredValue => {
                Ok(quote! { Option<#base_type> })
            },
            HolderStoragePlan::Direct => Ok(quote! { #base_type }),
            HolderStoragePlan::ShapePolicy { .. } => {
                let policy = self.shape_policy_type_tokens(context, field_name)?;
                let runtime_crate = context.paths.gpui_form_facade_runtime();
                Ok(quote! {
                    <#policy as #runtime_crate::shape::ValueStorage<#base_type>>::Storage
                })
            },
        }
    }

    fn present_tokens(
        &self,
        context: &DeriveContext,
        value: TokenStream,
        base_type: &Type,
        field_name: &syn::Ident,
    ) -> ValueHolderResult<TokenStream> {
        match self {
            HolderStoragePlan::OriginallyOptional | HolderStoragePlan::RequiredValue => {
                Ok(quote! { Some(#value) })
            },
            HolderStoragePlan::Direct => Ok(value),
            HolderStoragePlan::ShapePolicy { .. } => {
                let policy = self.shape_policy_type_tokens(context, field_name)?;
                let runtime_crate = context.paths.gpui_form_facade_runtime();
                Ok(quote! {
                    <#policy as #runtime_crate::shape::ValueStorage<#base_type>>::present(#value)
                })
            },
        }
    }

    fn default_storage_tokens(
        &self,
        context: &DeriveContext,
        base_type: &Type,
        field_name: &syn::Ident,
    ) -> ValueHolderResult<TokenStream> {
        match self {
            HolderStoragePlan::OriginallyOptional | HolderStoragePlan::RequiredValue => {
                Ok(quote! { None })
            },
            HolderStoragePlan::Direct => {
                let runtime_crate = context.paths.gpui_form_facade_runtime();
                Ok(quote_spanned! {field_name.span()=>
                    <#runtime_crate::shape::DirectValueStorage
                        as #runtime_crate::shape::DefaultValueStorage<
                            #base_type
                        >>::default_storage()
                })
            },
            HolderStoragePlan::ShapePolicy { .. } => {
                let policy = self.shape_policy_type_tokens(context, field_name)?;
                let runtime_crate = context.paths.gpui_form_facade_runtime();
                Ok(quote_spanned! {field_name.span()=>
                    <#policy
                        as #runtime_crate::shape::DefaultValueStorage<#base_type>>::default_storage()
                })
            },
        }
    }

    fn try_into_source_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        access: TokenStream,
        error_type: &syn::Ident,
    ) -> ValueHolderResult<TokenStream> {
        let field_name = field.field_name();
        match self {
            HolderStoragePlan::OriginallyOptional => {
                if needs_into_conversion(field) {
                    let converted = apply_into_conversion(field, quote! { value });
                    Ok(quote! {
                        #access.map(|value| #converted)
                    })
                } else {
                    Ok(access)
                }
            },
            HolderStoragePlan::RequiredValue => {
                let field_name_str = field_name.to_string();
                if needs_into_conversion(field) {
                    let converted = apply_into_conversion(field, quote! { value });
                    Ok(quote! {
                        {
                            let value = #access.ok_or(#error_type{
                                field_name: #field_name_str
                            })?;
                            #converted
                        }
                    })
                } else {
                    Ok(quote! {
                        #access.ok_or(#error_type{
                            field_name: #field_name_str
                        })?
                    })
                }
            },
            HolderStoragePlan::Direct => {
                let converted = apply_into_conversion(field, access);
                Ok(quote! {
                    #converted
                })
            },
            HolderStoragePlan::ShapePolicy { .. } => {
                let base_type = form_base_type(field);
                let field_name_str = field_name.to_string();
                let converted = apply_into_conversion(field, quote! { value });
                let policy = shape_policy_type_tokens(context, field)?;
                let runtime_crate = context.paths.gpui_form_facade_runtime();
                Ok(quote! {
                    <#policy
                        as #runtime_crate::shape::ValueStorage<#base_type>>::try_map_into_value(
                            #access,
                            |value| #converted,
                            #error_type {
                                field_name: #field_name_str
                            }
                        )?
                })
            },
        }
    }

    fn from_source_storage_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        access: TokenStream,
    ) -> ValueHolderResult<TokenStream> {
        match self {
            HolderStoragePlan::OriginallyOptional => {
                if needs_from_conversion(field) {
                    let converted = apply_from_conversion(field, quote! { value });
                    Ok(quote! {
                        #access.map(|value| #converted)
                    })
                } else {
                    Ok(access)
                }
            },
            HolderStoragePlan::RequiredValue => {
                if let Some(default_expr) = field.default_expr() {
                    let default_span = field.default_expr_span();
                    let default_original = default_expr_for_original(default_expr, default_span);
                    let original_type = field.original_type();
                    if needs_from_conversion(field) {
                        let converted = apply_from_conversion(field, quote! { value });
                        Ok(quote! {
                            {
                                let value = #access;
                                let default_original: #original_type = #default_original;
                                if value == default_original {
                                    None
                                } else {
                                    Some(#converted)
                                }
                            }
                        })
                    } else {
                        Ok(quote! {
                            {
                                let value = #access;
                                let default_original: #original_type = #default_original;
                                if value == default_original {
                                    None
                                } else {
                                    Some(value)
                                }
                            }
                        })
                    }
                } else if needs_from_conversion(field) {
                    let converted = apply_from_conversion(field, quote! { value });
                    Ok(quote! {
                        {
                            let value = #access;
                            Some(#converted)
                        }
                    })
                } else {
                    Ok(quote! {
                        Some(#access)
                    })
                }
            },
            HolderStoragePlan::Direct => {
                let converted = apply_from_conversion(field, access);
                Ok(quote! {
                    #converted
                })
            },
            HolderStoragePlan::ShapePolicy { .. } => {
                let base_type = form_base_type(field);
                let policy = shape_policy_type_tokens(context, field)?;
                let runtime_crate = context.paths.gpui_form_facade_runtime();
                if let Some(default_expr) = field.default_expr() {
                    let default_span = field.default_expr_span();
                    let default_original = default_expr_for_original(default_expr, default_span);
                    let original_type = field.original_type();
                    let converted = apply_from_conversion(field, quote! { value });
                    let converted_default =
                        apply_from_conversion(field, quote! { default_original });
                    Ok(quote! {
                        {
                            let value = #access;
                            let default_original: #original_type = #default_original;
                            let value = #converted;
                            let default_value = #converted_default;
                            <#policy
                                as #runtime_crate::shape::ValueStorage<#base_type>>::present_unless_default(
                                    value,
                                    default_value
                                )
                        }
                    })
                } else {
                    let converted = apply_from_conversion(field, access);
                    Ok(quote! {
                        <#policy
                            as #runtime_crate::shape::ValueStorage<#base_type>>::present(#converted)
                    })
                }
            },
        }
    }

    fn into_source_infallible_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        access: TokenStream,
    ) -> ValueHolderResult<TokenStream> {
        match self {
            HolderStoragePlan::OriginallyOptional => {
                if needs_into_conversion(field) {
                    let converted = apply_into_conversion(field, quote! { value });
                    Ok(quote! {
                        #access.map(|value| #converted)
                    })
                } else {
                    Ok(access)
                }
            },
            HolderStoragePlan::RequiredValue => {
                if let Some(default_expr) = field.default_expr() {
                    let default_span = field.default_expr_span();
                    let default_original = default_expr_for_original(default_expr, default_span);
                    if needs_into_conversion(field) {
                        let converted = apply_into_conversion(field, quote! { value });
                        Ok(quote! {
                            #access
                                .map(|value| #converted)
                                .unwrap_or(#default_original)
                        })
                    } else {
                        Ok(quote! {
                            #access.unwrap_or(#default_original)
                        })
                    }
                } else if needs_into_conversion(field) {
                    let converted = apply_into_conversion(field, quote! { value });
                    Ok(quote! {
                        #access
                            .map(|value| #converted)
                            .unwrap_or_default()
                    })
                } else {
                    Ok(quote! {
                        #access.unwrap_or_default()
                    })
                }
            },
            HolderStoragePlan::Direct => {
                let converted = apply_into_conversion(field, access);
                Ok(quote! {
                    #converted
                })
            },
            HolderStoragePlan::ShapePolicy { .. } => {
                let base_type = form_base_type(field);
                let converted = apply_into_conversion(field, quote! { value });
                let policy = shape_policy_type_tokens(context, field)?;
                let runtime_crate = context.paths.gpui_form_facade_runtime();
                if let Some(default_expr) = field.default_expr() {
                    let default_span = field.default_expr_span();
                    let default_original = default_expr_for_original(default_expr, default_span);
                    Ok(quote! {
                        <#policy
                            as #runtime_crate::shape::ValueStorage<#base_type>>::map_into_value(
                                #access,
                                |value| #converted,
                                || #default_original
                            )
                    })
                } else {
                    let field_name_str = field.field_name().to_string();
                    Ok(quote! {
                        <#policy
                            as #runtime_crate::shape::ValueStorage<#base_type>>::map_into_value(
                                #access,
                                |value| #converted,
                                || panic!("Missing required value for field '{}'", #field_name_str)
                            )
                    })
                }
            },
        }
    }

    fn present_field_entry_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        present_field_ident: &syn::Ident,
        _present_lifetime: &syn::Lifetime,
    ) -> ValueHolderResult<TokenStream> {
        let field_name = field.field_name();
        let variant_ident = present_field_variant_ident(field_name);

        match self {
            HolderStoragePlan::OriginallyOptional => {
                if needs_into_conversion(field) {
                    let converted = apply_into_conversion(field, quote! { value });
                    Ok(quote! {
                        if let Some(value) = self.#field_name.clone() {
                            entries.push(#present_field_ident::#variant_ident(#converted));
                        }
                    })
                } else {
                    Ok(quote! {
                        if let Some(value) = self.#field_name.as_ref() {
                            entries.push(#present_field_ident::#variant_ident(value));
                        }
                    })
                }
            },
            HolderStoragePlan::RequiredValue => {
                if needs_into_conversion(field) {
                    let converted = apply_into_conversion(field, quote! { value });
                    Ok(quote! {
                        if let Some(value) = self.#field_name.clone() {
                            entries.push(#present_field_ident::#variant_ident(#converted));
                        }
                    })
                } else {
                    Ok(quote! {
                        if let Some(value) = self.#field_name.as_ref() {
                            entries.push(#present_field_ident::#variant_ident(value));
                        }
                    })
                }
            },
            HolderStoragePlan::Direct => {
                if needs_into_conversion(field) {
                    let converted = apply_into_conversion(field, quote! { value });
                    Ok(quote! {
                        let value = self.#field_name.clone();
                        entries.push(#present_field_ident::#variant_ident(#converted));
                    })
                } else {
                    Ok(quote! {
                        entries.push(#present_field_ident::#variant_ident(&self.#field_name));
                    })
                }
            },
            HolderStoragePlan::ShapePolicy { .. } => {
                let base_type = form_base_type(field);
                let converted = apply_into_conversion(field, quote! { value });
                let policy = shape_policy_type_tokens(context, field)?;
                let runtime_crate = context.paths.gpui_form_facade_runtime();
                Ok(quote! {
                    if let Some(value) =
                        <#policy
                            as #runtime_crate::shape::ValueStorage<#base_type>>::map_present_cloned(
                                &self.#field_name,
                                |value| #converted
                            )
                    {
                        entries.push(#present_field_ident::#variant_ident(value));
                    }
                })
            },
        }
    }

    fn required_validation_builder_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        validator_ident: &syn::Ident,
        target_ident: &syn::Ident,
    ) -> ValueHolderResult<Option<TokenStream>> {
        match self {
            HolderStoragePlan::RequiredValue => Ok(Some(quote! {
                koruma_collection::general::RequiredValidation::<Option<_>>::builder()
            })),
            HolderStoragePlan::ShapePolicy { .. } => {
                let base_type = form_base_type(field);
                let policy = shape_policy_type_tokens(context, field)?;
                Ok(Some(quote! {
                    #validator_ident::<
                        #target_ident<
                            #policy,
                            #base_type
                        >
                    >::builder()
                }))
            },
            HolderStoragePlan::OriginallyOptional | HolderStoragePlan::Direct => Ok(None),
        }
    }
}

fn shape_policy_type_tokens(
    context: &DeriveContext,
    field: &HolderFieldIr,
) -> ValueHolderResult<TokenStream> {
    field
        .storage()
        .shape_policy_type_tokens(context, field.field_name())
}

fn form_field_type_tokens(
    context: &DeriveContext,
    field: &HolderFieldIr,
) -> ValueHolderResult<TokenStream> {
    let base_type = form_base_type(field);
    field
        .storage()
        .storage_type_tokens(context, &base_type, field.field_name())
}

fn apply_from_conversion(field: &HolderFieldIr, value: TokenStream) -> TokenStream {
    if let Some(expr) = field.source_to_form_expr() {
        let span = field.source_to_form_span();
        quote_spanned! {span=> (#expr)(#value) }
    } else {
        value
    }
}

fn apply_into_conversion(field: &HolderFieldIr, value: TokenStream) -> TokenStream {
    if let Some(expr) = field.form_to_source_expr() {
        let span = field.form_to_source_span();
        quote_spanned! {span=> (#expr)(#value) }
    } else {
        value
    }
}

fn needs_from_conversion(field: &HolderFieldIr) -> bool {
    field.source_to_form_expr().is_some()
}

fn needs_into_conversion(field: &HolderFieldIr) -> bool {
    field.form_to_source_expr().is_some()
}

fn conversion_signature_assertions(
    fields: &[FieldPlan],
    wrapped_ident: &syn::Ident,
    original_impl_generics: TokenStream,
    where_clause: TokenStream,
) -> ValueHolderResult<TokenStream> {
    let mut assertions = Vec::new();

    for shared in fields.iter().filter_map(FieldPlan::shared) {
        let field_name = shared.field_name();
        let source_type = &shared.source_value_type;
        let form_type = &shared.form_type;

        if let Some(expr) = shared.source_to_form_expr() {
            let span = shared.source_to_form_span();
            let assert_ident =
                format_ident!("__{}_assert_{}_source_to_form", wrapped_ident, field_name);
            assertions.push(quote_spanned! {span=>
                #[allow(dead_code, non_snake_case)]
                fn #assert_ident #original_impl_generics() #where_clause {
                    fn assert_source_to_form<F>(_: F)
                    where
                        F: ::core::ops::FnOnce(#source_type) -> #form_type,
                    {}

                    assert_source_to_form(#expr);
                }
            });
        }

        if let Some(expr) = shared.form_to_source_expr() {
            let span = shared.form_to_source_span();
            let assert_ident =
                format_ident!("__{}_assert_{}_form_to_source", wrapped_ident, field_name);
            assertions.push(quote_spanned! {span=>
                #[allow(dead_code, non_snake_case)]
                fn #assert_ident #original_impl_generics() #where_clause {
                    fn assert_form_to_source<F>(_: F)
                    where
                        F: ::core::ops::FnOnce(#form_type) -> #source_type,
                    {}

                    assert_form_to_source(#expr);
                }
            });
        }
    }

    Ok(quote! {
        #(#assertions)*
    })
}

fn try_from_field_tokens(
    context: &DeriveContext,
    field: &HolderFieldIr,
    source: TokenStream,
    error_type: &syn::Ident,
) -> ValueHolderResult<TokenStream> {
    let field_name = field.field_name();
    let access = quote! { #source.#field_name };
    let value = field
        .storage()
        .try_into_source_tokens(context, field, access, error_type)?;
    Ok(quote! {
        #field_name: #value
    })
}

fn generate_conversion_error_type(error_name: &syn::Ident) -> TokenStream {
    quote! {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct #error_name {
            pub field_name: &'static str,
        }

        impl ::core::fmt::Display for #error_name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                write!(
                    f,
                    "Missing required value for field '{}'",
                    self.field_name
                )
            }
        }

        impl ::core::error::Error for #error_name {}
    }
}

/// Generates a custom Default implementation for the FormValueHolder that uses
/// the specified default expressions for fields that have them.
fn unwrap_expr(expr: &syn::Expr) -> &syn::Expr {
    match expr {
        syn::Expr::Group(group) => unwrap_expr(&group.expr),
        syn::Expr::Paren(paren) => unwrap_expr(&paren.expr),
        _ => expr,
    }
}

fn should_wrap_default_into(expr: &syn::Expr) -> bool {
    matches!(unwrap_expr(expr), syn::Expr::Lit(_))
}

fn default_expr_for_original(expr: &syn::Expr, span: Span) -> TokenStream {
    let expr_tokens = quote_spanned! {span=> #expr };
    if should_wrap_default_into(expr) {
        quote_spanned! {span=> ::core::convert::Into::into(#expr_tokens) }
    } else {
        expr_tokens
    }
}

fn generate_default_impl(
    context: &DeriveContext,
    fields: &[&HolderFieldIr],
    struct_name: &syn::Ident,
    impl_generics: TokenStream,
    ty_generics: TokenStream,
    where_clause: TokenStream,
) -> ValueHolderResult<TokenStream> {
    let default_fields: Vec<TokenStream> = fields
        .iter()
        .map(|f| -> ValueHolderResult<TokenStream> {
            let field_name = f.field_name();
            let base_type = form_base_type(f);
            if let Some(default_expr) = f.default_expr() {
                let default_span = f.default_expr_span();
                let default_original = default_expr_for_original(default_expr, default_span);
                let default_value = apply_from_conversion(f, default_original);
                let storage =
                    f.storage()
                        .present_tokens(context, default_value, &base_type, field_name)?;
                Ok(quote! {
                    #field_name: #storage
                })
            } else {
                let storage = f
                    .storage()
                    .default_storage_tokens(context, &base_type, field_name)?;
                Ok(quote! {
                    #field_name: #storage
                })
            }
        })
        .collect::<ValueHolderResult<Vec<_>>>()?;

    Ok(quote! {
        impl #impl_generics ::core::default::Default for #struct_name #ty_generics #where_clause {
            fn default() -> Self {
                Self {
                    #(#default_fields),*
                }
            }
        }
    })
}

fn add_field_trait_bounds(
    context: &DeriveContext,
    mut where_clause: Option<syn::WhereClause>,
    fields: &[&HolderFieldIr],
    trait_path: syn::Path,
) -> ValueHolderResult<Option<syn::WhereClause>> {
    for field in fields {
        let field_type = form_field_type_tokens(context, field)?;
        let wc = where_clause.get_or_insert_with(|| syn::parse_quote!(where));
        wc.predicates
            .push(syn::parse_quote!(#field_type: #trait_path));
    }

    Ok(where_clause)
}

fn generate_clone_impl(
    context: &DeriveContext,
    fields: &[&HolderFieldIr],
    struct_name: &syn::Ident,
    impl_generics: TokenStream,
    ty_generics: TokenStream,
    where_clause: Option<syn::WhereClause>,
) -> ValueHolderResult<TokenStream> {
    let clone_fields: Vec<TokenStream> = fields
        .iter()
        .map(|field| {
            let field_name = field.field_name();
            quote! {
                #field_name: self.#field_name.clone()
            }
        })
        .collect();
    let where_clause = add_field_trait_bounds(
        context,
        where_clause,
        fields,
        syn::parse_quote!(::core::clone::Clone),
    )?;

    Ok(quote! {
        impl #impl_generics ::core::clone::Clone for #struct_name #ty_generics #where_clause {
            fn clone(&self) -> Self {
                Self {
                    #(#clone_fields),*
                }
            }
        }
    })
}

fn generate_debug_impl(
    context: &DeriveContext,
    fields: &[&HolderFieldIr],
    struct_name: &syn::Ident,
    impl_generics: TokenStream,
    ty_generics: TokenStream,
    where_clause: Option<syn::WhereClause>,
) -> ValueHolderResult<TokenStream> {
    let struct_name_str = struct_name.to_string();
    let debug_fields: Vec<TokenStream> = fields
        .iter()
        .map(|field| {
            let field_name = field.field_name();
            let field_name_str = field_name.to_string();
            quote! {
                .field(#field_name_str, &self.#field_name)
            }
        })
        .collect();
    let where_clause = add_field_trait_bounds(
        context,
        where_clause,
        fields,
        syn::parse_quote!(::core::fmt::Debug),
    )?;

    Ok(quote! {
        impl #impl_generics ::core::fmt::Debug for #struct_name #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(#struct_name_str)
                    #(#debug_fields)*
                    .finish()
            }
        }
    })
}

fn generate_to_wrapped_field(
    context: &DeriveContext,
    field: &HolderFieldIr,
) -> ValueHolderResult<TokenStream> {
    let field_name = field.field_name();
    let value =
        field
            .storage()
            .from_source_storage_tokens(context, field, quote! { from.#field_name })?;
    Ok(quote! {
        #field_name: #value
    })
}

fn generate_from_wrapped_field(
    context: &DeriveContext,
    field: &HolderFieldIr,
) -> ValueHolderResult<TokenStream> {
    let field_name = field.field_name();
    let value = field.storage().into_source_infallible_tokens(
        context,
        field,
        quote! { from.#field_name },
    )?;
    Ok(quote! {
        #field_name: #value
    })
}

fn present_field_variant_ident(field_name: &syn::Ident) -> syn::Ident {
    let source = field_name.to_string();
    let source = source.strip_prefix("r#").unwrap_or(&source);
    let mut ident = String::new();
    let mut uppercase_next = true;

    for ch in source.chars() {
        if ch.is_ascii_alphanumeric() {
            if uppercase_next {
                ident.push(ch.to_ascii_uppercase());
                uppercase_next = false;
            } else {
                ident.push(ch);
            }
        } else {
            uppercase_next = true;
        }
    }

    if ident.is_empty() {
        ident.push_str("Field");
    }
    if ident.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        ident.insert_str(0, "Field");
    }

    format_ident!("{ident}")
}

fn present_field_variant_type_tokens<'a>(
    field: &'a HolderFieldIr,
    present_lifetime: &'a syn::Lifetime,
) -> TokenStream {
    let source_type = &field.source_value_type;

    if field.storage().is_shape_policy() || needs_into_conversion(field) {
        quote! { #source_type }
    } else {
        quote! { &#present_lifetime #source_type }
    }
}

fn generate_present_field_variant(
    field: &HolderFieldIr,
    present_lifetime: &syn::Lifetime,
) -> TokenStream {
    let variant_ident = present_field_variant_ident(field.field_name());
    let variant_type = present_field_variant_type_tokens(field, present_lifetime);

    quote! {
        #variant_ident(#variant_type)
    }
}

fn generate_present_field_entry(
    context: &DeriveContext,
    field: &HolderFieldIr,
    present_field_ident: &syn::Ident,
    present_lifetime: &syn::Lifetime,
) -> ValueHolderResult<TokenStream> {
    field.storage().present_field_entry_tokens(
        context,
        field,
        present_field_ident,
        present_lifetime,
    )
}

fn value_storage_policy_validator_tokens(
    context: &DeriveContext,
    validator_ident: &syn::Ident,
    builder_ident: &syn::Ident,
    target_ident: &syn::Ident,
    target_trait_ident: &syn::Ident,
    enable_koruma_fluent: bool,
) -> TokenStream {
    let runtime_crate = context.paths.gpui_form_facade_runtime();
    let fluent_impl = if enable_koruma_fluent {
        quote! {
            impl<Target> ::es_fluent::FluentMessage for #validator_ident<Target> {
                fn to_fluent_string_with(
                    &self,
                    _localize: &mut dyn for<'a> FnMut(
                        &str,
                        &str,
                        Option<&::std::collections::HashMap<&str, ::es_fluent::FluentValue<'a>>>,
                    ) -> String,
                ) -> String {
                    "This field is required.".to_string()
                }
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #[doc(hidden)]
        pub struct #target_ident<Policy, Value> {
            _marker: ::core::marker::PhantomData<fn() -> (Policy, Value)>,
        }

        #[doc(hidden)]
        pub trait #target_trait_ident {
            type Storage;

            fn is_present(value: &Self::Storage) -> bool;
        }

        impl<Policy, Value> #target_trait_ident for #target_ident<Policy, Value>
        where
            Policy: #runtime_crate::shape::ValueStorage<Value>,
        {
            type Storage =
                <Policy as #runtime_crate::shape::ValueStorage<Value>>::Storage;

            fn is_present(value: &Self::Storage) -> bool {
                <Policy as #runtime_crate::shape::ValueStorage<Value>>::is_present(value)
            }
        }

        #[doc(hidden)]
        pub struct #validator_ident<Target> {
            _marker: ::core::marker::PhantomData<fn() -> Target>,
        }

        impl<Target> ::core::clone::Clone for #validator_ident<Target> {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<Target> ::core::marker::Copy for #validator_ident<Target> {}

        impl<Target> ::core::fmt::Debug for #validator_ident<Target> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(stringify!(#validator_ident)).finish()
            }
        }

        impl<Target> #validator_ident<Target> {
            pub fn builder() -> #builder_ident<Target> {
                #builder_ident {
                    _marker: ::core::marker::PhantomData,
                }
            }
        }

        impl<Target> #validator_ident<Target>
        where
            Target: #target_trait_ident,
        {
            pub fn validate(&self, value: &<Target as #target_trait_ident>::Storage) -> bool {
                <Target as #target_trait_ident>::is_present(value)
            }
        }

        #[doc(hidden)]
        pub struct #builder_ident<Target> {
            _marker: ::core::marker::PhantomData<fn() -> Target>,
        }

        impl<Target> ::core::clone::Clone for #builder_ident<Target> {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<Target> ::core::marker::Copy for #builder_ident<Target> {}

        impl<Target> ::core::fmt::Debug for #builder_ident<Target> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(stringify!(#builder_ident)).finish()
            }
        }

        impl<Target> #builder_ident<Target> {
            pub fn build(self) -> #validator_ident<Target> {
                #validator_ident {
                    _marker: ::core::marker::PhantomData,
                }
            }
        }

        impl<Target> ::koruma::BuilderWithValueRef<<Target as #target_trait_ident>::Storage>
            for #builder_ident<Target>
        where
            Target: #target_trait_ident,
        {
            type Output = Self;

            fn with_value_ref(
                self,
                _value: &<Target as #target_trait_ident>::Storage,
            ) -> Self::Output {
                self
            }
        }

        impl<Target> ::koruma::Validate<<Target as #target_trait_ident>::Storage>
            for #validator_ident<Target>
        where
            Target: #target_trait_ident,
        {
            fn validate(
                &self,
                value: &<Target as #target_trait_ident>::Storage,
            ) -> bool {
                self.validate(value)
            }
        }

        #fluent_impl
    }
}

fn required_validation_builder_tokens(
    context: &DeriveContext,
    field: &HolderFieldIr,
    validator_ident: &syn::Ident,
    target_ident: &syn::Ident,
) -> ValueHolderResult<Option<TokenStream>> {
    if !field.needs_required_validation() {
        return Ok(None);
    }

    field.storage().required_validation_builder_tokens(
        context,
        field,
        validator_ident,
        target_ident,
    )
}

/// Generates the FormValueHolder struct and its implementations.
pub(super) fn generate_value_holder(
    context: &DeriveContext,
    original_input: &DeriveInput,
    fields: &[FieldPlan],
    conversion_plan: &HolderConversionPlan,
    enable_koruma: bool,
    enable_koruma_fluent: bool,
) -> ValueHolderResult<TokenStream> {
    let has_skipped_fields = fields.iter().any(|f| f.is_skipped());
    let rendered_fields: Vec<&HolderFieldIr> =
        fields.iter().filter_map(FieldPlan::shared).collect();
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
    let has_any_required = rendered_fields
        .iter()
        .any(|f| f.needs_required_validation());
    let has_shape_policy_required = rendered_fields.iter().any(|shared| {
        !shared.was_optional && shared.storage.is_shape_policy() && !shared.validation.is_nested
    });

    let mut field_attrs: HashMap<String, Vec<TokenStream>> = HashMap::new();

    for f in &rendered_fields {
        let field_name = f.field_name().to_string();

        if enable_koruma {
            let required_validation = required_validation_builder_tokens(
                context,
                f,
                &required_policy_validator_ident,
                &required_policy_validator_target_ident,
            )?;

            let validation = f.validation();
            let has_existing_validations = !validation.field_validators.is_empty()
                || !validation.element_validators.is_empty();
            let has_newtype = validation.is_newtype;

            if required_validation.is_some() || has_existing_validations || has_newtype {
                let mut koruma_items: Vec<TokenStream> = Vec::new();

                if let Some(required_validation) = required_validation {
                    koruma_items.push(required_validation);
                }

                let existing_validations: Vec<TokenStream> = validation
                    .field_validators
                    .iter()
                    .chain(validation.element_validators.iter())
                    .map(validator_attr_to_tokens)
                    .collect();
                koruma_items.extend(existing_validations);

                // Build koruma attributes - newtype/nested must be separate attributes
                // when combined with other validators to avoid type resolution issues
                let mut attrs: Vec<TokenStream> = Vec::new();

                // Add validators first (if any)
                if !koruma_items.is_empty() {
                    attrs.push(quote! { #[koruma(#(#koruma_items),*)] });
                }

                // Add newtype/nested as separate attributes
                if validation.is_newtype {
                    attrs.insert(0, quote! { #[koruma(newtype)] });
                }
                if validation.is_nested {
                    attrs.insert(0, quote! { #[koruma(nested)] });
                }

                if !attrs.is_empty() {
                    field_attrs.insert(field_name, attrs);
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
    let conversion_signature_assertions = conversion_signature_assertions(
        fields,
        &wrapped_ident,
        quote! { #original_impl_generics },
        quote! { #where_clause },
    )?;
    let holder_declaration_generics = original_input.generics.clone();
    let holder_impl_generics = original_input.generics.clone();
    let mut holder_where_clause = where_clause.cloned();
    for f in &rendered_fields {
        if f.was_optional {
            continue;
        }

        if f.storage().is_shape_policy() {
            let base_type = form_base_type(f);
            let policy = shape_policy_type_tokens(context, f)?;
            let runtime_crate = context.paths.gpui_form_facade_runtime();
            let validation = f.validation();
            let wc = holder_where_clause.get_or_insert_with(|| syn::parse_quote!(where));
            if needs_koruma_derive
                && (!validation.field_validators.is_empty()
                    || !validation.element_validators.is_empty()
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
    for f in &rendered_fields {
        if !f.was_optional && f.default_expr().is_none() {
            let Some(shape) = f.storage().shape_policy() else {
                continue;
            };
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
                <#shape as #runtime_crate::shape::ComponentShape>::ValueStoragePolicy:
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
    let mut present_field_generics = original_input.generics.clone();
    present_field_generics.params.insert(
        0,
        syn::GenericParam::Lifetime(syn::parse_quote!('__gpui_form_present)),
    );
    let (_, present_field_ty_generics, _) = present_field_generics.split_for_impl();
    let present_field_variants: Vec<TokenStream> = rendered_fields
        .iter()
        .map(|f| generate_present_field_variant(f, &present_lifetime))
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

    let from_where_clause = holder_where_clause.clone();
    let skipped_params: Vec<TokenStream> = fields
        .iter()
        .filter(|f| f.is_skipped())
        .map(|f| {
            let field_name = f.field_name();
            let ty = f.original_type();
            quote! { #field_name: #ty }
        })
        .collect();

    let into_original_fields: Vec<TokenStream> = fields
        .iter()
        .map(|f| -> ValueHolderResult<TokenStream> {
            if f.is_skipped() {
                let field_name = f.field_name();
                Ok(quote! { #field_name })
            } else {
                let shared = f.shared().expect("non-skipped fields carry shared plans");
                try_from_field_tokens(context, shared, quote! { self }, &conversion_error_ident)
            }
        })
        .collect::<ValueHolderResult<Vec<_>>>()?;

    let skipped_fields_impl = match conversion_plan.mode() {
        HolderConversionMode::SkippedFields => {
            quote! {
                #present_field_enum

                impl #holder_impl_generics #storage_ident #holder_ty_generics #holder_where_clause {
                    pub fn present_fields<'__gpui_form_present>(
                        &'__gpui_form_present self
                    ) -> Vec<#present_field_ident #present_field_ty_generics> {
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

    let mut tokens = quote! {
        #conversion_error_type
        #required_policy_validator
        #conversion_signature_assertions
        #(#default_storage_assertions)*
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
    for f in &rendered_fields {
        if f.was_optional || f.default_expr().is_some() {
            continue;
        }

        if f.storage().is_shape_policy() {
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
