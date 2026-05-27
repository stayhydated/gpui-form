use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitBool, Result, Token, parse_macro_input};

use super::component_shape_metadata::{ComponentShapeMetadata, SHAPE_METADATA_OPTIONS};

fn parse_meta(attrs: &[syn::Attribute]) -> Result<ComponentShapeMetadata> {
    let mut shape = ComponentShapeMetadata::default();

    for attr in attrs
        .iter()
        .filter(|attr| attr.path().is_ident("gpui_form_shape"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("new") {
                let value = meta.value()?;
                let new = value.parse()?;
                shape.set_new(new, &meta.path)
            } else if meta.path.is_ident("component") {
                let value = meta.value()?;
                let component = value.parse()?;
                shape.set_component(component, &meta.path)
            } else if meta.path.is_ident("value_binding") {
                if meta.input.peek(Token![=]) {
                    let value = meta.value()?;
                    let value = value.parse::<LitBool>()?;
                    return Err(ComponentShapeMetadata::reject_value_binding_assignment(
                        value,
                    ));
                }
                shape.enable_value_binding(&meta.path)
            } else if meta.path.is_ident("field_suffix") {
                let value = meta.value()?;
                let field_suffix = value.parse()?;
                shape.set_field_suffix(field_suffix, &meta.path)
            } else {
                Err(meta.error(format!(
                    "unsupported `gpui_form_shape` option; expected {SHAPE_METADATA_OPTIONS}",
                )))
            }
        })?;
    }

    Ok(shape)
}

fn expand(input: DeriveInput) -> Result<TokenStream> {
    let ident = &input.ident;
    let meta = parse_meta(&input.attrs)?;
    let constructor_body = meta.constructor_body_or(quote! { Self::new(window, cx) });
    let runtime_crate = ComponentShapeMetadata::runtime_crate_path();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let metadata_consts = meta.const_tokens(&runtime_crate);

    Ok(quote! {
        impl #impl_generics #runtime_crate::shape::ComponentShape for #ident #ty_generics #where_clause {
            type State = Self;

            fn new(
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::Context<'_, Self::State>,
            ) -> Self::State {
                #constructor_body
            }

            #metadata_consts
        }
    })
}

pub fn from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[cfg(test)]
mod tests {
    use super::expand;
    use quote::quote;
    use syn::DeriveInput;

    fn compact_tokens(tokens: &str) -> String {
        tokens.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn test_component_shape_default_new_path() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            struct TagsState;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("impl::gpui_form_runtime::shape::ComponentShapeforTagsState"),
            "should implement ComponentShape for derived type"
        );
        assert!(
            compact.contains("Self::new(window,cx)"),
            "should default to Self::new constructor"
        );
    }

    #[test]
    fn test_component_shape_explicit_new_path() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(new = crate::state::build)]
            struct TagsState;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("(crate::state::build)(window,cx)"),
            "should use explicit new path from attribute"
        );
        assert!(
            !compact.contains("COMPONENT_PATH"),
            "should not emit COMPONENT_PATH when component is not specified"
        );
    }

    #[test]
    fn test_component_shape_with_component_path() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(new = Self::new, component = crate::ui::TagsInput)]
            struct TagsState;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("COMPONENT_PATH"),
            "should emit COMPONENT_PATH const when component is specified"
        );
        assert!(
            compact.contains("crate::ui::TagsInput"),
            "should embed the component path as a string"
        );
    }

    #[test]
    fn test_component_shape_with_constructor_expression() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(new = |window, cx| Self::with_mode(window, cx, Mode::Tags))]
            struct TagsState;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("(|window,cx|Self::with_mode(window,cx,Mode::Tags))(window,cx)"),
            "should allow derive constructors to be full expressions"
        );
    }

    #[test]
    fn test_component_shape_with_direct_constructor_call() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(new = Self::with_mode(window, cx, Mode::Tags))]
            struct TagsState;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("Self::with_mode(window,cx,Mode::Tags)"),
            "direct constructor calls should be emitted as written: {compact}"
        );
        assert!(
            !compact.contains("(Self::with_mode(window,cx,Mode::Tags))(window,cx)"),
            "direct constructor calls should not receive window/cx twice: {compact}"
        );
    }

    #[test]
    fn test_component_shape_with_value_binding() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(new = Self::new, value_binding)]
            struct TagsState;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("VALUE_BINDING:bool=true"),
            "should emit VALUE_BINDING const when value_binding is specified"
        );
    }

    #[test]
    fn test_component_shape_rejects_value_binding_bool() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(new = Self::new, value_binding = true)]
            struct TagsState;
        })
        .unwrap();

        let err = expand(input).unwrap_err();

        assert!(
            err.to_string().contains("`value_binding` is a flag"),
            "should explain the concise value_binding syntax: {err}"
        );
    }

    #[test]
    fn test_component_shape_rejects_disabled_value_binding_bool() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(new = Self::new, value_binding = false)]
            struct TagsState;
        })
        .unwrap();

        let err = expand(input).unwrap_err();

        assert!(
            err.to_string().contains("`value_binding` is a flag"),
            "should explain that disabling is done by omitting the flag: {err}"
        );
    }

    #[test]
    fn test_component_shape_with_field_suffix() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(new = Self::new, field_suffix = "tags")]
            struct TagsState;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("ComponentPrototyping::new().field_suffix(\"tags\")"),
            "should emit prototyping field suffix metadata"
        );
    }

    #[test]
    fn test_component_shape_rejects_duplicate_options() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(new = Self::new, new = crate::state::build)]
            struct TagsState;
        })
        .unwrap();

        let err = expand(input).unwrap_err();

        assert!(
            err.to_string().contains("duplicate `new` option"),
            "should report the duplicated option name: {err}"
        );
    }
}
