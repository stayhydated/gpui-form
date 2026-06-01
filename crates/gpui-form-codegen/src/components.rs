use component_shape_codegen::{
    ResolvedComponentShape as SharedResolvedComponentShape, ShapeOptions as SharedShapeOptions,
    field_assertion_ident_fragment, rust_path_metadata_tokens, tokens_with_span,
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};

use crate::CratePaths;

pub use component_shape_codegen::validate_shape_field_suffix;

#[derive(Clone, Debug)]
pub struct ComponentFieldIr {
    pub shape: ResolvedComponentShape,
    pub field_ident: syn::Ident,
    pub storage_policy: ComponentShapeStoragePolicy,
}

#[derive(Clone, Debug)]
pub struct ComponentShapeStoragePolicy {
    shape: syn::Path,
}

impl ComponentShapeStoragePolicy {
    pub fn shape_owned(shape: syn::Path) -> Self {
        Self { shape }
    }

    pub fn shape(&self) -> &syn::Path {
        &self.shape
    }
}

#[derive(Clone, Debug)]
pub struct ShapeOptions {
    inner: SharedShapeOptions,
}

impl ShapeOptions {
    pub fn from_shape(shape: syn::Path) -> Self {
        Self {
            inner: SharedShapeOptions::from_shape(shape),
        }
    }

    pub fn from_shape_with_span(shape: syn::Path, span: Span) -> Self {
        Self {
            inner: SharedShapeOptions::from_shape_with_span(shape, span),
        }
    }

    pub fn resolve(&self, field_name: String, field_type: syn::Type) -> ResolvedComponentShape {
        ResolvedComponentShape {
            inner: self.inner.resolve(field_name, field_type),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResolvedComponentShape {
    inner: SharedResolvedComponentShape,
}

impl ResolvedComponentShape {
    pub fn shape(&self) -> &syn::Path {
        self.inner.shape()
    }

    pub fn component_suffix(&self) -> &str {
        self.inner.component_suffix()
    }

    pub fn constructor_tokens(&self) -> TokenStream {
        let shape = self.shape();
        let crate_paths = CratePaths::resolve();
        let runtime_crate = crate_paths.gpui_form_facade_runtime();

        quote! {
            <#shape as #runtime_crate::shape::GpuiComponentShape>::new(window, cx)
        }
    }

    pub fn storage_policy(&self) -> ComponentShapeStoragePolicy {
        ComponentShapeStoragePolicy::shape_owned(self.inner.shape().clone())
    }

    pub fn type_check_tokens(&self) -> TokenStream {
        let shape = self.shape();
        let field_type = self.inner.field_type();
        let span = self.inner.span();
        let crate_paths = CratePaths::resolve();
        let runtime_crate = tokens_with_span(&crate_paths.gpui_form_facade_runtime(), span);
        let field_fragment = field_assertion_ident_fragment(self.inner.field_name());
        let declared_shape_assertion = format_ident!(
            "__gpui_form_assert_{field_fragment}_declared_component_shape",
            span = span
        );
        let value_compat_assertion = format_ident!(
            "__gpui_form_assert_{field_fragment}_component_value_compatibility",
            span = span
        );
        let value_binding_assertion = format_ident!(
            "__gpui_form_assert_{field_fragment}_component_value_binding",
            span = span
        );

        quote_spanned! {span=>
            {
                fn #declared_shape_assertion<
                    Shape: #runtime_crate::shape::DeclaredGpuiComponentShape
                        + #runtime_crate::shape::GpuiComponentShape
                        + #runtime_crate::shape::GpuiFormComponentShapePolicy,
                >() {}
                #declared_shape_assertion::<#shape>();

                fn #value_compat_assertion<
                    Shape: #runtime_crate::shape::GpuiComponentShapeFor<Value>,
                    Value,
                >() {}
                #value_compat_assertion::<#shape, #field_type>();

                fn #value_binding_assertion<
                    Shape: #runtime_crate::shape::GpuiComponentShape
                        + #runtime_crate::shape::ComponentShapeMetadata,
                    Value,
                >() {}
                #value_binding_assertion::<#shape, #field_type>();
            }
        }
    }

    pub fn field_ir(&self) -> syn::Result<ComponentFieldIr> {
        let field_ident = crate::names::ComponentEntityFieldName::try_new_spanned(
            self.inner.field_name(),
            self.inner.span(),
        )?;
        Ok(ComponentFieldIr {
            shape: self.clone(),
            field_ident: field_ident.0,
            storage_policy: self.storage_policy(),
        })
    }

    pub fn component_variant_tokens(&self) -> TokenStream {
        let schema_crate = CratePaths::resolve().gpui_form;
        let shape = rust_path_metadata_tokens(
            quote! { #schema_crate::schema::registry::RustPath },
            self.shape(),
        );
        let capabilities_tokens = self.capabilities_tokens();
        let prototyping_tokens = self.prototyping_tokens();

        quote! {
            #schema_crate::schema::registry::FieldComponentVariant::new(
                #shape
            )
            .with_capabilities(#capabilities_tokens)
            #prototyping_tokens
        }
    }

    fn capabilities_tokens(&self) -> TokenStream {
        let shape = self.shape();
        let crate_paths = CratePaths::resolve();
        let runtime_crate = crate_paths.gpui_form_facade_runtime();
        let schema_crate = crate_paths.gpui_form;

        quote! {
            #schema_crate::schema::registry::ComponentCapabilities::new()
                .with_render(
                    if <<#shape as #runtime_crate::shape::GpuiComponentShape>::RenderComponent
                        as #runtime_crate::shape::GpuiComponentRender<
                            <#shape as #runtime_crate::shape::GpuiComponentShape>::State
                        >>::RENDERS
                    {
                        #schema_crate::schema::registry::RenderCapability::Component
                    } else {
                        #schema_crate::schema::registry::RenderCapability::None
                    }
                )
                .with_value_binding(
                    if <#shape as #runtime_crate::shape::ComponentShapeMetadata>::CAPABILITIES
                        .value_binding()
                        .enabled()
                    {
                        #schema_crate::schema::registry::ValueBindingCapability::Inherited
                    } else {
                        #schema_crate::schema::registry::ValueBindingCapability::None
                    }
                )
        }
    }

    fn prototyping_tokens(&self) -> TokenStream {
        let shape = self.shape();
        let suffix = self.component_suffix();
        let crate_paths = CratePaths::resolve();
        let runtime_crate = crate_paths.gpui_form_facade_runtime();
        let schema_crate = crate_paths.gpui_form;
        quote! {
            .with_prototyping_field_suffix(
                match <#shape as #runtime_crate::shape::ComponentShapeMetadata>::PROTOTYPING.field_suffix {
                    Some(suffix) => Some(suffix),
                    None => Some(
                        #schema_crate::schema::registry::ComponentSuffix::new(#suffix)
                    ),
                }
            )
        }
    }
}

impl ShapeOptions {
    pub fn field_ir(
        &self,
        field_name: String,
        field_type: syn::Type,
        _field_default: Option<syn::Expr>,
    ) -> syn::Result<ComponentFieldIr> {
        self.resolve(field_name, field_type).field_ir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens as _;

    fn compact_path(path: &syn::Path) -> String {
        path.to_token_stream()
            .to_string()
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect()
    }

    #[test]
    fn component_field_ir_plans_identifier_shape_and_storage_policy() {
        let options = ShapeOptions::from_shape(syn::parse_quote!(crate::Input<_>));
        let field_type: syn::Type = syn::parse_quote!(String);

        let ir = options
            .field_ir("email".to_string(), field_type, None)
            .expect("valid component field should plan semantic IR");

        assert_eq!(ir.field_ident.to_string(), "email");
        assert_eq!(compact_path(ir.shape.shape()), "crate::Input<String>");
        assert_eq!(
            compact_path(ir.storage_policy.shape()),
            "crate::Input<String>"
        );
    }
}
