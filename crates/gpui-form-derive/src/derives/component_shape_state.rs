use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result, parse_macro_input};

use gpui_form_codegen::CratePaths;

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
            } else if meta.path.is_ident("state") {
                let value = meta.value()?;
                let state = value.parse()?;
                shape.set_state(state, &meta.path)
            } else if meta.path.is_ident("component") {
                let value = meta.value()?;
                let component = value.parse()?;
                shape.set_component(component, &meta.path)
            } else if meta.path.is_ident("requires_value") {
                let value = meta.value()?;
                let requires_value = value.parse()?;
                shape.set_requires_value(requires_value, &meta.path)
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
    let mut meta = parse_meta(&input.attrs)?;
    meta.enable_value_binding(ident)?;
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
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let metadata_impl_items = meta.impl_items_tokens(&runtime_crate);
    let gpui_crate = CratePaths::resolve().gpui;
    let inferred_component_const = if !meta.has_component() {
        Some(quote! {
            const COMPONENT_PATH: Option<&'static str> =
                Some(concat!(module_path!(), "::", stringify!(#ident)));
        })
    } else {
        None
    };
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
            #inferred_component_const
        }

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
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("(crate::state::build)(window,cx)"),
            "should use explicit new path from attribute"
        );
        assert!(
            compact.contains("concat!(module_path!(),\"::\",stringify!(TagsInput))"),
            "component path should be inferred when component is not specified"
        );
    }

    #[test]
    fn test_component_shape_with_component_path() {
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
            #[gpui_form_shape(
                state = crate::state::TagsState,
                new = |window, cx| crate::state::TagsState::with_mode(window, cx, Mode::Tags)
            )]
            struct TagsInput;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains(
                "(|window,cx|crate::state::TagsState::with_mode(window,cx,Mode::Tags))(window,cx)"
            ),
            "should allow derive constructors to be full expressions"
        );
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

        assert!(
            compact.contains("crate::state::TagsState::with_mode(window,cx,Mode::Tags)"),
            "direct constructor calls should be emitted as written: {compact}"
        );
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
            #[gpui_form_shape(state = crate::state::TagsState, field_suffix = "tags")]
            struct TagsInput;
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
    fn test_component_shape_with_required_value_policy() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(ComponentShape)]
            #[gpui_form_shape(state = crate::state::SwitchState, requires_value = false)]
            struct SwitchInput;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact
                .contains("typeRequiredValuePolicy=::gpui_form_runtime::shape::AllowMissingValue"),
            "should emit shape-owned required-value policy"
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

        assert!(
            compact.contains("typeState=crate::state::TagsState"),
            "component derive should store the declared backing state: {compact}"
        );
        assert!(
            compact.contains("<crate::state::TagsState>::new(window,cx)"),
            "component derive should default construction to State::new: {compact}"
        );
        assert!(
            compact.contains("concat!(module_path!(),\"::\",stringify!(TagsInput))"),
            "component derive should infer the UI component path: {compact}"
        );
        assert!(
            compact.contains("VALUE_BINDING:bool=true"),
            "component derive should opt into value binding by default: {compact}"
        );
        assert!(
            compact.contains(
                "ComponentValueBinding<__GpuiFormValueBindingValue>forTagsInputwherecrate::state::TagsState:::gpui_form_runtime::shape::ComponentStateValueBinding<__GpuiFormValueBindingValue>"
            ),
            "component derive should delegate value binding through the backing state: {compact}"
        );
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
}
