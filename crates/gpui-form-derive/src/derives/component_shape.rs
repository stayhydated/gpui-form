use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{
    Attribute, Expr, GenericParam, Generics, Ident, Path, Result, Token, Type, Visibility, braced,
    parse_macro_input,
};

use super::component_shape_constructor::constructor_body_tokens;

mod kw {
    syn::custom_keyword!(component);
    syn::custom_keyword!(field_suffix);
    syn::custom_keyword!(new);
    syn::custom_keyword!(shape_crate);
    syn::custom_keyword!(value_binding);
}

struct ComponentShapeInput {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    state: Type,
    new: Option<Expr>,
    component: Option<Path>,
    value_binding: bool,
    field_suffix: Option<syn::LitStr>,
    shape_crate: Option<Path>,
}

impl Parse for ComponentShapeInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        input.parse::<Token![struct]>()?;
        let ident = input.parse()?;
        let mut generics: Generics = input.parse()?;
        generics.where_clause = input.parse()?;

        let content;
        braced!(content in input);

        let mut state = None;
        let mut new = None;
        let mut component = None;
        let mut value_binding = false;
        let mut field_suffix = None;
        let mut shape_crate = None;

        while !content.is_empty() {
            if content.peek(Token![type]) {
                content.parse::<Token![type]>()?;
                let type_ident: Ident = content.parse()?;
                if type_ident != "State" {
                    return Err(syn::Error::new_spanned(
                        type_ident,
                        "expected `type State = ...;`",
                    ));
                }
                content.parse::<Token![=]>()?;
                if state.replace(content.parse()?).is_some() {
                    return Err(syn::Error::new_spanned(
                        type_ident,
                        "duplicate `type State = ...;`",
                    ));
                }
                content.parse::<Token![;]>()?;
            } else if content.peek(kw::new) {
                let key = content.parse::<kw::new>()?;
                content.parse::<Token![=]>()?;
                if new.replace(content.parse()?).is_some() {
                    return Err(syn::Error::new_spanned(key, "duplicate `new = ...;`"));
                }
                content.parse::<Token![;]>()?;
            } else if content.peek(kw::component) {
                let key = content.parse::<kw::component>()?;
                content.parse::<Token![=]>()?;
                if component.replace(content.parse()?).is_some() {
                    return Err(syn::Error::new_spanned(key, "duplicate `component = ...;`"));
                }
                content.parse::<Token![;]>()?;
            } else if content.peek(kw::value_binding) {
                let key = content.parse::<kw::value_binding>()?;
                if value_binding {
                    return Err(syn::Error::new_spanned(key, "duplicate `value_binding;`"));
                }
                value_binding = true;
                content.parse::<Token![;]>()?;
            } else if content.peek(kw::field_suffix) {
                let key = content.parse::<kw::field_suffix>()?;
                content.parse::<Token![=]>()?;
                if field_suffix.replace(content.parse()?).is_some() {
                    return Err(syn::Error::new_spanned(
                        key,
                        "duplicate `field_suffix = ...;`",
                    ));
                }
                content.parse::<Token![;]>()?;
            } else if content.peek(kw::shape_crate) {
                let key = content.parse::<kw::shape_crate>()?;
                content.parse::<Token![=]>()?;
                if shape_crate.replace(content.parse()?).is_some() {
                    return Err(syn::Error::new_spanned(
                        key,
                        "duplicate `shape_crate = ...;`",
                    ));
                }
                content.parse::<Token![;]>()?;
            } else {
                return Err(content.error(
                    "expected `type State = ...;`, `new = ...;`, `component = ...;`, `value_binding;`, `field_suffix = ...;`, or `shape_crate = ...;`",
                ));
            }
        }

        Ok(Self {
            attrs,
            vis,
            ident,
            generics,
            state: state.ok_or_else(|| input.error("missing `type State = ...;`"))?,
            new,
            component,
            value_binding,
            field_suffix,
            shape_crate,
        })
    }
}

fn phantom_type_tokens(generics: &Generics) -> TokenStream {
    let params: Vec<TokenStream> = generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(param) => {
                let ident = &param.ident;
                Some(quote! { #ident })
            },
            GenericParam::Lifetime(param) => {
                let lifetime = &param.lifetime;
                Some(quote! { &#lifetime () })
            },
            GenericParam::Const(_) => None,
        })
        .collect();

    if params.is_empty() {
        quote! { () }
    } else {
        quote! { (#(#params),*) }
    }
}

fn expand(input: ComponentShapeInput) -> TokenStream {
    let ComponentShapeInput {
        attrs,
        vis,
        ident,
        generics,
        state,
        new,
        component,
        value_binding,
        field_suffix,
        shape_crate,
    } = input;

    let phantom_type = phantom_type_tokens(&generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let shape_crate = shape_crate.unwrap_or_else(|| syn::parse_quote!(::gpui_form_component));
    let constructor_body = new
        .as_ref()
        .map(constructor_body_tokens)
        .unwrap_or_else(|| quote! { <#state>::new(window, cx) });
    let component_path_const = component.map(|component| {
        quote! {
            const COMPONENT_PATH: Option<&'static str> = Some(stringify!(#component));
        }
    });
    let value_binding_const = value_binding.then(|| {
        quote! {
            const VALUE_BINDING: bool = true;
        }
    });
    let prototyping_const = field_suffix.map(|field_suffix| {
        quote! {
            const PROTOTYPING: #shape_crate::shape::ComponentPrototyping =
                #shape_crate::shape::ComponentPrototyping::new()
                    .field_suffix(#field_suffix);
        }
    });
    quote! {
        #(#attrs)*
        #vis struct #ident #generics(
            ::core::marker::PhantomData<fn() -> #phantom_type>
        ) #where_clause;

        impl #impl_generics #shape_crate::shape::ComponentShape for #ident #ty_generics #where_clause {
            type State = #state;

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
    }
}

pub fn function(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ComponentShapeInput);
    expand(input).into()
}

#[cfg(test)]
mod tests {
    use super::{ComponentShapeInput, expand};
    use quote::quote;

    fn compact_tokens(tokens: &str) -> String {
        tokens.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn component_shape_function_macro_emits_contract_impl() {
        let input: ComponentShapeInput = syn::parse2(quote! {
            pub struct InputShape<T>
            where
                T: ::std::str::FromStr + ::std::string::ToString + 'static,
            {
                type State = ::gpui_component::input::InputState;
                new = |window, cx| ::gpui_component::input::InputState::new(window, cx);
                component = ::gpui_component::input::Input;
                value_binding;
                field_suffix = "input";
            }
        })
        .unwrap();

        let expanded = expand(input);
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("pubstructInputShape<T>(::core::marker::PhantomData<fn()->(T)>)"),
            "generic shape type should carry PhantomData for external component wrappers: {compact}"
        );
        assert!(
            compact.contains("impl<T>::gpui_form_component::shape::ComponentShapeforInputShape<T>whereT:::std::str::FromStr+::std::string::ToString+'static"),
            "macro should implement the shape contract with caller generics and where clause: {compact}"
        );
        assert!(
            compact.contains("typeState=::gpui_component::input::InputState"),
            "macro should emit the configured state type: {compact}"
        );
        assert!(
            compact.contains("COMPONENT_PATH:Option<&'staticstr>=Some(stringify!(::gpui_component::input::Input))"),
            "macro should embed component metadata: {compact}"
        );
        assert!(
            compact.contains("VALUE_BINDING:bool=true"),
            "macro should emit value-binding metadata: {compact}"
        );
        assert!(
            compact.contains("ComponentPrototyping::new().field_suffix(\"input\")"),
            "macro should emit prototyping field suffix metadata: {compact}"
        );
    }

    #[test]
    fn component_shape_function_macro_accepts_shape_crate_path() {
        let input: ComponentShapeInput = syn::parse2(quote! {
            pub struct LocalInputShape {
                type State = crate::state::InputState;
                new = crate::state::InputState::new;
                shape_crate = crate;
            }
        })
        .unwrap();

        let expanded = expand(input);
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("implcrate::shape::ComponentShapeforLocalInputShape"),
            "macro should allow runtime crates to target their local shape trait path: {compact}"
        );
    }

    #[test]
    fn component_shape_function_macro_defaults_to_state_new() {
        let input: ComponentShapeInput = syn::parse2(quote! {
            pub struct LocalInputShape {
                type State = crate::state::InputState;
            }
        })
        .unwrap();

        let expanded = expand(input);
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("<crate::state::InputState>::new(window,cx)"),
            "macro should default omitted constructors to State::new: {compact}"
        );
    }

    #[test]
    fn component_shape_function_macro_accepts_direct_constructor_call() {
        let input: ComponentShapeInput = syn::parse2(quote! {
            pub struct LocalInputShape {
                type State = crate::state::InputState;
                new = crate::state::InputState::new(window, cx).with_label("email");
            }
        })
        .unwrap();

        let expanded = expand(input);
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("crate::state::InputState::new(window,cx).with_label(\"email\")"),
            "direct constructor expressions should be emitted as written: {compact}"
        );
        assert!(
            !compact.contains(
                "(crate::state::InputState::new(window,cx).with_label(\"email\"))(window,cx)"
            ),
            "direct constructor expressions should not receive window/cx twice: {compact}"
        );
    }

    #[test]
    fn component_shape_function_macro_rejects_duplicate_options() {
        let err = match syn::parse2::<ComponentShapeInput>(quote! {
            pub struct InputShape {
                type State = crate::state::InputState;
                type State = crate::state::OtherState;
                new = crate::state::InputState::new;
            }
        }) {
            Ok(_) => panic!("component_shape! should reject duplicate options"),
            Err(err) => err,
        };

        assert!(
            err.to_string().contains("duplicate `type State = ...;`"),
            "macro should report duplicate options clearly: {err}"
        );
    }
}
