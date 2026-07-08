use darling::{FromDeriveInput, FromVariant};
use gpui_form_codegen::resolve_crate_path;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{DeriveInput, Ident, Type};

#[derive(FromVariant)]
#[darling(attributes(tuple_enum))]
struct VariantArgs {
    ident: Ident,
    fields: darling::ast::Fields<syn::Field>,
    #[darling(default)]
    skip: bool,
    #[darling(default)]
    key: Option<String>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(tuple_enum), forward_attrs(fluent_kv), supports(enum_any))]
struct InfiniteSelectArgs {
    ident: Ident,
    data: darling::ast::Data<VariantArgs, ()>,
    attrs: Vec<syn::Attribute>,
}

struct VariantInfo {
    ident: Ident,
    key: String,
    inner_type: Option<Type>,
    field_name: Option<Ident>,
    is_unit: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FluentKvOptions {
    has_label: bool,
    has_description: bool,
    keys_this: bool,
}

impl FluentKvOptions {
    fn from_attrs(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        attrs
            .iter()
            .filter(|attr| attr.path().is_ident("fluent_kv"))
            .try_fold(Self::default(), |options, attr| {
                Ok(options.merged(Self::from_attr(attr)?))
            })
    }

    fn from_attr(attr: &syn::Attribute) -> syn::Result<Self> {
        let mut options = Self::default();

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("keys") {
                let list: syn::ExprArray = meta.value()?.parse()?;
                options = options.merged(Self::from_keys(list));
            } else if meta.path.is_ident("keys_this") {
                options.keys_this = true;
            }

            Ok(())
        })?;

        Ok(options)
    }

    fn from_keys(list: syn::ExprArray) -> Self {
        let mut options = Self::default();

        for elem in list.elems {
            if let syn::Expr::Lit(lit) = elem
                && let syn::Lit::Str(string) = lit.lit
            {
                match string.value().as_str() {
                    "label" => options.has_label = true,
                    "description" => options.has_description = true,
                    _ => {},
                }
            }
        }

        options
    }

    fn merged(self, other: Self) -> Self {
        Self {
            has_label: self.has_label || other.has_label,
            has_description: self.has_description || other.has_description,
            keys_this: self.keys_this || other.keys_this,
        }
    }

    fn uses_type_label(self) -> bool {
        self.has_label && self.keys_this
    }

    fn uses_type_description(self) -> bool {
        self.has_description && self.keys_this
    }
}

impl VariantInfo {
    fn ignore_pattern(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        if self.is_unit {
            quote! { Self::#ident }
        } else if let Some(field_name) = &self.field_name {
            quote! { Self::#ident { #field_name: _ } }
        } else {
            quote! { Self::#ident(_) }
        }
    }

    fn binding_pattern(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        if self.is_unit {
            quote! { Self::#ident }
        } else if let Some(field_name) = &self.field_name {
            quote! { Self::#ident { #field_name: inner } }
        } else {
            quote! { Self::#ident(inner) }
        }
    }

    fn constructor(&self, value: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        if self.is_unit {
            quote! { Self::#ident }
        } else if let Some(field_name) = &self.field_name {
            quote! { Self::#ident { #field_name: #value } }
        } else {
            quote! { Self::#ident(#value) }
        }
    }
}

fn map_variant_arms<FLeaf, FInner>(
    variants: &[VariantInfo],
    mut leaf: FLeaf,
    mut inner: FInner,
) -> Vec<proc_macro2::TokenStream>
where
    FLeaf: FnMut(&VariantInfo) -> proc_macro2::TokenStream,
    FInner: FnMut(&VariantInfo, &Type) -> proc_macro2::TokenStream,
{
    variants
        .iter()
        .map(|variant| match variant.inner_type.as_ref() {
            Some(inner_type) => inner(variant, inner_type),
            None => leaf(variant),
        })
        .collect()
}

pub fn from(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let args = match InfiniteSelectArgs::from_derive_input(&input) {
        Ok(args) => args,
        Err(err) => return err.write_errors().into(),
    };

    let enum_ident = &args.ident;
    let runtime_crate = resolve_crate_path("gpui-form-component", "::gpui_form_component");

    let fluent_kv = match FluentKvOptions::from_attrs(&args.attrs) {
        Ok(options) => options,
        Err(err) => return err.to_compile_error().into(),
    };

    let _type_label_uses_fluent = fluent_kv.uses_type_label();
    let type_label_impl = quote! { stringify!(#enum_ident).into() };

    let _type_description_uses_fluent = fluent_kv.uses_type_description();
    let type_description_impl = quote! { stringify!(#enum_ident).into() };

    let variants: Result<Vec<VariantInfo>, syn::Error> = match &args.data {
        darling::ast::Data::Enum(variants) => variants
            .iter()
            .filter(|variant| !variant.skip)
            .map(|variant| {
                let (inner_type, field_name) = match &variant.fields.style {
                    darling::ast::Style::Tuple => {
                        if variant.fields.fields.len() == 1 {
                            (Some(variant.fields.fields[0].ty.clone()), None)
                        } else if variant.fields.fields.is_empty() {
                            (None, None)
                        } else {
                            return Err(syn::Error::new_spanned(
                                &variant.ident,
                                format!(
                                    "InfiniteSelect only supports single-element tuple variants for tree construction, got {} elements in variant `{}`",
                                    variant.fields.fields.len(),
                                    variant.ident
                                ),
                            ));
                        }
                    }
                    darling::ast::Style::Unit => (None, None),
                    darling::ast::Style::Struct => {
                        if variant.fields.fields.len() == 1 {
                            let field = &variant.fields.fields[0];
                            let Some(field_name) = field.ident.clone() else {
                                return Err(syn::Error::new_spanned(
                                    field,
                                    "InfiniteSelect only supports named struct variant fields",
                                ));
                            };
                            (Some(field.ty.clone()), Some(field_name))
                        } else if variant.fields.fields.is_empty() {
                            (None, None)
                        } else {
                            return Err(syn::Error::new_spanned(
                                &variant.ident,
                                format!(
                                    "InfiniteSelect only supports single-field struct variants for tree construction, got {} fields in variant `{}`",
                                    variant.fields.fields.len(),
                                    variant.ident
                                ),
                            ));
                        }
                    }
                };

                let is_unit = inner_type.is_none();
                Ok(VariantInfo {
                    ident: variant.ident.clone(),
                    key: variant
                        .key
                        .clone()
                        .unwrap_or_else(|| variant.ident.to_string()),
                    inner_type,
                    field_name,
                    is_unit,
                })
            })
            .collect(),
        _ => unreachable!("InfiniteSelect only supports enums"),
    };

    let variants = match variants {
        Ok(variants) => variants,
        Err(err) => return err.to_compile_error().into(),
    };

    let mut seen_keys = HashMap::new();
    for variant in &variants {
        if let Some(previous_variant) = seen_keys.insert(variant.key.clone(), variant.ident.clone())
        {
            return syn::Error::new_spanned(
                &variant.ident,
                format!(
                    "duplicate infinite-select key {:?} on variants `{}` and `{}`",
                    variant.key, previous_variant, variant.ident
                ),
            )
            .to_compile_error()
            .into();
        }
    }

    let all_unit = variants.iter().all(|variant| variant.is_unit);

    let variant_items: Vec<_> = variants
        .iter()
        .map(|variant| {
            let constructor = variant.constructor(quote! { Default::default() });
            quote! { #constructor, }
        })
        .collect();

    let variant_name_arms: Vec<_> = variants
        .iter()
        .map(|variant| {
            let pattern = variant.ignore_pattern();
            let name = variant.ident.to_string();
            quote! { #pattern => #name, }
        })
        .collect();

    let variant_key_arms: Vec<_> = variants
        .iter()
        .map(|variant| {
            let pattern = variant.ignore_pattern();
            let key = &variant.key;
            quote! { #pattern => #key, }
        })
        .collect();

    let variant_label_arms: Vec<_> = variants
        .iter()
        .map(|variant| {
            let pattern = variant.ignore_pattern();
            let label = variant.ident.to_string();
            quote! { #pattern => #label.into(), }
        })
        .collect();

    let has_inner_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => false, }
        },
        |variant, _| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => true, }
        },
    );

    let child_variant_names_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => vec![], }
        },
        |variant, inner_type| {
            let pattern = variant.ignore_pattern();
            quote! {
                #pattern => {
                    <#inner_type as #runtime_crate::infinite_select::InfiniteSelectValue>::variants()
                        .into_iter()
                        .map(|variant| variant.variant_name())
                        .collect()
                }
            }
        },
    );

    let child_variant_key_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => vec![], }
        },
        |variant, inner_type| {
            let pattern = variant.ignore_pattern();
            quote! {
                #pattern => {
                    <#inner_type as #runtime_crate::infinite_select::InfiniteSelectValue>::variants()
                        .into_iter()
                        .map(|variant| variant.variant_key())
                        .collect()
                }
            }
        },
    );

    let child_variant_label_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => vec![], }
        },
        |variant, inner_type| {
            let pattern = variant.ignore_pattern();
            quote! {
                #pattern => {
                    <#inner_type as #runtime_crate::infinite_select::InfiniteSelectValue>::variants()
                        .into_iter()
                        .map(|variant| variant.variant_label())
                        .collect()
                }
            }
        },
    );

    let inner_child_variant_names_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => vec![], }
        },
        |variant, _| {
            let pattern = variant.binding_pattern();
            quote! { #pattern => inner.child_variant_names(), }
        },
    );

    let inner_child_variant_key_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => vec![], }
        },
        |variant, _| {
            let pattern = variant.binding_pattern();
            quote! { #pattern => inner.child_variant_keys(), }
        },
    );

    let inner_child_variant_label_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => vec![], }
        },
        |variant, _| {
            let pattern = variant.binding_pattern();
            quote! { #pattern => inner.child_variant_labels(), }
        },
    );

    let inner_set_child_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => None, }
        },
        |variant, _| {
            let pattern = variant.binding_pattern();
            let constructor = variant.constructor(quote! { new_inner });
            quote! {
                #pattern => {
                    inner.set_child_by_index(index).map(|new_inner| #constructor)
                }
            }
        },
    );

    let inner_set_child_key_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => None, }
        },
        |variant, _| {
            let pattern = variant.binding_pattern();
            let constructor = variant.constructor(quote! { new_inner });
            quote! {
                #pattern => {
                    inner.set_child_by_key(key).map(|new_inner| #constructor)
                }
            }
        },
    );

    let inner_has_inner_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => false, }
        },
        |variant, _| {
            let pattern = variant.binding_pattern();
            quote! { #pattern => inner.has_inner(), }
        },
    );

    let set_child_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => None, }
        },
        |variant, inner_type| {
            let pattern = variant.ignore_pattern();
            let constructor = variant.constructor(quote! { child.clone() });
            quote! {
                #pattern => {
                    let children = <#inner_type as #runtime_crate::infinite_select::InfiniteSelectValue>::variants();
                    children.get(index).map(|child| #constructor)
                }
            }
        },
    );

    let set_child_key_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => None, }
        },
        |variant, inner_type| {
            let pattern = variant.ignore_pattern();
            let constructor = variant.constructor(quote! { child });
            quote! {
                #pattern => {
                    let children = <#inner_type as #runtime_crate::infinite_select::InfiniteSelectValue>::variants();
                    children
                        .into_iter()
                        .find(|child| child.variant_key() == key)
                        .map(|child| #constructor)
                }
            }
        },
    );

    let set_child_by_path_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => None, }
        },
        |variant, inner_type| {
            let pattern = variant.ignore_pattern();
            let constructor_child = variant.constructor(quote! { child });
            let constructor_updated = variant.constructor(quote! { updated_child });
            quote! {
                #pattern => {
                    if path.is_empty() {
                        return None;
                    }
                    let children = <#inner_type as #runtime_crate::infinite_select::InfiniteSelectValue>::variants();
                    let child = children.get(path[0])?.clone();
                    if path.len() == 1 {
                        Some(#constructor_child)
                    } else {
                        let updated_child = child.set_child_by_path(&path[1..])?;
                        Some(#constructor_updated)
                    }
                }
            }
        },
    );

    let set_child_by_key_path_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => None, }
        },
        |variant, inner_type| {
            let pattern = variant.ignore_pattern();
            let constructor_child = variant.constructor(quote! { child });
            let constructor_updated = variant.constructor(quote! { updated_child });
            quote! {
                #pattern => {
                    if path.is_empty() {
                        return None;
                    }
                    let children = <#inner_type as #runtime_crate::infinite_select::InfiniteSelectValue>::variants();
                    let child = children
                        .into_iter()
                        .find(|child| child.variant_key() == path[0].as_str())?;
                    if path.len() == 1 {
                        Some(#constructor_child)
                    } else {
                        let updated_child = child.set_child_by_key_path(&path[1..])?;
                        Some(#constructor_updated)
                    }
                }
            }
        },
    );

    let child_depth_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => 0, }
        },
        |variant, inner_type| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => <#inner_type as #runtime_crate::infinite_select::InfiniteSelectValue>::depth(), }
        },
    );

    let selection_path_arms: Vec<_> = variants
        .iter()
        .enumerate()
        .map(|(index, variant)| {
            let root_index = index;
            match variant.inner_type.as_ref() {
                Some(_) => {
                    let pattern = variant.binding_pattern();
                    quote! {
                        #pattern => {
                            let mut indices = vec![#root_index];
                            indices.extend(inner.selection_path().indices().iter().copied());
                            #runtime_crate::infinite_select::InfiniteSelectPath::with_indices(indices)
                        }
                    }
                },
                None => {
                    let pattern = variant.ignore_pattern();
                    quote! {
                        #pattern => {
                            #runtime_crate::infinite_select::InfiniteSelectPath::with_indices(vec![#root_index])
                        }
                    }
                },
            }
        })
        .collect();

    let selection_key_path_arms: Vec<_> = variants
        .iter()
        .map(|variant| {
            let root_key = &variant.key;
            match variant.inner_type.as_ref() {
                Some(_) => {
                    let pattern = variant.binding_pattern();
                    quote! {
                        #pattern => {
                            let mut keys = vec![#root_key.to_string()];
                            keys.extend(inner.selection_key_path().keys().iter().cloned());
                            #runtime_crate::infinite_select::InfiniteSelectKeyPath::with_keys(keys)
                        }
                    }
                },
                None => {
                    let pattern = variant.ignore_pattern();
                    quote! {
                        #pattern => {
                            #runtime_crate::infinite_select::InfiniteSelectKeyPath::with_keys(vec![#root_key.to_string()])
                        }
                    }
                },
            }
        })
        .collect();

    let inner_child_label_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => None, }
        },
        |variant, _| {
            let pattern = variant.binding_pattern();
            quote! { #pattern => inner.child_label_at_depth(depth), }
        },
    );

    let inner_child_description_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => None, }
        },
        |variant, _| {
            let pattern = variant.binding_pattern();
            quote! { #pattern => inner.child_description_at_depth(depth), }
        },
    );

    let child_label_immediate_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => None, }
        },
        |variant, _| {
            let pattern = variant.binding_pattern();
            quote! { #pattern => Some(inner.type_label()), }
        },
    );

    let child_description_immediate_arms = map_variant_arms(
        &variants,
        |variant| {
            let pattern = variant.ignore_pattern();
            quote! { #pattern => None, }
        },
        |variant, _| {
            let pattern = variant.binding_pattern();
            quote! { #pattern => Some(inner.type_description()), }
        },
    );

    let depth_calculation = if all_unit {
        quote! { 1 }
    } else {
        let depth_checks = map_variant_arms(
            &variants,
            |_| quote! {},
            |_, inner_type| {
                quote! { <#inner_type as #runtime_crate::infinite_select::InfiniteSelectValue>::depth() }
            },
        )
        .into_iter()
        .filter(|tokens| !tokens.is_empty())
        .collect::<Vec<_>>();

        quote! {
            1 + [#(#depth_checks),*].into_iter().max().unwrap_or(0)
        }
    };

    let expanded = quote! {
        impl #runtime_crate::infinite_select::InfiniteSelectValue for #enum_ident {
            fn variants() -> Vec<Self> {
                vec![
                    #(#variant_items)*
                ]
            }

            fn variant_name(&self) -> &'static str {
                match self {
                    #(#variant_name_arms)*
                }
            }

            fn variant_key(&self) -> &'static str {
                match self {
                    #(#variant_key_arms)*
                }
            }

            fn variant_label(&self) -> gpui::SharedString {
                match self {
                    #(#variant_label_arms)*
                }
            }

            fn has_inner(&self) -> bool {
                match self {
                    #(#has_inner_arms)*
                }
            }

            fn child_variant_names(&self) -> Vec<&'static str> {
                match self {
                    #(#child_variant_names_arms)*
                }
            }

            fn child_variant_keys(&self) -> Vec<&'static str> {
                match self {
                    #(#child_variant_key_arms)*
                }
            }

            fn child_variant_labels(&self) -> Vec<gpui::SharedString> {
                match self {
                    #(#child_variant_label_arms)*
                }
            }

            fn set_child_by_index(&self, index: usize) -> Option<Self> {
                match self {
                    #(#set_child_arms)*
                }
            }

            fn set_child_by_key(&self, key: &str) -> Option<Self> {
                match self {
                    #(#set_child_key_arms)*
                }
            }

            fn set_child_by_path(&self, path: &[usize]) -> Option<Self> {
                match self {
                    #(#set_child_by_path_arms)*
                }
            }

            fn set_child_by_key_path(&self, path: &[String]) -> Option<Self> {
                match self {
                    #(#set_child_by_key_path_arms)*
                }
            }

            fn child_depth(&self) -> usize {
                match self {
                    #(#child_depth_arms)*
                }
            }

            fn depth() -> usize {
                #depth_calculation
            }

            fn selection_path(&self) -> #runtime_crate::infinite_select::InfiniteSelectPath {
                match self {
                    #(#selection_path_arms)*
                }
            }

            fn selection_key_path(&self) -> #runtime_crate::infinite_select::InfiniteSelectKeyPath {
                match self {
                    #(#selection_key_path_arms)*
                }
            }

            fn inner_child_variant_names(&self) -> Vec<&'static str> {
                match self {
                    #(#inner_child_variant_names_arms)*
                }
            }

            fn inner_child_variant_keys(&self) -> Vec<&'static str> {
                match self {
                    #(#inner_child_variant_key_arms)*
                }
            }

            fn inner_child_variant_labels(&self) -> Vec<gpui::SharedString> {
                match self {
                    #(#inner_child_variant_label_arms)*
                }
            }

            fn inner_set_child_by_index(&self, index: usize) -> Option<Self> {
                match self {
                    #(#inner_set_child_arms)*
                }
            }

            fn inner_set_child_by_key(&self, key: &str) -> Option<Self> {
                match self {
                    #(#inner_set_child_key_arms)*
                }
            }

            fn inner_has_inner(&self) -> bool {
                match self {
                    #(#inner_has_inner_arms)*
                }
            }

            fn type_label(&self) -> gpui::SharedString {
                #type_label_impl
            }

            fn type_description(&self) -> gpui::SharedString {
                #type_description_impl
            }

            fn inner_child_label_at_depth(&self, depth: usize) -> Option<gpui::SharedString> {
                match self {
                    #(#inner_child_label_arms)*
                }
            }

            fn inner_child_description_at_depth(&self, depth: usize) -> Option<gpui::SharedString> {
                match self {
                    #(#inner_child_description_arms)*
                }
            }

            fn child_label_at_depth(&self, depth: usize) -> Option<gpui::SharedString> {
                if depth == 0 {
                    match self {
                        #(#child_label_immediate_arms)*
                    }
                } else {
                    self.inner_child_label_at_depth(depth - 1)
                }
            }

            fn child_description_at_depth(&self, depth: usize) -> Option<gpui::SharedString> {
                if depth == 0 {
                    match self {
                        #(#child_description_immediate_arms)*
                    }
                } else {
                    self.inner_child_description_at_depth(depth - 1)
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::{FluentKvOptions, VariantArgs};
    use darling::FromVariant as _;

    #[test]
    fn fluent_kv_options_merge_across_attributes() {
        let attrs = vec![
            syn::parse_quote!(#[doc = "ignored"]),
            syn::parse_quote!(#[fluent_kv(keys = ["label"])]),
            syn::parse_quote!(#[fluent_kv(keys = ["description"], keys_this)]),
        ];

        let options = FluentKvOptions::from_attrs(&attrs).expect("fluent_kv attrs should parse");

        assert_eq!(
            options,
            FluentKvOptions {
                has_label: true,
                has_description: true,
                keys_this: true,
            }
        );
    }

    #[test]
    fn fluent_kv_options_report_invalid_keys_syntax() {
        let attrs = vec![syn::parse_quote!(#[fluent_kv(keys = "label")])];

        assert!(FluentKvOptions::from_attrs(&attrs).is_err());
    }

    #[test]
    fn variant_args_parse_custom_key() {
        let variant: syn::Variant = syn::parse_quote!(
            #[tuple_enum(key = "docs/reference")]
            Docs
        );

        let args = VariantArgs::from_variant(&variant).expect("variant attrs should parse");

        assert_eq!(args.key.as_deref(), Some("docs/reference"));
    }
}
