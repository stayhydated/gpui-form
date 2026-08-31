use super::storage::{
    HolderStorageStrategy as _, apply_from_conversion, form_base_type, form_field_type_tokens,
};
use super::*;

/// Generates a custom Default implementation for the FormValueHolder that uses
/// the specified default expressions for fields that have them.
pub(super) fn generate_default_impl(
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
                let default_original = source_default_tokens(default_expr, default_span);
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

pub(super) fn generate_clone_impl(
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

pub(super) fn generate_debug_impl(
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

pub(super) fn generate_to_wrapped_field(
    context: &DeriveContext,
    field: &HolderFieldIr,
) -> ValueHolderResult<TokenStream> {
    let field_name = field.field_name();
    let value =
        field
            .storage()
            .source_to_storage_tokens(context, field, quote! { from.#field_name })?;
    Ok(quote! {
        #field_name: #value
    })
}

pub(super) fn generate_from_wrapped_field(
    context: &DeriveContext,
    field: &HolderFieldIr,
) -> ValueHolderResult<TokenStream> {
    let field_name = field.field_name();
    let value = field.storage().storage_to_source_infallible_tokens(
        context,
        field,
        quote! { from.#field_name },
    )?;
    Ok(quote! {
        #field_name: #value
    })
}
