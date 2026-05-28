use gpui_form_codegen::components::RequiredValue;
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
    quote! {
        <<#shape as ::gpui_form_runtime::shape::ComponentShape>::RequiredValuePolicy
            as ::gpui_form_runtime::shape::ValueHolderStorage<#base_type>>::Storage
    }
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
            quote! {
                #field_name: <<#shape as ::gpui_form_runtime::shape::ComponentShape>::RequiredValuePolicy
                    as ::gpui_form_runtime::shape::ValueHolderStorage<#base_type>>::try_map_into_value(
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
                        quote! {
                            #field_name: <<#shape as ::gpui_form_runtime::shape::ComponentShape>::RequiredValuePolicy
                                as ::gpui_form_runtime::shape::ValueHolderStorage<#base_type>>::present(#default_value)
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
                        quote! {
                            #field_name: <<#shape as ::gpui_form_runtime::shape::ComponentShape>::RequiredValuePolicy
                                as ::gpui_form_runtime::shape::ValueHolderStorage<#base_type>>::default_storage()
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
                        <<#shape as ::gpui_form_runtime::shape::ComponentShape>::RequiredValuePolicy
                            as ::gpui_form_runtime::shape::ValueHolderStorage<#base_type>>::present_unless_default(
                                value,
                                default_value
                            )
                    }
                }
            } else {
                let converted = apply_from_conversion(field, quote! { from.#field_name });
                quote! {
                    #field_name: <<#shape as ::gpui_form_runtime::shape::ComponentShape>::RequiredValuePolicy
                        as ::gpui_form_runtime::shape::ValueHolderStorage<#base_type>>::present(#converted)
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
            if let Some(default_expr) = &field.default_expr {
                let default_original = default_expr_for_original(default_expr);
                quote! {
                    #field_name: <<#shape as ::gpui_form_runtime::shape::ComponentShape>::RequiredValuePolicy
                        as ::gpui_form_runtime::shape::ValueHolderStorage<#base_type>>::map_into_value(
                            from.#field_name,
                            |value| #converted,
                            || #default_original
                        )
                }
            } else {
                let field_name_str = field_name.to_string();
                quote! {
                    #field_name: <<#shape as ::gpui_form_runtime::shape::ComponentShape>::RequiredValuePolicy
                        as ::gpui_form_runtime::shape::ValueHolderStorage<#base_type>>::map_into_value(
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
            quote! {
                if let Some(value) =
                    <<#shape as ::gpui_form_runtime::shape::ComponentShape>::RequiredValuePolicy
                        as ::gpui_form_runtime::shape::ValueHolderStorage<#base_type>>::map_present_cloned(
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

pub fn parse_field_default(field: &ComponentField) -> Option<syn::Expr> {
    field.default.as_ref().map(|expr| expr.0.clone())
}

/// Generates the FormValueHolder struct and its implementations.
pub fn generate_value_holder(
    original_input: &DeriveInput,
    fields: &[FieldOptionality],
    enable_koruma: bool,
    enable_koruma_fluent: bool,
) -> (TokenStream, Vec<String>) {
    let has_skipped_fields = fields.iter().any(|f| f.skip);
    let fields_requiring_required: Vec<String> = fields
        .iter()
        .filter(|f| f.needs_required_validation())
        .map(|f| f.field_name.to_string())
        .collect();

    let has_any_required = fields.iter().any(|f| f.needs_required_validation());

    let mut field_attrs: HashMap<String, Vec<TokenStream>> = HashMap::new();

    for f in fields {
        if f.skip {
            continue;
        }
        let field_name = f.field_name.to_string();

        if enable_koruma {
            let needs_required = f.needs_required_validation();

            let has_existing_validations = !f.validation.field_validators.is_empty()
                || !f.validation.element_validators.is_empty();
            let has_newtype = f.validation.is_newtype;

            if needs_required || has_existing_validations || has_newtype {
                let mut koruma_items: Vec<TokenStream> = Vec::new();

                if needs_required {
                    koruma_items.push(quote! {
                        koruma_collection::general::RequiredValidation::<Option<_>>::builder()
                    });
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

    // Check if any field has a custom default expression
    let has_custom_defaults = fields.iter().any(|f| !f.skip && f.default_expr.is_some());

    let mut derives: Vec<TokenStream> = vec![quote! { Clone }, quote! { Debug }];
    if !has_custom_defaults {
        derives.push(quote! { Default });
    }
    if has_skipped_fields {
        derives.push(quote! { ::gpui_form::bon::Builder });
    }
    if needs_koruma_derive {
        if enable_koruma_fluent {
            derives.push(quote! { ::koruma::Koruma });
            derives.push(quote! { ::koruma::KorumaAllFluent });
        } else {
            derives.push(quote! { ::koruma::Koruma });
        }
    }

    let derive_output = quote! { #[derive(#(#derives),*)] };
    let builder_attr = if has_skipped_fields {
        quote! { #[builder(crate = ::gpui_form::bon)] }
    } else {
        quote! {}
    };

    let original_ident = &original_input.ident;
    let wrapped_ident = format_ident!("{}FormValueHolder", original_ident);
    let conversion_error_ident = format_ident!("{}ConversionError", wrapped_ident);
    let conversion_error_type = generate_conversion_error_type(&conversion_error_ident);
    let (impl_generics, ty_generics, where_clause) = original_input.generics.split_for_impl();
    let mut holder_where_clause = where_clause.cloned();
    for f in fields {
        if f.skip || f.was_optional {
            continue;
        }

        if let FieldStorage::ShapePolicy(shape) = field_storage(f) {
            let base_type = form_base_type(f);
            let wc = holder_where_clause.get_or_insert_with(|| syn::parse_quote!(where));
            wc.predicates.push(syn::parse_quote! {
                <#shape as ::gpui_form_runtime::shape::ComponentShape>::RequiredValuePolicy:
                    ::gpui_form_runtime::shape::ValueHolderStorage<#base_type>
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

    let mut from_where_clause = holder_where_clause.clone();
    let mut new_predicates: Vec<syn::WherePredicate> = Vec::new();
    for f in fields {
        if !f.skip && !f.was_optional && matches!(field_storage(f), FieldStorage::RequiredValue) {
            let original_type = &f.original_type;
            new_predicates.push(syn::parse_quote!(#original_type: ::core::default::Default));
        }
    }
    if !new_predicates.is_empty() {
        let wc = from_where_clause.get_or_insert_with(|| syn::parse_quote!(where));
        wc.predicates.extend(new_predicates);
    }

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
                pub fn try_from(
                    from: #wrapped_ident #ty_generics
                ) -> Result<#original_ident #ty_generics, #conversion_error_ident> {
                    Ok(#original_ident {
                        #(#try_from_fields),*
                    })
                }
            }
        }
    };

    let mut tokens = quote! {
        #conversion_error_type
        #derive_output
        #builder_attr
        pub struct #wrapped_ident #ty_generics #holder_where_clause {
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

    if has_custom_defaults {
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
    }

    (tokens, fields_requiring_required)
}
