use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Expr, LitStr, Path, Result, Token, parse_macro_input};

use super::component_shape_constructor::constructor_body_tokens;

#[derive(Debug, Default)]
struct ComponentShapeMeta {
    new: Option<Expr>,
    /// Optional UI component type path.
    /// When set, `ComponentShape::COMPONENT_PATH` is populated so that
    /// field annotations do not need to repeat `component = …`.
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

fn set_once<T>(
    slot: &mut Option<T>,
    value: T,
    meta: &syn::meta::ParseNestedMeta<'_>,
    option: &str,
) -> Result<()> {
    if slot.replace(value).is_some() {
        return Err(meta.error(format!("duplicate `{option}` option")));
    }

    Ok(())
}

fn parse_meta(attrs: &[syn::Attribute]) -> Result<ComponentShapeMeta> {
    let mut shape = ComponentShapeMeta::default();

    for attr in attrs
        .iter()
        .filter(|attr| attr.path().is_ident("gpui_form_shape"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("new") {
                let value = meta.value()?;
                let new = value.parse()?;
                set_once(&mut shape.new, new, &meta, "new")
            } else if meta.path.is_ident("component") {
                let value = meta.value()?;
                let component = value.parse()?;
                set_once(&mut shape.component, component, &meta, "component")
            } else if meta.path.is_ident("value_binding") {
                if meta.input.peek(Token![=]) {
                    return Err(meta.error("`value_binding` does not take a value"));
                }
                if shape.value_binding {
                    return Err(meta.error("duplicate `value_binding` option"));
                }
                shape.value_binding = true;
                Ok(())
            } else if meta.path.is_ident("field_suffix") {
                let value = meta.value()?;
                let field_suffix = value.parse()?;
                set_once(&mut shape.field_suffix, field_suffix, &meta, "field_suffix")
            } else if meta.path.is_ident("shape_crate") {
                let value = meta.value()?;
                let shape_crate = value.parse()?;
                set_once(&mut shape.shape_crate, shape_crate, &meta, "shape_crate")
            } else {
                Err(meta.error(
                    "unsupported `gpui_form_shape` option; expected `new`, `component`, `value_binding`, `field_suffix`, or `shape_crate`",
                ))
            }
        })?;
    }

    Ok(shape)
}

fn expand(input: DeriveInput) -> Result<TokenStream> {
    let ident = &input.ident;
    let meta = parse_meta(&input.attrs)?;
    let constructor_body = meta
        .new
        .as_ref()
        .map(constructor_body_tokens)
        .unwrap_or_else(|| quote! { Self::new(window, cx) });
    let shape_crate = meta
        .shape_crate
        .unwrap_or_else(|| syn::parse_quote!(::gpui_form_component));
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let component_path_const = if let Some(comp) = meta.component {
        quote! {
            const COMPONENT_PATH: Option<&'static str> = Some(stringify!(#comp));
        }
    } else {
        quote! {}
    };
    let value_binding_const = if meta.value_binding {
        quote! {
            const VALUE_BINDING: bool = true;
        }
    } else {
        quote! {}
    };
    let prototyping_const = if let Some(field_suffix) = meta.field_suffix {
        quote! {
            const PROTOTYPING: #shape_crate::shape::ComponentPrototyping =
                #shape_crate::shape::ComponentPrototyping::new()
                    .field_suffix(#field_suffix);
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        impl #impl_generics #shape_crate::shape::ComponentShape for #ident #ty_generics #where_clause {
            type State = Self;

            fn new(
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::Context<'_, Self::State>,
            ) -> Self::State {
                #constructor_body
            }

            #component_path_const
            #value_binding_const
            #prototyping_const
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
            compact.contains("impl::gpui_form_component::shape::ComponentShapeforTagsState"),
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
    fn test_component_shape_with_shape_crate_path() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(new = Self::new, shape_crate = crate)]
            struct TagsState;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("implcrate::shape::ComponentShapeforTagsState"),
            "should allow runtime crates to implement their local shape trait path"
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
