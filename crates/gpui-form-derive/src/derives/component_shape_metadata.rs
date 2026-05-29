use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Expr, Ident, LitStr, Path, Result, Token, Type,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use gpui_form_codegen::{CratePaths, components::validate_shape_field_suffix};

use super::component_shape_constructor::constructor_body_tokens;

pub(super) const SHAPE_METADATA_OPTIONS: &str = "`new = ...`, `state = ...`, `component = ...`, `value = ...`, `values(...)`, \
     `value_storage = require_value|direct`, `value_binding`, or `field_suffix = ...`";

#[derive(Clone, Copy, Debug)]
pub(super) enum ValueStoragePolicy {
    RequiredValueStorage,
    Direct,
}

impl Parse for ValueStoragePolicy {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        if ident == "require_value" {
            Ok(Self::RequiredValueStorage)
        } else if ident == "direct" {
            Ok(Self::Direct)
        } else {
            Err(syn::Error::new_spanned(
                ident,
                "expected `require_value` or `direct` for `value_storage`",
            ))
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct ComponentShapeMetadata {
    new: Option<Expr>,
    /// Optional backing state type when deriving on a render component.
    state: Option<Type>,
    /// Optional UI component type.
    /// When set, `ComponentShape::COMPONENT_TYPE` is populated so that
    /// field annotations do not need to repeat `component = ...`.
    component: Option<Type>,
    /// Explicit form-side value types supported by this shape.
    values: Vec<Type>,
    /// Storage policy for non-optional source fields in generated form value
    /// holders.
    value_storage: Option<ValueStoragePolicy>,
    /// Opt generated prototyping code into ComponentValueBinding by default.
    value_binding: bool,
    /// Preferred generated field/helper suffix for prototyping output.
    field_suffix: Option<LitStr>,
}

impl ComponentShapeMetadata {
    pub(super) fn set_new<T: quote::ToTokens>(&mut self, new: Expr, span: T) -> Result<()> {
        set_once(&mut self.new, new, span, "new")
    }

    pub(super) fn set_state<T: quote::ToTokens>(&mut self, state: Type, span: T) -> Result<()> {
        set_once(&mut self.state, state, span, "state")
    }

    pub(super) fn state(&self) -> Option<&Type> {
        self.state.as_ref()
    }

    pub(super) fn set_component<T: quote::ToTokens>(
        &mut self,
        component: Type,
        span: T,
    ) -> Result<()> {
        match &component {
            Type::Path(_) => {},
            Type::Infer(_) => {
                return Err(syn::Error::new_spanned(
                    span,
                    "component type cannot be inferred with bare `_`; use an explicit component type",
                ));
            },
            _ => {
                return Err(syn::Error::new_spanned(
                    span,
                    "component metadata must be a path-like type, such as `my_crate::Input`",
                ));
            },
        }

        set_once(&mut self.component, component, span, "component")
    }

    pub(super) fn has_component(&self) -> bool {
        self.component.is_some()
    }

    pub(super) fn add_value<T: quote::ToTokens>(&mut self, value: Type, _span: T) -> Result<()> {
        self.values.push(value);
        Ok(())
    }

    pub(super) fn add_values<I, T>(&mut self, values: I, span: T) -> Result<()>
    where
        I: IntoIterator<Item = Type>,
        T: quote::ToTokens,
    {
        let mut added = false;
        for value in values {
            self.add_value(value, &span)?;
            added = true;
        }

        if !added {
            return Err(syn::Error::new_spanned(
                span,
                "`values(...)` requires at least one value type",
            ));
        }

        Ok(())
    }

    pub(super) fn parse_values(input: ParseStream<'_>) -> Result<Vec<Type>> {
        Ok(Punctuated::<Type, Token![,]>::parse_terminated(input)?
            .into_iter()
            .collect())
    }

    pub(super) fn value_impl_tokens(
        &self,
        runtime_crate: &Path,
        ident: &Ident,
        generics: &syn::Generics,
    ) -> TokenStream {
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let impls = self.values.iter().map(|value| {
            quote! {
                impl #impl_generics #runtime_crate::shape::ComponentShapeFor<#value>
                    for #ident #ty_generics
                    #where_clause
                {
                }
            }
        });

        quote! { #(#impls)* }
    }

    pub(super) fn set_value_storage<T: quote::ToTokens>(
        &mut self,
        value_storage: ValueStoragePolicy,
        span: T,
    ) -> Result<()> {
        set_once(
            &mut self.value_storage,
            value_storage,
            span,
            "value_storage",
        )
    }

    pub(super) fn enable_value_binding<T: quote::ToTokens>(&mut self, span: T) -> Result<()> {
        if self.value_binding {
            return Err(syn::Error::new_spanned(
                span,
                "duplicate `value_binding` option",
            ));
        }

        self.value_binding = true;
        Ok(())
    }

    pub(super) fn has_value_binding(&self) -> bool {
        self.value_binding
    }

    pub(super) fn set_field_suffix<T: quote::ToTokens>(
        &mut self,
        field_suffix: LitStr,
        span: T,
    ) -> Result<()> {
        validate_shape_field_suffix(&field_suffix.value())
            .map_err(|message| syn::Error::new_spanned(&field_suffix, message))?;
        set_once(&mut self.field_suffix, field_suffix, span, "field_suffix")
    }

    pub(super) fn runtime_crate_path() -> Path {
        CratePaths::resolve().gpui_form_runtime
    }

    pub(super) fn constructor_body_or(&self, default_body: TokenStream) -> TokenStream {
        self.new
            .as_ref()
            .map(constructor_body_tokens)
            .unwrap_or(default_body)
    }

    pub(super) fn render_component_contract_tokens(
        &self,
        gpui_crate: &Path,
        state: &Type,
    ) -> TokenStream {
        let Some(component) = &self.component else {
            return quote! {};
        };

        if type_contains_infer(component) {
            return quote! {};
        }

        quote! {
            #[allow(dead_code)]
            fn __gpui_form_assert_render_component_contract(
                entity: &#gpui_crate::Entity<#state>,
            ) -> #component {
                <#component>::new(entity)
            }
        }
    }

    pub(super) fn impl_items_tokens(&self, runtime_crate: &Path) -> TokenStream {
        let value_storage_policy = match self
            .value_storage
            .unwrap_or(ValueStoragePolicy::RequiredValueStorage)
        {
            ValueStoragePolicy::RequiredValueStorage => {
                quote! { #runtime_crate::shape::RequiredValueStorage }
            },
            ValueStoragePolicy::Direct => quote! { #runtime_crate::shape::DirectValueStorage },
        };
        let value_storage_policy_assoc = quote! {
            type ValueStoragePolicy = #value_storage_policy;
        };
        let value_binding_policy = if self.value_binding {
            quote! { #runtime_crate::shape::InheritedComponentValueBinding }
        } else {
            quote! { #runtime_crate::shape::NoComponentValueBinding }
        };
        let value_binding_policy_assoc = quote! {
            type ValueBindingPolicy = #value_binding_policy;
        };
        let component_type_const = self.component.as_ref().map(|component| {
            quote! {
                const COMPONENT_TYPE: Option<&'static str> = Some(stringify!(#component));
            }
        });
        let prototyping_const = self.field_suffix.as_ref().map(|field_suffix| {
            quote! {
                const PROTOTYPING: #runtime_crate::shape::ComponentPrototyping =
                    #runtime_crate::shape::ComponentPrototyping::new()
                        .field_suffix(#field_suffix);
            }
        });

        quote! {
            #value_storage_policy_assoc
            #value_binding_policy_assoc
            #component_type_const
            #prototyping_const
        }
    }
}

fn type_contains_infer(ty: &Type) -> bool {
    match ty {
        Type::Infer(_) => true,
        Type::Path(path) => path.path.segments.iter().any(|segment| match &segment.arguments {
            syn::PathArguments::AngleBracketed(args) => args.args.iter().any(|arg| match arg {
                syn::GenericArgument::Type(ty) => type_contains_infer(ty),
                syn::GenericArgument::AssocType(assoc) => type_contains_infer(&assoc.ty),
                syn::GenericArgument::Constraint(constraint) => constraint
                    .bounds
                    .iter()
                    .any(|bound| matches!(bound, syn::TypeParamBound::Trait(_))),
                _ => false,
            }),
            syn::PathArguments::Parenthesized(args) => {
                args.inputs.iter().any(type_contains_infer)
                    || matches!(&args.output, syn::ReturnType::Type(_, ty) if type_contains_infer(ty))
            },
            syn::PathArguments::None => false,
        }),
        Type::Array(array) => type_contains_infer(&array.elem),
        Type::BareFn(function) => {
            function.inputs.iter().any(|input| type_contains_infer(&input.ty))
                || matches!(&function.output, syn::ReturnType::Type(_, ty) if type_contains_infer(ty))
        },
        Type::Group(group) => type_contains_infer(&group.elem),
        Type::Paren(paren) => type_contains_infer(&paren.elem),
        Type::Ptr(ptr) => type_contains_infer(&ptr.elem),
        Type::Reference(reference) => type_contains_infer(&reference.elem),
        Type::Slice(slice) => type_contains_infer(&slice.elem),
        Type::Tuple(tuple) => tuple.elems.iter().any(type_contains_infer),
        _ => false,
    }
}

fn set_once<T, S: quote::ToTokens>(
    slot: &mut Option<T>,
    value: T,
    span: S,
    option: &str,
) -> Result<()> {
    if slot.replace(value).is_some() {
        return Err(syn::Error::new_spanned(
            span,
            format!("duplicate `{option}` option"),
        ));
    }

    Ok(())
}
