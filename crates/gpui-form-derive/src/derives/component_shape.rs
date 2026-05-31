use gpui_form_codegen::CratePaths;
use proc_macro2::TokenStream;
use quote::{ToTokens as _, format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{
    Attribute, GenericParam, Generics, Ident, ItemImpl, Result, Token, Type, Visibility, braced,
    parse_macro_input,
};

use super::component_shape_metadata::{
    ComponentShapeMetadata, FUNCTION_SHAPE_OPTIONS, ShapeOption, kw,
};

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
            } else if is_shape_option_start(&content) {
                let option = content.call(ShapeOption::parse_function)?;
                option.apply(&mut metadata)?;
                parse_option_separator(&content)?;
            } else if content.peek(Token![impl]) || content.peek(Token![#]) {
                let impl_item: ItemImpl = content.parse()?;
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

fn is_shape_option_start(input: ParseStream<'_>) -> bool {
    input.peek(kw::new)
        || input.peek(kw::component)
        || input.peek(kw::value)
        || input.peek(kw::values)
        || input.peek(kw::compatibility)
        || input.peek(kw::value_storage)
        || input.peek(kw::value_binding)
        || input.peek(kw::field_suffix)
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NestedShapeImplKind {
    ComponentValueBinding,
    ComponentShapeFor,
    Other,
}

fn classify_nested_shape_impl(impl_item: &ItemImpl) -> NestedShapeImplKind {
    let Some((_, path, _)) = impl_item.trait_.as_ref() else {
        return NestedShapeImplKind::Other;
    };
    let Some(last) = path.segments.last() else {
        return NestedShapeImplKind::Other;
    };

    if last.ident == "ComponentShapeFor" {
        NestedShapeImplKind::ComponentShapeFor
    } else if last.ident == "ComponentValueBinding" {
        NestedShapeImplKind::ComponentValueBinding
    } else {
        NestedShapeImplKind::Other
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
    let render_component_adapter_ident = format_ident!("__{}RenderComponent", ident);
    let (render_component_assoc, render_component_adapter) =
        match ComponentShapeMetadata::render_component_tokens(
            &gpui_crate,
            &runtime_crate,
            &vis,
            &render_component_adapter_ident,
            &state,
            metadata.component(),
            &generics,
        ) {
            Ok(tokens) => tokens,
            Err(error) => return error.to_compile_error(),
        };
    let metadata_impl_items = metadata.impl_items_tokens(&runtime_crate, render_component_assoc);
    let nested_impl_kinds = impls
        .iter()
        .map(classify_nested_shape_impl)
        .collect::<Vec<_>>();
    if let Some(impl_item) = impls.iter().find(|impl_item| {
        classify_nested_shape_impl(impl_item) == NestedShapeImplKind::ComponentShapeFor
    }) {
        let path = impl_item
            .trait_
            .as_ref()
            .map(|(_, path, _)| path.to_token_stream())
            .expect("component shape compatibility impls always have a trait path");
        return syn::Error::new_spanned(
            path,
            "`component_shape!` no longer accepts manual `ComponentShapeFor` impls inside the macro block; use `value = ...`, `values(...)`, or `compatibility<Value> where ...;` metadata",
        )
        .to_compile_error();
    }
    if !metadata.has_value_compatibility() {
        return syn::Error::new_spanned(
            ident,
            "`component_shape!` requires explicit value compatibility; add `value = ...`, `values(...)`, or `compatibility<Value> where ...;`",
        )
        .to_compile_error();
    }
    if metadata.has_value_binding()
        && !nested_impl_kinds.contains(&NestedShapeImplKind::ComponentValueBinding)
    {
        return syn::Error::new_spanned(
            ident,
            "`value_binding` requires a nested `ComponentValueBinding<T>` impl in the `component_shape!` block",
        )
        .to_compile_error();
    }
    let component_shape_for_impls = metadata.value_impl_tokens(&runtime_crate, &ident, &generics);

    quote! {
        #(#attrs)*
        #vis struct #ident #generics(
            ::core::marker::PhantomData<fn() -> #phantom_type>
        );

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

        #render_component_adapter

        impl #impl_generics #runtime_crate::shape::DeclaredComponentShape for #ident #ty_generics #where_clause {}

        #(#impls)*
        #component_shape_for_impls
    }
}

pub fn function(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ComponentShapeInput);
    expand(input).into()
}

#[cfg(test)]
mod tests {
    use super::{ComponentShapeInput, NestedShapeImplKind, classify_nested_shape_impl, expand};
    use quote::quote;

    fn compact_tokens(tokens: &str) -> String {
        tokens.chars().filter(|c| !c.is_whitespace()).collect()
    }

    fn pretty_tokens(tokens: proc_macro2::TokenStream) -> String {
        syn::parse2::<syn::File>(tokens.clone())
            .map(|file| prettyplease::unparse(&file))
            .unwrap_or_else(|_| tokens.to_string())
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
                value = T;
                field_suffix = "input";
            }
        })
        .unwrap();

        let expanded = expand(input);

        insta::assert_snapshot!(pretty_tokens(expanded));
    }

    #[test]
    fn component_shape_function_macro_defaults_to_state_new() {
        let input: ComponentShapeInput = syn::parse2(quote! {
            pub struct LocalInputShape {
                type State = crate::state::InputState;
                value = String;
            }
        })
        .unwrap();

        let expanded = expand(input);

        insta::assert_snapshot!(pretty_tokens(expanded));
    }

    #[test]
    fn component_shape_function_macro_accepts_multiple_values() {
        let input: ComponentShapeInput = syn::parse2(quote! {
            pub struct SliderShape {
                type State = crate::state::SliderState;
                values(f32, gpui_component::slider::SliderValue);
            }
        })
        .unwrap();

        let expanded = expand(input);
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("ComponentShapeFor<f32>forSliderShape")
                && compact.contains(
                    "ComponentShapeFor<gpui_component::slider::SliderValue>forSliderShape"
                ),
            "values(...) should emit one compatibility impl per value type: {compact}"
        );
    }

    #[test]
    fn component_shape_function_macro_rejects_duplicate_values() {
        let err = match syn::parse2::<ComponentShapeInput>(quote! {
            pub struct SliderShape {
                type State = crate::state::SliderState;
                value = f32;
                values(gpui_component::slider::SliderValue, f32);
            }
        }) {
            Ok(_) => panic!("component_shape! should reject duplicate value metadata"),
            Err(err) => err,
        };

        assert!(
            err.to_string()
                .contains("duplicate `value` compatibility metadata"),
            "macro should report duplicate value metadata clearly: {err}"
        );
    }

    #[test]
    fn component_shape_function_macro_rejects_manual_component_shape_for_with_values() {
        let input: ComponentShapeInput = syn::parse2(quote! {
            pub struct InputShape {
                type State = crate::state::InputState;
                value = String;

                impl gpui_form_runtime::shape::ComponentShapeFor<String> for InputShape {}
            }
        })
        .unwrap();

        let expanded = expand(input).to_string();

        assert!(
            expanded.contains("no longer accepts manual `ComponentShapeFor` impls"),
            "value metadata plus manual ComponentShapeFor impl should emit a compile error: {expanded}"
        );
    }

    #[test]
    fn component_shape_function_macro_rejects_manual_component_shape_for_trait() {
        let input: ComponentShapeInput = syn::parse2(quote! {
            pub struct InputShape {
                type State = crate::state::InputState;

                impl ComponentShapeFor<String> for InputShape {}
            }
        })
        .unwrap();

        let expanded = expand(input).to_string();

        assert!(
            expanded.contains("no longer accepts manual `ComponentShapeFor` impls"),
            "ambiguous ComponentShapeFor impl should emit a targeted compile error: {expanded}"
        );
    }

    #[test]
    fn component_shape_function_macro_emits_constrained_compatibility_impl() {
        let input: ComponentShapeInput = syn::parse2(quote! {
            pub struct Combobox<T>
            where
                T: Clone + 'static,
            {
                type State = crate::state::ComboboxState<T>;
                compatibility<Value>
                where
                    Value: crate::ComboboxFormValue<T>;
            }
        })
        .unwrap();

        let expanded = expand(input);
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains(
                "impl<T,Value>::gpui_form_runtime::shape::ComponentShapeFor<Value>forCombobox<T>"
            ) && compact.contains("Value:crate::ComboboxFormValue<T>"),
            "compatibility<Value> should emit a constrained compatibility impl: {compact}"
        );
    }

    #[test]
    fn component_shape_function_macro_requires_value_compatibility() {
        let input: ComponentShapeInput = syn::parse2(quote! {
            pub struct InputShape {
                type State = crate::state::InputState;
            }
        })
        .unwrap();

        let expanded = expand(input).to_string();

        assert!(
            expanded.contains("requires explicit value compatibility"),
            "missing value compatibility should emit a targeted compile error: {expanded}"
        );
    }

    #[test]
    fn component_shape_function_macro_requires_compatibility_where_clause() {
        let err = match syn::parse2::<ComponentShapeInput>(quote! {
            pub struct InputShape {
                type State = crate::state::InputState;
                compatibility<Value>;
            }
        }) {
            Ok(_) => panic!("component_shape! should reject unconstrained compatibility metadata"),
            Err(err) => err,
        };

        assert!(
            err.to_string().contains("requires a `where` clause"),
            "compatibility<Value> without where should point users to value metadata: {err}"
        );
    }

    #[test]
    fn nested_impl_classification_accepts_value_binding_impl() {
        let impl_item: syn::ItemImpl = syn::parse_quote! {
            impl<T> gpui_form_runtime::shape::ComponentValueBinding<T> for InputShape<T> {
                type Event = crate::InputEvent;
            }
        };

        assert_eq!(
            classify_nested_shape_impl(&impl_item),
            NestedShapeImplKind::ComponentValueBinding
        );
    }

    #[test]
    fn component_shape_function_macro_accepts_direct_constructor_call() {
        let input: ComponentShapeInput = syn::parse2(quote! {
            pub struct LocalInputShape {
                type State = crate::state::InputState;
                new = crate::state::InputState::new(window, cx).with_label("email");
                value = String;
            }
        })
        .unwrap();

        let expanded = expand(input);
        let compact = compact_tokens(&expanded.to_string());

        insta::assert_snapshot!(pretty_tokens(expanded));
        assert!(
            !compact.contains(
                "(crate::state::InputState::new(window,cx).with_label(\"email\"))(window,cx)"
            ),
            "direct constructor expressions should not receive window/cx twice: {compact}"
        );
    }

    #[test]
    fn component_shape_function_macro_accepts_value_storage_policy() {
        let input: ComponentShapeInput = syn::parse2(quote! {
            pub struct SwitchShape {
                type State = crate::state::SwitchState;
                value = bool;
                value_storage = direct;
            }
        })
        .unwrap();

        let expanded = expand(input);

        insta::assert_snapshot!(pretty_tokens(expanded));
    }

    #[test]
    fn component_shape_function_macro_accepts_explicit_require_value_storage() {
        let input: ComponentShapeInput = syn::parse2(quote! {
            pub struct RequiredInputShape {
                type State = crate::state::InputState;
                value = String;
                value_storage = require_value;
            }
        })
        .unwrap();

        let expanded = expand(input);
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains(
                "typeValueStoragePolicy=::gpui_form_runtime::shape::RequiredValueStorage;"
            ),
            "explicit require_value storage should emit RequiredValueStorage policy: {compact}"
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
                value = T;
                value_binding;

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

        insta::assert_snapshot!(pretty_tokens(expanded));
    }

    #[test]
    fn component_shape_function_macro_requires_explicit_value_binding_metadata() {
        let input: ComponentShapeInput = syn::parse2(quote! {
            pub struct InputShape<T> {
                type State = ::gpui_component::input::InputState;
                value = T;

                impl<T> ::gpui_form_runtime::shape::ComponentValueBinding<T> for InputShape<T> {
                    type Event = ::gpui_component::input::InputEvent;

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
            compact.contains(
                "typeValueBindingPolicy=::gpui_form_runtime::shape::NoComponentValueBinding"
            ),
            "nested binding impls should not publish metadata without explicit value_binding: {compact}"
        );
    }

    #[test]
    fn component_shape_function_macro_rejects_value_binding_without_impl() {
        let input: ComponentShapeInput = syn::parse2(quote! {
            pub struct InputShape<T> {
                type State = ::gpui_component::input::InputState;
                value = T;
                value_binding;
            }
        })
        .unwrap();

        let expanded = expand(input).to_string();

        assert!(
            expanded.contains("requires a nested `ComponentValueBinding<T>` impl"),
            "value_binding metadata without a nested binding impl should emit a targeted compile error: {expanded}"
        );
    }

    #[test]
    fn component_shape_function_macro_ignores_unrelated_component_value_binding_names() {
        let input: ComponentShapeInput = syn::parse2(quote! {
            pub struct InputShape<T> {
                type State = ::gpui_component::input::InputState;
                value = T;

                impl<T> unrelated::ComponentValueBinding<T> for InputShape<T> {}
            }
        })
        .unwrap();

        let expanded = expand(input);
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains(
                "typeValueBindingPolicy=::gpui_form_runtime::shape::NoComponentValueBinding"
            ),
            "unrelated traits with a matching final segment should not publish metadata: {compact}"
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

    #[test]
    fn component_shape_function_macro_rejects_non_path_component_type() {
        let err = match syn::parse2::<ComponentShapeInput>(quote! {
            pub struct InputShape {
                type State = crate::state::InputState;
                component = (crate::ui::Input, crate::ui::OtherInput);
            }
        }) {
            Ok(_) => panic!("component_shape! should reject non-path component metadata"),
            Err(err) => err,
        };

        assert!(
            err.to_string()
                .contains("component metadata must be a path-like type"),
            "macro should reject non-path component metadata clearly: {err}"
        );
    }

    #[test]
    fn component_shape_function_macro_rejects_invalid_field_suffix() {
        let err = match syn::parse2::<ComponentShapeInput>(quote! {
            pub struct InputShape {
                type State = crate::state::InputState;
                field_suffix = "";
            }
        }) {
            Ok(_) => panic!("component_shape! should reject invalid field suffix metadata"),
            Err(err) => err,
        };

        assert!(
            err.to_string()
                .contains("`field_suffix` must be a non-empty ASCII identifier suffix"),
            "macro should reject invalid field suffix metadata clearly: {err}"
        );
    }
}
