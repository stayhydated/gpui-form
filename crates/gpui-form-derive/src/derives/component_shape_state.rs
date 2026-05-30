use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Result, Type, parse_macro_input};

use gpui_form_codegen::CratePaths;

use super::component_shape_metadata::{ComponentShapeMetadata, ShapeOption};

fn parse_meta(attrs: &[syn::Attribute]) -> Result<ComponentShapeMetadata> {
    let mut shape = ComponentShapeMetadata::default();

    for attr in attrs
        .iter()
        .filter(|attr| attr.path().is_ident("gpui_form_shape"))
    {
        attr.parse_nested_meta(|meta| ShapeOption::from_nested_meta(&meta)?.apply(&mut shape))?;
    }

    Ok(shape)
}

fn expand(input: DeriveInput) -> Result<TokenStream> {
    let ident = &input.ident;
    let meta = parse_meta(&input.attrs)?;
    let state = meta.state().cloned().ok_or_else(|| {
        syn::Error::new_spanned(
            ident,
            "`#[derive(ComponentShape)]` requires `#[gpui_form_shape(state = ...)]`; \
             use `gpui_form_derive::component_shape!` for wrapper shapes",
        )
    })?;
    let default_constructor = quote! { <#state>::new(window, cx) };
    let constructor_body = meta.constructor_body_or(default_constructor);
    let runtime_crate = ComponentShapeMetadata::runtime_crate_path();
    let gpui_crate = CratePaths::resolve().gpui;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let component_shape_for_impls = meta.value_impl_tokens(&runtime_crate, ident, &input.generics);
    let inferred_component_type: Type = if input.generics.params.is_empty() {
        syn::parse_quote!(#ident)
    } else {
        syn::parse2(quote! { #ident #ty_generics })?
    };
    let render_component = meta.component().unwrap_or(&inferred_component_type);
    let render_component_adapter_ident = format_ident!("__{}RenderComponent", ident);
    let (render_component_assoc, render_component_adapter) =
        ComponentShapeMetadata::render_component_tokens(
            &gpui_crate,
            &runtime_crate,
            &input.vis,
            &render_component_adapter_ident,
            &state,
            Some(render_component),
            &input.generics,
        )?;
    let metadata_impl_items = meta.impl_items_tokens(&runtime_crate, render_component_assoc);
    let binding_impl = if meta.has_value_binding() {
        let mut binding_generics = input.generics.clone();
        binding_generics
            .params
            .push(syn::parse_quote!(__GpuiFormValueBindingValue));
        binding_generics
            .make_where_clause()
            .predicates
            .push(syn::parse_quote! {
                #state: #runtime_crate::shape::ComponentStateValueBinding<__GpuiFormValueBindingValue>
            });
        let (binding_impl_generics, _, binding_where_clause) = binding_generics.split_for_impl();

        Some(quote! {
            impl #binding_impl_generics #runtime_crate::shape::ComponentValueBinding<__GpuiFormValueBindingValue>
                for #ident #ty_generics
                #binding_where_clause
            {
                type Event =
                    <#state as #runtime_crate::shape::ComponentStateValueBinding<
                        __GpuiFormValueBindingValue
                    >>::Event;

                fn seed_value_binding_state(
                    state: &mut Self::State,
                    value: Option<&__GpuiFormValueBindingValue>,
                    window: &mut #gpui_crate::Window,
                    cx: &mut #gpui_crate::Context<'_, Self::State>,
                ) {
                    <#state as #runtime_crate::shape::ComponentStateValueBinding<
                        __GpuiFormValueBindingValue
                    >>::seed_value_binding_state(state, value, window, cx);
                }

                fn form_value_change(
                    state: &Self::State,
                    event: &Self::Event,
                ) -> #runtime_crate::shape::FormValueChange<__GpuiFormValueBindingValue> {
                    <#state as #runtime_crate::shape::ComponentStateValueBinding<
                        __GpuiFormValueBindingValue
                    >>::form_value_change(state, event)
                }
            }
        })
    } else {
        None
    };

    Ok(quote! {
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

        #binding_impl

        #component_shape_for_impls
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

    fn pretty_tokens(tokens: proc_macro2::TokenStream) -> String {
        syn::parse2::<syn::File>(tokens.clone())
            .map(|file| prettyplease::unparse(&file))
            .unwrap_or_else(|_| tokens.to_string())
    }

    #[test]
    fn test_component_shape_requires_state() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            struct TagsInput;
        })
        .unwrap();

        let err = expand(input).unwrap_err();

        assert!(
            err.to_string()
                .contains("requires `#[gpui_form_shape(state = ...)]`"),
            "derive should require explicit backing state: {err}"
        );
    }

    #[test]
    fn test_component_shape_explicit_new_path() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(state = crate::state::TagsState, new = crate::state::build)]
            struct TagsInput;
        })
        .unwrap();

        let expanded = expand(input).unwrap();

        insta::assert_snapshot!(pretty_tokens(expanded));
    }

    #[test]
    fn test_component_shape_with_component_type() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(
                state = crate::state::TagsState,
                new = crate::state::TagsState::new,
                component = crate::ui::TagsInput
            )]
            struct TagsInput;
        })
        .unwrap();

        let expanded = expand(input).unwrap();

        insta::assert_snapshot!(pretty_tokens(expanded));
    }

    #[test]
    fn test_component_shape_with_constructor_expression() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(
                state = crate::state::TagsState,
                new = |window, cx| crate::state::TagsState::with_mode(window, cx, Mode::Tags)
            )]
            struct TagsInput;
        })
        .unwrap();

        let expanded = expand(input).unwrap();

        insta::assert_snapshot!(pretty_tokens(expanded));
    }

    #[test]
    fn test_component_shape_with_direct_constructor_call() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(
                state = crate::state::TagsState,
                new = crate::state::TagsState::with_mode(window, cx, Mode::Tags)
            )]
            struct TagsInput;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        insta::assert_snapshot!(pretty_tokens(expanded));
        assert!(
            !compact
                .contains("(crate::state::TagsState::with_mode(window,cx,Mode::Tags))(window,cx)"),
            "direct constructor calls should not receive window/cx twice: {compact}"
        );
    }

    #[test]
    fn test_component_shape_with_field_suffix() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(
                state = crate::state::TagsState,
                value = Vec<String>,
                field_suffix = "tags"
            )]
            struct TagsInput;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        insta::assert_snapshot!(pretty_tokens(expanded));
        assert!(
            compact.contains("ComponentShapeFor<Vec<String>>forTagsInput"),
            "value = ... should emit explicit compatibility impl: {compact}"
        );
    }

    #[test]
    fn test_component_shape_with_value_storage_policy() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(state = crate::state::SwitchState, value_storage = direct)]
            struct SwitchInput;
        })
        .unwrap();

        let expanded = expand(input).unwrap();

        insta::assert_snapshot!(pretty_tokens(expanded));
    }

    #[test]
    fn test_component_shape_with_explicit_require_value_storage() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(state = crate::state::InputState, value_storage = require_value)]
            struct RequiredInput;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains(
                "typeValueStoragePolicy=::gpui_form_runtime::shape::RequiredValueStorage;"
            ),
            "explicit require_value storage should emit RequiredValueStorage policy: {compact}"
        );
    }

    #[test]
    fn test_component_shape_derive_on_component_with_state() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(state = crate::state::TagsState, field_suffix = "tags")]
            struct TagsInput;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        insta::assert_snapshot!(pretty_tokens(expanded));
        assert!(
            !compact.contains("VALUE_BINDING:bool=true"),
            "component derive should not opt into value binding without explicit metadata: {compact}"
        );
        assert!(
            !compact.contains("ComponentValueBinding<__GpuiFormValueBindingValue>forTagsInput"),
            "component derive should not emit binding delegation without explicit metadata: {compact}"
        );
    }

    #[test]
    fn test_component_shape_derive_infers_generic_component_type() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(state = crate::state::PickerState<T, D>)]
            struct Picker<T, D = Vec<T>>(T, D);
        })
        .unwrap();

        let expanded = expand(input).unwrap();

        insta::assert_snapshot!(pretty_tokens(expanded));
    }

    #[test]
    fn test_component_shape_derive_explicit_value_binding() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(state = crate::state::TagsState, field_suffix = "tags", value_binding)]
            struct TagsInput;
        })
        .unwrap();

        let expanded = expand(input).unwrap();

        insta::assert_snapshot!(pretty_tokens(expanded));
    }

    #[test]
    fn test_component_shape_rejects_duplicate_options() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(
                state = crate::state::TagsState,
                new = crate::state::TagsState::new,
                new = crate::state::build
            )]
            struct TagsInput;
        })
        .unwrap();

        let err = expand(input).unwrap_err();

        assert!(
            err.to_string().contains("duplicate `new` option"),
            "should report the duplicated option name: {err}"
        );
    }

    #[test]
    fn test_component_shape_rejects_non_path_component_type() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(
                state = crate::state::TagsState,
                component = (crate::ui::TagsInput, crate::ui::OtherInput)
            )]
            struct TagsInput;
        })
        .unwrap();

        let err = expand(input).unwrap_err();

        assert!(
            err.to_string()
                .contains("component metadata must be a path-like type"),
            "should reject non-path component metadata: {err}"
        );
    }

    #[test]
    fn test_component_shape_rejects_invalid_field_suffix() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(state = crate::state::TagsState, field_suffix = "tags-input")]
            struct TagsInput;
        })
        .unwrap();

        let err = expand(input).unwrap_err();

        assert!(
            err.to_string()
                .contains("`field_suffix` must be a non-empty ASCII identifier suffix"),
            "should reject invalid field suffix metadata: {err}"
        );
    }
}
