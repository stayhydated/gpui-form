use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, LitBool, LitStr, Path, Result};

use super::component_shape_constructor::constructor_body_tokens;

pub(super) const SHAPE_METADATA_OPTIONS: &str =
    "`new = ...`, `component = ...`, `value_binding`, `field_suffix = ...`, or `shape_crate = ...`";

#[derive(Debug, Default)]
pub(super) struct ComponentShapeMetadata {
    new: Option<Expr>,
    /// Optional UI component type path.
    /// When set, `ComponentShape::COMPONENT_PATH` is populated so that
    /// field annotations do not need to repeat `component = ...`.
    component: Option<Path>,
    /// Opt generated prototyping code into ComponentValueBinding by default.
    value_binding: bool,
    /// Preferred generated field/helper suffix for prototyping output.
    field_suffix: Option<LitStr>,
    /// Runtime crate path that owns `shape::ComponentShape`.
    ///
    /// This defaults to the explicit `gpui_form_component` runtime crate for
    /// downstream users.
    /// Runtime crates that own a state type can set `shape_crate = crate`.
    shape_crate: Option<Path>,
}

impl ComponentShapeMetadata {
    pub(super) fn set_new<T: quote::ToTokens>(&mut self, new: Expr, span: T) -> Result<()> {
        set_once(&mut self.new, new, span, "new")
    }

    pub(super) fn set_component<T: quote::ToTokens>(
        &mut self,
        component: Path,
        span: T,
    ) -> Result<()> {
        set_once(&mut self.component, component, span, "component")
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

    pub(super) fn set_shape_crate<T: quote::ToTokens>(
        &mut self,
        shape_crate: Path,
        span: T,
    ) -> Result<()> {
        set_once(&mut self.shape_crate, shape_crate, span, "shape_crate")
    }

    pub(super) fn shape_crate_path(&self) -> Path {
        self.shape_crate
            .clone()
            .unwrap_or_else(|| syn::parse_quote!(::gpui_form_component))
    }

    pub(super) fn constructor_body_or(&self, default_body: TokenStream) -> TokenStream {
        self.new
            .as_ref()
            .map(constructor_body_tokens)
            .unwrap_or(default_body)
    }

    pub(super) fn const_tokens(&self, shape_crate: &Path) -> TokenStream {
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
                const PROTOTYPING: #shape_crate::shape::ComponentPrototyping =
                    #shape_crate::shape::ComponentPrototyping::new()
                        .field_suffix(#field_suffix);
            }
        });

        quote! {
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
