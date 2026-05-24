use darling::{FromAttributes, util::Flag};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Path, parse_macro_input};

#[derive(Debug, Default, FromAttributes)]
#[darling(attributes(gpui_form_shape))]
struct ComponentShapeMeta {
    #[darling(default)]
    new: Option<Path>,
    /// Optional UI component type path.
    /// When set, `ComponentShape::COMPONENT_PATH` is populated so that
    /// field annotations do not need to repeat `component = …`.
    #[darling(default)]
    component: Option<Path>,
    /// Opt generated prototyping code into ComponentValueBinding by default.
    #[darling(default)]
    value_binding: Flag,
    /// Preferred generated field/helper suffix for prototyping output.
    #[darling(default)]
    field_suffix: Option<String>,
    /// Runtime crate path that owns `shape::ComponentShape`.
    ///
    /// This defaults to the explicit `gpui_form_component` runtime crate for
    /// downstream users.
    /// Runtime crates that own a state type can set `shape_crate = crate`.
    #[darling(default)]
    shape_crate: Option<Path>,
}

fn parse_meta(attrs: &[syn::Attribute]) -> darling::Result<ComponentShapeMeta> {
    ComponentShapeMeta::from_attributes(attrs)
}

fn expand(input: DeriveInput) -> darling::Result<TokenStream> {
    let ident = &input.ident;
    let meta = parse_meta(&input.attrs)?;
    let new_path = meta.new.unwrap_or_else(|| syn::parse_quote!(Self::new));
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
    let value_binding_const = if meta.value_binding.is_present() {
        quote! {
            const VALUE_BINDING: bool = true;
        }
    } else {
        quote! {}
    };
    let prototyping_const = if let Some(field_suffix) = meta.field_suffix {
        let field_suffix = syn::LitStr::new(&field_suffix, proc_macro2::Span::call_site());
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
                #new_path(window, cx)
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
        Err(err) => err.write_errors().into(),
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
            compact.contains("crate::state::build(window,cx)"),
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
}
