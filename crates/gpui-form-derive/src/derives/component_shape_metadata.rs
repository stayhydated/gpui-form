use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Expr, Ident, LitStr, Path, Result, Type,
    parse::{Parse, ParseStream},
};

use gpui_form_codegen::{CratePaths, components::validate_shape_field_suffix};

use super::component_shape_constructor::constructor_body_tokens;

pub(super) const SHAPE_METADATA_OPTIONS: &str = "`new = ...`, `state = ...`, `component = ...`, `value_storage = require_value|direct`, \
     `value_binding`, or `field_suffix = ...`";

#[derive(Clone, Copy, Debug)]
pub(super) enum ValueStoragePolicy {
    RequireValue,
    Direct,
}

impl Parse for ValueStoragePolicy {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        if ident == "require_value" {
            Ok(Self::RequireValue)
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

    pub(super) fn impl_items_tokens(&self, runtime_crate: &Path) -> TokenStream {
        let required_value_policy = match self
            .value_storage
            .unwrap_or(ValueStoragePolicy::RequireValue)
        {
            ValueStoragePolicy::RequireValue => quote! { #runtime_crate::shape::RequireValue },
            ValueStoragePolicy::Direct => quote! { #runtime_crate::shape::AllowMissingValue },
        };
        let required_value_policy_assoc = quote! {
            type RequiredValuePolicy = #required_value_policy;
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
            #required_value_policy_assoc
            #value_binding_policy_assoc
            #component_type_const
            #prototyping_const
        }
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
