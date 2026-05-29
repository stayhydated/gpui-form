use gpui_form_codegen::{CratePaths, components::RequiredValue};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{DeriveInput, Type};

use crate::derives::gpui_form::koruma::validator_attr_to_tokens;
use crate::derives::gpui_form::structs::{ComponentField, FieldOptionality};
use crate::derives::gpui_form::utils::extract_option_inner_type;

#[derive(Clone, Debug)]
enum FieldStorage {
    OriginallyOptional,
    RequiredValue,
    Plain,
    ShapePolicy(syn::Path),
}

fn field_storage(field: &FieldOptionality) -> FieldStorage {
    if field.was_optional {
        FieldStorage::OriginallyOptional
    } else {
        match &field.required_value {
            RequiredValue::Explicit(true) => FieldStorage::RequiredValue,
            RequiredValue::Explicit(false) => FieldStorage::Plain,
            RequiredValue::Shape(shape) => FieldStorage::ShapePolicy(shape.clone()),
        }
    }
}

fn form_base_type(field: &FieldOptionality) -> Type {
    if let Some(override_type) = &field.override_type {
        extract_option_inner_type(override_type).1
    } else {
        field.inner_type.clone()
    }
}

fn shape_policy_storage_type_tokens(shape: &syn::Path, base_type: &Type) -> TokenStream {
    let runtime_crate = runtime_crate_path();
    quote! {
        <<#shape as #runtime_crate::shape::ComponentShape>::RequiredValuePolicy
            as #runtime_crate::shape::ValueHolderStorage<#base_type>>::Storage
    }
}

fn facade_crate_path() -> syn::Path {
    CratePaths::resolve().gpui_form
}

fn runtime_crate_path() -> syn::Path {
    let crate_paths = CratePaths::resolve();
    crate_paths.gpui_form_facade_runtime()
}

fn form_field_type_tokens(field: &FieldOptionality) -> TokenStream {
    let base_type = form_base_type(field);
    match field_storage(field) {
        FieldStorage::OriginallyOptional | FieldStorage::RequiredValue => {
            quote! { Option<#base_type> }
        },
        FieldStorage::Plain => quote! { #base_type },
        FieldStorage::ShapePolicy(shape) => shape_policy_storage_type_tokens(&shape, &base_type),
    }
}

fn apply_from_conversion(field: &FieldOptionality, value: TokenStream) -> TokenStream {
    if let Some(expr) = &field.from_expr {
        quote! { (#expr)(#value) }
    } else if field.override_type.is_some() {
        quote! { ::core::convert::From::from(#value) }
    } else {
        value
    }
}

fn apply_into_conversion(field: &FieldOptionality, value: TokenStream) -> TokenStream {
    if let Some(expr) = &field.into_expr {
        quote! { (#expr)(#value) }
    } else if field.override_type.is_some() {
        quote! { ::core::convert::From::from(#value) }
    } else {
        value
    }
}

fn needs_from_conversion(field: &FieldOptionality) -> bool {
    field.from_expr.is_some() || field.override_type.is_some()
}

fn needs_into_conversion(field: &FieldOptionality) -> bool {
    field.into_expr.is_some() || field.override_type.is_some()
}

fn field_conversion_can_fail(field: &FieldOptionality) -> bool {
    if field.default_expr.is_some() {
        return false;
    }

    matches!(
        field_storage(field),
        FieldStorage::RequiredValue | FieldStorage::ShapePolicy(_)
    )
}

pub(super) fn holder_conversion_can_fail(fields: &[FieldOptionality]) -> bool {
    fields
        .iter()
        .filter(|field| !field.skip)
        .any(field_conversion_can_fail)
}

fn try_from_field_tokens(
    field: &FieldOptionality,
    source: TokenStream,
    error_type: &syn::Ident,
) -> TokenStream {
    let field_name = &field.field_name;
    let access = quote! { #source.#field_name };
    match field_storage(field) {
        FieldStorage::OriginallyOptional => {
            if needs_into_conversion(field) {
                let converted = apply_into_conversion(field, quote! { value });
                quote! {
                    #field_name: #access.map(|value| #converted)
                }
            } else {
                quote! {
                    #field_name: #access
                }
            }
        },
        FieldStorage::RequiredValue => {
            let field_name_str = field_name.to_string();
            if needs_into_conversion(field) {
                let converted = apply_into_conversion(field, quote! { value });
                quote! {
                    #field_name: {
                        let value = #access.ok_or(#error_type{
                            field_name: #field_name_str
                        })?;
                        #converted
                    }
                }
            } else {
                quote! {
                    #field_name: #access.ok_or(#error_type{
                        field_name: #field_name_str
                    })?
                }
            }
        },
        FieldStorage::Plain => {
            let converted = apply_into_conversion(field, access);
            quote! {
                #field_name: #converted
            }
        },
        FieldStorage::ShapePolicy(shape) => {
            let base_type = form_base_type(field);
            let field_name_str = field_name.to_string();
            let converted = apply_into_conversion(field, quote! { value });
            let runtime_crate = runtime_crate_path();
            quote! {
                #field_name: <<#shape as #runtime_crate::shape::ComponentShape>::RequiredValuePolicy
                    as #runtime_crate::shape::ValueHolderStorage<#base_type>>::try_map_into_value(
                        #access,
                        |value| #converted,
                        #error_type {
                            field_name: #field_name_str
                        }
                    )?
            }
        },
    }
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

fn default_expr_for_original(expr: &syn::Expr) -> TokenStream {
    let expr_tokens = quote! { #expr };
    if should_wrap_default_into(expr) {
        quote! { ::core::convert::Into::into(#expr_tokens) }
    } else {
        expr_tokens
    }
}

fn generate_default_impl(
    fields: &[FieldOptionality],
    struct_name: &syn::Ident,
    impl_generics: TokenStream,
    ty_generics: TokenStream,
    where_clause: TokenStream,
) -> TokenStream {
    let default_fields: Vec<TokenStream> = fields
        .iter()
        .filter(|f| !f.skip)
        .map(|f| {
            let field_name = &f.field_name;
            let base_type = form_base_type(f);
            if let Some(default_expr) = &f.default_expr {
                let default_original = default_expr_for_original(default_expr);
                let default_value = apply_from_conversion(f, default_original);
                match field_storage(f) {
                    FieldStorage::OriginallyOptional | FieldStorage::RequiredValue => {
                        quote! {
                            #field_name: Some(#default_value)
                        }
                    },
                    FieldStorage::Plain => {
                        quote! {
                            #field_name: #default_value
                        }
                    },
                    FieldStorage::ShapePolicy(shape) => {
                        let runtime_crate = runtime_crate_path();
                        quote! {
                            #field_name: <<#shape as #runtime_crate::shape::ComponentShape>::RequiredValuePolicy
                                as #runtime_crate::shape::ValueHolderStorage<#base_type>>::present(#default_value)
                        }
                    },
                }
            } else {
                match field_storage(f) {
                    FieldStorage::OriginallyOptional | FieldStorage::RequiredValue => {
                        quote! {
                            #field_name: None
                        }
                    },
                    FieldStorage::Plain => {
                        quote! {
                            #field_name: ::core::default::Default::default()
                        }
                    },
                    FieldStorage::ShapePolicy(shape) => {
                        let runtime_crate = runtime_crate_path();
                        quote! {
                            #field_name: <<#shape as #runtime_crate::shape::ComponentShape>::RequiredValuePolicy
                                as #runtime_crate::shape::ValueHolderStorage<#base_type>>::default_storage()
                        }
                    },
                }
            }
        })
        .collect();

    quote! {
        impl #impl_generics ::core::default::Default for #struct_name #ty_generics #where_clause {
            fn default() -> Self {
                Self {
                    #(#default_fields),*
                }
            }
        }
    }
}

fn add_field_trait_bounds(
    mut where_clause: Option<syn::WhereClause>,
    fields: &[FieldOptionality],
    trait_path: syn::Path,
) -> Option<syn::WhereClause> {
    for field in fields {
        if field.skip {
            continue;
        }

        let field_type = form_field_type_tokens(field);
        let wc = where_clause.get_or_insert_with(|| syn::parse_quote!(where));
        wc.predicates
            .push(syn::parse_quote!(#field_type: #trait_path));
    }

    where_clause
}

fn generate_clone_impl(
    fields: &[FieldOptionality],
    struct_name: &syn::Ident,
    impl_generics: TokenStream,
    ty_generics: TokenStream,
    where_clause: Option<syn::WhereClause>,
) -> TokenStream {
    let clone_fields: Vec<TokenStream> = fields
        .iter()
        .filter(|field| !field.skip)
        .map(|field| {
            let field_name = &field.field_name;
            quote! {
                #field_name: self.#field_name.clone()
            }
        })
        .collect();
    let where_clause = add_field_trait_bounds(
        where_clause,
        fields,
        syn::parse_quote!(::core::clone::Clone),
    );

    quote! {
        impl #impl_generics ::core::clone::Clone for #struct_name #ty_generics #where_clause {
            fn clone(&self) -> Self {
                Self {
                    #(#clone_fields),*
                }
            }
        }
    }
}

fn generate_debug_impl(
    fields: &[FieldOptionality],
    struct_name: &syn::Ident,
    impl_generics: TokenStream,
    ty_generics: TokenStream,
    where_clause: Option<syn::WhereClause>,
) -> TokenStream {
    let struct_name_str = struct_name.to_string();
    let debug_fields: Vec<TokenStream> = fields
        .iter()
        .filter(|field| !field.skip)
        .map(|field| {
            let field_name = &field.field_name;
            let field_name_str = field_name.to_string();
            quote! {
                .field(#field_name_str, &self.#field_name)
            }
        })
        .collect();
    let where_clause =
        add_field_trait_bounds(where_clause, fields, syn::parse_quote!(::core::fmt::Debug));

    quote! {
        impl #impl_generics ::core::fmt::Debug for #struct_name #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(#struct_name_str)
                    #(#debug_fields)*
                    .finish()
            }
        }
    }
}

fn generate_to_wrapped_field(field: &FieldOptionality) -> TokenStream {
    let field_name = &field.field_name;

    match field_storage(field) {
        FieldStorage::OriginallyOptional => {
            if needs_from_conversion(field) {
                let converted = apply_from_conversion(field, quote! { value });
                quote! {
                    #field_name: from.#field_name.map(|value| #converted)
                }
            } else {
                quote! {
                    #field_name: from.#field_name
                }
            }
        },
        FieldStorage::RequiredValue => {
            if let Some(default_expr) = &field.default_expr {
                let default_original = default_expr_for_original(default_expr);
                let original_type = &field.original_type;
                if needs_from_conversion(field) {
                    let converted = apply_from_conversion(field, quote! { value });
                    quote! {
                        #field_name: {
                            let value = from.#field_name;
                            let default_original: #original_type = #default_original;
                            if value == default_original {
                                None
                            } else {
                                Some(#converted)
                            }
                        }
                    }
                } else {
                    quote! {
                        #field_name: {
                            let value = from.#field_name;
                            let default_original: #original_type = #default_original;
                            if value == default_original {
                                None
                            } else {
                                Some(value)
                            }
                        }
                    }
                }
            } else if needs_from_conversion(field) {
                let converted = apply_from_conversion(field, quote! { value });
                quote! {
                    #field_name: {
                        let value = from.#field_name;
                        Some(#converted)
                    }
                }
            } else {
                quote! {
                    #field_name: Some(from.#field_name)
                }
            }
        },
        FieldStorage::Plain => {
            let converted = apply_from_conversion(field, quote! { from.#field_name });
            quote! {
                #field_name: #converted
            }
        },
        FieldStorage::ShapePolicy(shape) => {
            let base_type = form_base_type(field);
            let runtime_crate = runtime_crate_path();
            if let Some(default_expr) = &field.default_expr {
                let default_original = default_expr_for_original(default_expr);
                let original_type = &field.original_type;
                let converted = apply_from_conversion(field, quote! { value });
                let converted_default = apply_from_conversion(field, quote! { default_original });
                quote! {
                    #field_name: {
                        let value = from.#field_name;
                        let default_original: #original_type = #default_original;
                        let value = #converted;
                        let default_value = #converted_default;
                        <<#shape as #runtime_crate::shape::ComponentShape>::RequiredValuePolicy
                            as #runtime_crate::shape::ValueHolderStorage<#base_type>>::present_unless_default(
                                value,
                                default_value
                            )
                    }
                }
            } else {
                let converted = apply_from_conversion(field, quote! { from.#field_name });
                quote! {
                    #field_name: <<#shape as #runtime_crate::shape::ComponentShape>::RequiredValuePolicy
                        as #runtime_crate::shape::ValueHolderStorage<#base_type>>::present(#converted)
                }
            }
        },
    }
}

fn generate_from_wrapped_field(field: &FieldOptionality) -> TokenStream {
    let field_name = &field.field_name;

    match field_storage(field) {
        FieldStorage::OriginallyOptional => {
            if needs_into_conversion(field) {
                let converted = apply_into_conversion(field, quote! { value });
                quote! {
                    #field_name: from.#field_name.map(|value| #converted)
                }
            } else {
                quote! {
                    #field_name: from.#field_name
                }
            }
        },
        FieldStorage::RequiredValue => {
            if let Some(default_expr) = &field.default_expr {
                let default_original = default_expr_for_original(default_expr);
                if needs_into_conversion(field) {
                    let converted = apply_into_conversion(field, quote! { value });
                    quote! {
                        #field_name: from.#field_name
                            .map(|value| #converted)
                            .unwrap_or(#default_original)
                    }
                } else {
                    quote! {
                        #field_name: from.#field_name.unwrap_or(#default_original)
                    }
                }
            } else if needs_into_conversion(field) {
                let converted = apply_into_conversion(field, quote! { value });
                quote! {
                    #field_name: from.#field_name
                        .map(|value| #converted)
                        .unwrap_or_default()
                }
            } else {
                quote! {
                    #field_name: from.#field_name.unwrap_or_default()
                }
            }
        },
        FieldStorage::Plain => {
            let converted = apply_into_conversion(field, quote! { from.#field_name });
            quote! {
                #field_name: #converted
            }
        },
        FieldStorage::ShapePolicy(shape) => {
            let base_type = form_base_type(field);
            let converted = apply_into_conversion(field, quote! { value });
            let runtime_crate = runtime_crate_path();
            if let Some(default_expr) = &field.default_expr {
                let default_original = default_expr_for_original(default_expr);
                quote! {
                    #field_name: <<#shape as #runtime_crate::shape::ComponentShape>::RequiredValuePolicy
                        as #runtime_crate::shape::ValueHolderStorage<#base_type>>::map_into_value(
                            from.#field_name,
                            |value| #converted,
                            || #default_original
                        )
                }
            } else {
                let field_name_str = field_name.to_string();
                quote! {
                    #field_name: <<#shape as #runtime_crate::shape::ComponentShape>::RequiredValuePolicy
                        as #runtime_crate::shape::ValueHolderStorage<#base_type>>::map_into_value(
                            from.#field_name,
                            |value| #converted,
                            || panic!("Missing required value for field '{}'", #field_name_str)
                        )
                }
            }
        },
    }
}

fn generate_present_fields_json_entry(field: &FieldOptionality) -> TokenStream {
    let field_name = &field.field_name;
    let field_name_str = field_name.to_string();

    match field_storage(field) {
        FieldStorage::OriginallyOptional => {
            if needs_into_conversion(field) {
                let converted = apply_into_conversion(field, quote! { value });
                quote! {
                    let converted = self.#field_name.clone().map(|value| #converted);
                    if converted.is_some() {
                        entries.push(format!(
                            "\"{}\":\"{}\"",
                            #field_name_str,
                            Self::escape_json_string(&format!("{:?}", converted))
                        ));
                    }
                }
            } else {
                quote! {
                    if self.#field_name.is_some() {
                        entries.push(format!(
                            "\"{}\":\"{}\"",
                            #field_name_str,
                            Self::escape_json_string(&format!("{:?}", &self.#field_name))
                        ));
                    }
                }
            }
        },
        FieldStorage::RequiredValue => {
            if needs_into_conversion(field) {
                let converted = apply_into_conversion(field, quote! { value });
                quote! {
                    if let Some(value) = self.#field_name.clone() {
                        let converted = #converted;
                        entries.push(format!(
                            "\"{}\":\"{}\"",
                            #field_name_str,
                            Self::escape_json_string(&format!("{:?}", converted))
                        ));
                    }
                }
            } else {
                quote! {
                    if let Some(value) = self.#field_name.as_ref() {
                        entries.push(format!(
                            "\"{}\":\"{}\"",
                            #field_name_str,
                            Self::escape_json_string(&format!("{:?}", value))
                        ));
                    }
                }
            }
        },
        FieldStorage::Plain => {
            if needs_into_conversion(field) {
                let converted = apply_into_conversion(field, quote! { value });
                quote! {
                    let value = self.#field_name.clone();
                    let converted = #converted;
                    entries.push(format!(
                        "\"{}\":\"{}\"",
                        #field_name_str,
                        Self::escape_json_string(&format!("{:?}", converted))
                    ));
                }
            } else {
                quote! {
                    entries.push(format!(
                        "\"{}\":\"{}\"",
                        #field_name_str,
                        Self::escape_json_string(&format!("{:?}", &self.#field_name))
                    ));
                }
            }
        },
        FieldStorage::ShapePolicy(shape) => {
            let base_type = form_base_type(field);
            let converted = apply_into_conversion(field, quote! { value });
            let runtime_crate = runtime_crate_path();
            quote! {
                if let Some(value) =
                    <<#shape as #runtime_crate::shape::ComponentShape>::RequiredValuePolicy
                        as #runtime_crate::shape::ValueHolderStorage<#base_type>>::map_present_cloned(
                            &self.#field_name,
                            |value| #converted
                        )
                {
                    entries.push(format!(
                        "\"{}\":\"{}\"",
                        #field_name_str,
                        Self::escape_json_string(&format!("{:?}", value))
                    ));
                }
            }
        },
    }
}

fn required_value_policy_validator_tokens(
    validator_ident: &syn::Ident,
    builder_ident: &syn::Ident,
    target_ident: &syn::Ident,
    target_trait_ident: &syn::Ident,
    enable_koruma_fluent: bool,
) -> TokenStream {
    let runtime_crate = runtime_crate_path();
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
            Policy: #runtime_crate::shape::ValueHolderStorage<Value>,
        {
            type Storage =
                <Policy as #runtime_crate::shape::ValueHolderStorage<Value>>::Storage;

            fn is_present(value: &Self::Storage) -> bool {
                <Policy as #runtime_crate::shape::ValueHolderStorage<Value>>::is_present(value)
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
    field: &FieldOptionality,
    validator_ident: &syn::Ident,
    target_ident: &syn::Ident,
) -> Option<TokenStream> {
    if !field.needs_required_validation() {
        return None;
    }

    match field_storage(field) {
        FieldStorage::RequiredValue => Some(quote! {
            koruma_collection::general::RequiredValidation::<Option<_>>::builder()
        }),
        FieldStorage::ShapePolicy(shape) => {
            let base_type = form_base_type(field);
            let runtime_crate = runtime_crate_path();
            Some(quote! {
                #validator_ident::<
                    #target_ident<
                        <#shape as #runtime_crate::shape::ComponentShape>::RequiredValuePolicy,
                        #base_type
                    >
                >::builder()
            })
        },
        FieldStorage::OriginallyOptional | FieldStorage::Plain => None,
    }
}

pub fn parse_field_default(field: &ComponentField) -> Option<syn::Expr> {
    field
        .rendered()
        .and_then(|rendered| rendered.default.map(|expr| expr.0.clone()))
}

/// Generates the FormValueHolder struct and its implementations.
pub fn generate_value_holder(
    original_input: &DeriveInput,
    fields: &[FieldOptionality],
    enable_koruma: bool,
    enable_koruma_fluent: bool,
) -> (TokenStream, Vec<String>) {
    let has_skipped_fields = fields.iter().any(|f| f.skip);
    let original_ident = &original_input.ident;
    let wrapped_ident = format_ident!("{}FormValueHolder", original_ident);
    let conversion_error_ident = format_ident!("{}ConversionError", wrapped_ident);
    let required_policy_validator_ident =
        format_ident!("__{}RequiredValuePolicyValidation", wrapped_ident);
    let required_policy_validator_builder_ident =
        format_ident!("__{}RequiredValuePolicyValidationBuilder", wrapped_ident);
    let required_policy_validator_target_ident =
        format_ident!("__{}RequiredValuePolicyValidationTarget", wrapped_ident);
    let required_policy_validator_target_trait_ident = format_ident!(
        "__{}RequiredValuePolicyValidationTargetTrait",
        wrapped_ident
    );
    let fields_requiring_required: Vec<String> = fields
        .iter()
        .filter(|f| f.needs_required_validation())
        .map(|f| f.field_name.to_string())
        .collect();

    let has_any_required = fields.iter().any(|f| f.needs_required_validation());
    let has_shape_policy_required = fields.iter().any(|f| {
        !f.skip
            && !f.was_optional
            && matches!(f.required_value, RequiredValue::Shape(_))
            && !f.validation.is_nested
    });

    let mut field_attrs: HashMap<String, Vec<TokenStream>> = HashMap::new();

    for f in fields {
        if f.skip {
            continue;
        }
        let field_name = f.field_name.to_string();

        if enable_koruma {
            let required_validation = required_validation_builder_tokens(
                f,
                &required_policy_validator_ident,
                &required_policy_validator_target_ident,
            );

            let has_existing_validations = !f.validation.field_validators.is_empty()
                || !f.validation.element_validators.is_empty();
            let has_newtype = f.validation.is_newtype;

            if required_validation.is_some() || has_existing_validations || has_newtype {
                let mut koruma_items: Vec<TokenStream> = Vec::new();

                if let Some(required_validation) = required_validation {
                    koruma_items.push(required_validation);
                }

                let existing_validations: Vec<TokenStream> = f
                    .validation
                    .field_validators
                    .iter()
                    .chain(f.validation.element_validators.iter())
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
                if f.validation.is_newtype {
                    attrs.insert(0, quote! { #[koruma(newtype)] });
                }
                if f.validation.is_nested {
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

    let reverse_conversion_can_fail = holder_conversion_can_fail(fields);

    let mut derives: Vec<TokenStream> = Vec::new();
    if has_skipped_fields {
        let facade_crate = facade_crate_path();
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
        let facade_crate = facade_crate_path();
        quote! { #[builder(crate = #facade_crate::bon)] }
    } else {
        quote! {}
    };

    let conversion_error_type = generate_conversion_error_type(&conversion_error_ident);
    let (impl_generics, ty_generics, where_clause) = original_input.generics.split_for_impl();
    let mut holder_where_clause = where_clause.cloned();
    for f in fields {
        if f.skip || f.was_optional {
            continue;
        }

        if let FieldStorage::ShapePolicy(shape) = field_storage(f) {
            let base_type = form_base_type(f);
            let runtime_crate = runtime_crate_path();
            let wc = holder_where_clause.get_or_insert_with(|| syn::parse_quote!(where));
            wc.predicates.push(syn::parse_quote! {
                <#shape as #runtime_crate::shape::ComponentShape>::RequiredValuePolicy:
                    #runtime_crate::shape::ValueHolderStorage<#base_type>
            });
        }
    }

    let field_definitions: Vec<TokenStream> = fields
        .iter()
        .filter(|f| !f.skip)
        .map(|f| {
            let field_name = &f.field_name;
            let field_type = form_field_type_tokens(f);
            let attrs = field_attrs
                .get(&field_name.to_string())
                .cloned()
                .unwrap_or_default();
            quote! {
                #(#attrs)* pub #field_name: #field_type
            }
        })
        .collect();

    let to_wrapped_fields: Vec<TokenStream> = fields
        .iter()
        .filter(|f| !f.skip)
        .map(generate_to_wrapped_field)
        .collect();

    let from_wrapped_fields: Vec<TokenStream> = fields
        .iter()
        .filter(|f| !f.skip)
        .map(generate_from_wrapped_field)
        .collect();

    let try_from_fields: Vec<TokenStream> = fields
        .iter()
        .filter(|f| !f.skip)
        .map(|f| try_from_field_tokens(f, quote! { from }, &conversion_error_ident))
        .collect();

    let present_fields_json_entries: Vec<TokenStream> = fields
        .iter()
        .filter(|f| !f.skip)
        .map(generate_present_fields_json_entry)
        .collect();

    let from_where_clause = holder_where_clause.clone();

    let skipped_params: Vec<TokenStream> = fields
        .iter()
        .filter(|f| f.skip)
        .map(|f| {
            let field_name = &f.field_name;
            let ty = &f.original_type;
            quote! { #field_name: #ty }
        })
        .collect();

    let into_original_fields: Vec<TokenStream> = fields
        .iter()
        .map(|f| {
            if f.skip {
                let field_name = &f.field_name;
                quote! { #field_name }
            } else {
                try_from_field_tokens(f, quote! { self }, &conversion_error_ident)
            }
        })
        .collect();

    let skipped_fields_impl = if has_skipped_fields {
        quote! {
            impl #impl_generics #wrapped_ident #ty_generics #holder_where_clause {
                pub fn present_fields_json(&self) -> String {
                    let mut entries: Vec<String> = Vec::new();
                    #(#present_fields_json_entries)*
                    format!("{{{}}}", entries.join(","))
                }

                fn escape_json_string(input: &str) -> String {
                    let mut escaped = String::new();
                    for ch in input.chars() {
                        match ch {
                            '"' => escaped.push_str("\\\""),
                            '\\' => escaped.push_str("\\\\"),
                            '\n' => escaped.push_str("\\n"),
                            '\r' => escaped.push_str("\\r"),
                            '\t' => escaped.push_str("\\t"),
                            '\u{08}' => escaped.push_str("\\b"),
                            '\u{0C}' => escaped.push_str("\\f"),
                            c if c.is_control() => {
                                use ::core::fmt::Write as _;
                                let _ = write!(&mut escaped, "\\u{:04x}", c as u32);
                            },
                            c => escaped.push(c),
                        }
                    }
                    escaped
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
    } else if reverse_conversion_can_fail {
        quote! {
            impl #impl_generics ::core::convert::TryFrom<#wrapped_ident #ty_generics>
                for #original_ident #ty_generics #holder_where_clause
            {
                type Error = #conversion_error_ident;

                fn try_from(from: #wrapped_ident #ty_generics) -> Result<Self, Self::Error> {
                    Ok(Self {
                        #(#try_from_fields),*
                    })
                }
            }

            impl #impl_generics #wrapped_ident #ty_generics #holder_where_clause {
                pub fn try_into_original(
                    self
                ) -> Result<#original_ident #ty_generics, #conversion_error_ident> {
                    <#original_ident #ty_generics as ::core::convert::TryFrom<
                        #wrapped_ident #ty_generics
                    >>::try_from(self)
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics ::core::convert::From<#wrapped_ident #ty_generics>
                for #original_ident #ty_generics #from_where_clause
            {
                fn from(from: #wrapped_ident #ty_generics) -> Self {
                    Self {
                        #(#from_wrapped_fields),*
                    }
                }
            }

            impl #impl_generics #wrapped_ident #ty_generics #holder_where_clause {
                pub fn into_original(self) -> #original_ident #ty_generics {
                    ::core::convert::From::from(self)
                }
            }
        }
    };

    let required_policy_validator = if needs_koruma_derive && has_shape_policy_required {
        required_value_policy_validator_tokens(
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
        #derive_output
        #builder_attr
        pub struct #wrapped_ident #impl_generics #holder_where_clause {
            #(#field_definitions),*
        }

        impl #impl_generics ::core::convert::From<#original_ident #ty_generics>
            for #wrapped_ident #ty_generics #holder_where_clause
        {
            fn from(from: #original_ident #ty_generics) -> Self {
                Self {
                    #(#to_wrapped_fields),*
                }
            }
        }

        #skipped_fields_impl
    };

    let default_impl = generate_default_impl(
        fields,
        &wrapped_ident,
        quote! { #impl_generics },
        quote! { #ty_generics },
        quote! { #holder_where_clause },
    );
    tokens = quote! {
        #tokens
        #default_impl
    };
    let clone_impl = generate_clone_impl(
        fields,
        &wrapped_ident,
        quote! { #impl_generics },
        quote! { #ty_generics },
        holder_where_clause.clone(),
    );
    let debug_impl = generate_debug_impl(
        fields,
        &wrapped_ident,
        quote! { #impl_generics },
        quote! { #ty_generics },
        holder_where_clause,
    );
    tokens = quote! {
        #tokens
        #clone_impl
        #debug_impl
    };

    (tokens, fields_requiring_required)
}
