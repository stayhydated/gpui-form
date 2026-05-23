use darling::{FromAttributes, util::Flag};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Path, parse_macro_input};

#[derive(Debug, Default, FromAttributes)]
#[darling(attributes(gpui_form_custom))]
struct CustomComponentStateMeta {
    #[darling(default)]
    new: Option<Path>,
    /// Optional UI component type path.
    /// When set, `CustomComponentShape::COMPONENT_PATH` is populated so that
    /// field annotations do not need to repeat `component = …`.
    #[darling(default)]
    component: Option<Path>,
    /// Opt generated prototyping code into CustomComponentValueAdapter by default.
    #[darling(default)]
    value_binding: Flag,
    /// Runtime crate path that owns `custom::CustomComponentShape`.
    ///
    /// This defaults to the public `gpui_form` facade for downstream users.
    /// Runtime crates that own a state type can set `custom_crate = crate`.
    #[darling(default)]
    custom_crate: Option<Path>,
}

fn parse_meta(attrs: &[syn::Attribute]) -> darling::Result<CustomComponentStateMeta> {
    CustomComponentStateMeta::from_attributes(attrs)
}

fn expand(input: DeriveInput) -> darling::Result<TokenStream> {
    let ident = &input.ident;
    let meta = parse_meta(&input.attrs)?;
    let new_path = meta.new.unwrap_or_else(|| syn::parse_quote!(Self::new));
    let custom_crate = meta
        .custom_crate
        .unwrap_or_else(|| syn::parse_quote!(::gpui_form));
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

    Ok(quote! {
        impl #impl_generics #custom_crate::custom::CustomComponentShape for #ident #ty_generics #where_clause {
            type State = Self;

            fn new(
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::Context<'_, Self::State>,
            ) -> Self::State {
                #new_path(window, cx)
            }

            #component_path_const
            #value_binding_const
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
    fn test_custom_component_state_default_new_path() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(CustomComponentState)]
            struct TagsState;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("impl::gpui_form::custom::CustomComponentShapeforTagsState"),
            "should implement CustomComponentShape for derived type"
        );
        assert!(
            compact.contains("Self::new(window,cx)"),
            "should default to Self::new constructor"
        );
    }

    #[test]
    fn test_custom_component_state_explicit_new_path() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(CustomComponentState)]
            #[gpui_form_custom(new = crate::state::build)]
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
    fn test_custom_component_state_with_component_path() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(CustomComponentState)]
            #[gpui_form_custom(new = Self::new, component = crate::ui::TagsInput)]
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
    fn test_custom_component_state_with_value_binding() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(CustomComponentState)]
            #[gpui_form_custom(new = Self::new, value_binding)]
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
    fn test_custom_component_state_with_custom_crate_path() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(CustomComponentState)]
            #[gpui_form_custom(new = Self::new, custom_crate = crate)]
            struct TagsState;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("implcrate::custom::CustomComponentShapeforTagsState"),
            "should allow runtime crates to implement their local custom trait path"
        );
    }
}
