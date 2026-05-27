use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, LitBool, LitStr, Path, Result, Type};

use super::component_shape_constructor::constructor_body_tokens;

pub(super) const SHAPE_METADATA_OPTIONS: &str = "`new = ...`, `state = ...`, `component = ...`, `requires_value = ...`, \
     `value_binding`, or `field_suffix = ...`";

#[derive(Debug, Default)]
pub(super) struct ComponentShapeMetadata {
    new: Option<Expr>,
    /// Optional backing state type when deriving on a render component.
    state: Option<Type>,
    /// Optional UI component type path.
    /// When set, `ComponentShape::COMPONENT_PATH` is populated so that
    /// field annotations do not need to repeat `component = ...`.
    component: Option<Path>,
    /// Whether non-optional source fields should keep a missing-value state in
    /// generated form value holders.
    requires_value: Option<LitBool>,
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
        component: Path,
        span: T,
    ) -> Result<()> {
        set_once(&mut self.component, component, span, "component")
    }

    pub(super) fn has_component(&self) -> bool {
        self.component.is_some()
    }

    pub(super) fn set_requires_value<T: quote::ToTokens>(
        &mut self,
        requires_value: LitBool,
        span: T,
    ) -> Result<()> {
        set_once(
            &mut self.requires_value,
            requires_value,
            span,
            "requires_value",
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

    pub(super) fn reject_value_binding_assignment(value: LitBool) -> syn::Error {
        syn::Error::new_spanned(
            value,
            "`value_binding` is a flag; use `value_binding` to enable it or omit it to disable it",
        )
    }

    pub(super) fn set_field_suffix<T: quote::ToTokens>(
        &mut self,
        field_suffix: LitStr,
        span: T,
    ) -> Result<()> {
        set_once(&mut self.field_suffix, field_suffix, span, "field_suffix")
    }

    pub(super) fn runtime_crate_path() -> Path {
        syn::parse_quote!(::gpui_form_runtime)
    }

    pub(super) fn constructor_body_or(&self, default_body: TokenStream) -> TokenStream {
        self.new
            .as_ref()
            .map(constructor_body_tokens)
            .unwrap_or(default_body)
    }

    pub(super) fn impl_items_tokens(&self, runtime_crate: &Path) -> TokenStream {
        let required_value_policy = if self
            .requires_value
            .as_ref()
            .map(syn::LitBool::value)
            .unwrap_or(true)
        {
            quote! { #runtime_crate::shape::RequireValue }
        } else {
            quote! { #runtime_crate::shape::AllowMissingValue }
        };
        let required_value_policy_assoc = quote! {
            type RequiredValuePolicy = #required_value_policy;
        };
        let component_path_const = self.component.as_ref().map(|component| {
            quote! {
                const COMPONENT_PATH: Option<&'static str> = Some(stringify!(#component));
            }
        });
        let value_binding_const = self.value_binding.then(|| {
            quote! {
                const VALUE_BINDING: bool = true;
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
            #component_path_const
            #value_binding_const
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
