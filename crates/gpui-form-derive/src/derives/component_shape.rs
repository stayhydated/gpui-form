use gpui_form_codegen::CratePaths;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{
    Attribute, GenericParam, Generics, Ident, ItemImpl, Result, Token, Type, Visibility, braced,
    parse_macro_input,
};

use super::component_shape_metadata::ComponentShapeMetadata;

const FUNCTION_SHAPE_OPTIONS: &str =
    "`new = ...`, `component = ...`, `requires_value = ...`, or `field_suffix = ...`";

mod kw {
    syn::custom_keyword!(component);
    syn::custom_keyword!(field_suffix);
    syn::custom_keyword!(new);
    syn::custom_keyword!(requires_value);
}

struct ComponentShapeInput {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    state: Type,
    metadata: ComponentShapeMetadata,
    impls: Vec<ItemImpl>,
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
        let mut metadata = ComponentShapeMetadata::default();
        let mut impls = Vec::new();

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
                parse_option_separator(&content)?;
            } else if content.peek(kw::new) {
                let key = content.parse::<kw::new>()?;
                content.parse::<Token![=]>()?;
                metadata.set_new(content.parse()?, key)?;
                parse_option_separator(&content)?;
            } else if content.peek(kw::component) {
                let key = content.parse::<kw::component>()?;
                content.parse::<Token![=]>()?;
                metadata.set_component(content.parse()?, key)?;
                parse_option_separator(&content)?;
            } else if content.peek(kw::requires_value) {
                let key = content.parse::<kw::requires_value>()?;
                content.parse::<Token![=]>()?;
                metadata.set_requires_value(content.parse()?, key)?;
                parse_option_separator(&content)?;
            } else if content.peek(kw::field_suffix) {
                let key = content.parse::<kw::field_suffix>()?;
                content.parse::<Token![=]>()?;
                metadata.set_field_suffix(content.parse()?, key)?;
                parse_option_separator(&content)?;
            } else if content.peek(Token![impl]) || content.peek(Token![#]) {
                let impl_item: ItemImpl = content.parse()?;
                if is_component_value_binding_impl(&impl_item) && !metadata.has_value_binding() {
                    metadata.enable_value_binding(&impl_item)?;
                }
                impls.push(impl_item);
            } else {
                return Err(content.error(format!(
                    "expected `type State = ...;`, an `impl` item, or {FUNCTION_SHAPE_OPTIONS}"
                )));
            }
        }

        Ok(Self {
            attrs,
            vis,
            ident,
            generics,
            state: state.ok_or_else(|| input.error("missing `type State = ...;`"))?,
            metadata,
            impls,
        })
    }
}

fn is_component_value_binding_impl(impl_item: &ItemImpl) -> bool {
    impl_item
        .trait_
        .as_ref()
        .and_then(|(_, path, _)| path.segments.last())
        .is_some_and(|segment| segment.ident == "ComponentValueBinding")
}

fn parse_option_separator(input: ParseStream<'_>) -> Result<()> {
    if input.peek(Token![;]) {
        input.parse::<Token![;]>()?;
        Ok(())
    } else {
        Err(input.error("expected `;` after component shape option"))
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
        metadata,
        impls,
    } = input;

    let phantom_type = phantom_type_tokens(&generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let runtime_crate = ComponentShapeMetadata::runtime_crate_path();
    let gpui_crate = CratePaths::resolve().gpui;
    let constructor_body = metadata.constructor_body_or(quote! { <#state>::new(window, cx) });
    let metadata_impl_items = metadata.impl_items_tokens(&runtime_crate);
    quote! {
        #(#attrs)*
        #vis struct #ident #generics(
            ::core::marker::PhantomData<fn() -> #phantom_type>
        ) #where_clause;

        impl #impl_generics #runtime_crate::shape::ComponentShape for #ident #ty_generics #where_clause {
            type State = #state;

            fn new(
                window: &mut #gpui_crate::Window,
                cx: &mut #gpui_crate::Context<'_, Self::State>,
            ) -> Self::State {
                #constructor_body
            }

            #metadata_impl_items
        }

        #(#impls)*
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
            compact.contains("impl<T>::gpui_form_runtime::shape::ComponentShapeforInputShape<T>whereT:::std::str::FromStr+::std::string::ToString+'static"),
            "macro should implement the shape contract with caller generics and where clause: {compact}"
        );
        assert!(
            compact.contains("typeState=::gpui_component::input::InputState"),
            "macro should emit the configured state type: {compact}"
        );
        assert!(
            compact.contains("typeRequiredValuePolicy=::gpui_form_runtime::shape::RequireValue"),
            "macro should default required-value policy to RequireValue: {compact}"
        );
        assert!(
            compact.contains("COMPONENT_TYPE:Option<&'staticstr>=Some(stringify!(::gpui_component::input::Input))"),
            "macro should embed component metadata: {compact}"
        );
        assert!(
            compact.contains("ComponentPrototyping::new().field_suffix(\"input\")"),
            "macro should emit prototyping field suffix metadata: {compact}"
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
    fn component_shape_function_macro_accepts_required_value_policy() {
        let input: ComponentShapeInput = syn::parse2(quote! {
            pub struct SwitchShape {
                type State = crate::state::SwitchState;
                requires_value = false;
            }
        })
        .unwrap();

        let expanded = expand(input);
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact
                .contains("typeRequiredValuePolicy=::gpui_form_runtime::shape::AllowMissingValue"),
            "macro should emit shape-owned missing-value policy: {compact}"
        );
    }

    #[test]
    fn component_shape_function_macro_emits_nested_value_binding_impl() {
        let input: ComponentShapeInput = syn::parse2(quote! {
            pub struct InputShape<T>
            where
                T: ::std::str::FromStr + ::std::string::ToString + 'static,
            {
                type State = ::gpui_component::input::InputState;
                component = ::gpui_component::input::Input;

                impl<T> ::gpui_form_runtime::shape::ComponentValueBinding<T> for InputShape<T>
                where
                    T: ::std::str::FromStr + ::std::string::ToString + 'static,
                {
                    type Event = ::gpui_component::input::InputEvent;

                    fn seed_value_binding_state(
                        _state: &mut Self::State,
                        _value: Option<&T>,
                        _window: &mut ::gpui::Window,
                        _cx: &mut ::gpui::Context<'_, Self::State>,
                    ) {
                    }

                    fn form_value_change(
                        _state: &Self::State,
                        _event: &Self::Event,
                    ) -> ::gpui_form_runtime::shape::FormValueChange<T> {
                        ::gpui_form_runtime::shape::FormValueChange::Unchanged
                    }
                }
            }
        })
        .unwrap();

        let expanded = expand(input);
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("VALUE_BINDING:bool=true"),
            "nested ComponentValueBinding impl should publish shape-level metadata: {compact}"
        );
        assert!(
            compact.contains("impl<T>::gpui_form_runtime::shape::ComponentValueBinding<T>forInputShape<T>whereT:::std::str::FromStr+::std::string::ToString+'static"),
            "macro should emit nested impl items after the shape contract: {compact}"
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
