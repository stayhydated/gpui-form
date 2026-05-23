use gpui_form_schema::registry::{FieldVariant, GpuiFormShape};
use proc_macro2::TokenStream;
use quote::quote;

use crate::imports::ImportItem;

use super::{
    FieldCodeGenerator, GeneratedSubscription, ResolvedField, generate_entity_creation,
    generate_entity_field_initializer, render_standard_field,
};

/// Custom component support in prototyping initializes generated form fields
/// but does not infer subscriptions or actual widget rendering.
pub struct CustomCodeGenerator;

impl FieldCodeGenerator for CustomCodeGenerator {
    fn generate_imports(&self, field: &FieldVariant) -> Vec<ImportItem> {
        let _ = field;
        Vec::new()
    }

    fn generate_cx_new_call(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        Some(generate_entity_creation(field, component))
    }

    fn generate_field_initializers(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        Some(generate_entity_field_initializer(field))
    }

    fn generate_render_child(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> TokenStream {
        let field_in_struct_name_ident = field.field_ident_with_behaviour();

        // When the component type is known, emit Component::new(&entity) like other components.
        // The component type is in scope via `use {module}::*;` in the generated file.
        let child_tokens = if let Some(component_path) = field.custom_component_path() {
            quote! {
                #component_path::new(&self.fields.#field_in_struct_name_ident)
            }
        } else {
            let field_name = field.field_name();
            quote! {
                div().child(format!(
                    "Custom component `{}` – wire rendering via self.fields.{}",
                    #field_name,
                    stringify!(#field_in_struct_name_ident)
                ))
            }
        };

        render_standard_field(field, component, child_tokens)
    }

    fn generate_focusable_cycle(
        &self,
        _field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        None
    }

    fn generate_subscription(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<GeneratedSubscription> {
        if !field.custom_value_binding() {
            return None;
        }

        let shape = field.custom_shape_path()?;
        let field_type = field.value_type();
        let field_var_name_ident = field.field_ident_with_behaviour();
        let field_name_ident = field.field_ident();
        let event_handler_fn_name_ident = field.event_handler_ident("custom_event");

        let calls = vec![
            quote! { cx.subscribe_in(&#field_var_name_ident, window, Self::#event_handler_fn_name_ident) },
        ];

        let set_tokens = if field.value_holder_wraps_in_option() {
            quote! { self.current_data.#field_name_ident = Some(value); }
        } else {
            quote! { self.current_data.#field_name_ident = value; }
        };
        let clear_tokens = if field.value_holder_wraps_in_option() {
            quote! { self.current_data.#field_name_ident = None; }
        } else {
            quote! {}
        };

        let handler = quote! {
            fn #event_handler_fn_name_ident(
                &mut self,
                state: &Entity<<#shape as ::gpui_form::custom::CustomComponentShape>::State>,
                event: &<#shape as ::gpui_form::custom::CustomComponentValueAdapter<#field_type>>::Event,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) {
                let change = {
                    let state = state.read(_cx);
                    <#shape as ::gpui_form::custom::CustomComponentValueAdapter<#field_type>>::value_change(
                        &state,
                        event,
                    )
                };

                match change {
                    ::gpui_form::custom::CustomComponentValueChange::Set(value) => {
                        #set_tokens
                    }
                    ::gpui_form::custom::CustomComponentValueChange::Clear => {
                        #clear_tokens
                    }
                    ::gpui_form::custom::CustomComponentValueChange::Unchanged => {}
                }
            }
        };

        Some(GeneratedSubscription {
            calls,
            handlers: vec![handler],
        })
    }

    fn generate_post_subscription_initialization(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        if !field.custom_value_binding() {
            return None;
        }

        let shape = field.custom_shape_path()?;
        let field_type = field.value_type();
        let field_var_name_ident = field.field_ident_with_behaviour();
        let field_name_ident = field.field_ident();
        let value_tokens = if field.value_holder_wraps_in_option() {
            quote! { current_data.#field_name_ident.as_ref() }
        } else {
            quote! { Some(&current_data.#field_name_ident) }
        };

        Some(quote! {
            #field_var_name_ident.update(cx, |state, cx| {
                <#shape as ::gpui_form::custom::CustomComponentValueAdapter<#field_type>>::set_state_value(
                    state,
                    #value_tokens,
                    window,
                    cx,
                );
            });
        })
    }
}

#[cfg(test)]
mod tests {
    use super::CustomCodeGenerator;
    use crate::implementations::FieldCodeGenerator as _;
    use gpui_form_schema::{
        components::ComponentsBehaviour,
        registry::{FieldVariant, GpuiFormShape},
    };

    const CUSTOM_FIELDS: [FieldVariant; 1] = [FieldVariant::new(
        "tags",
        "Vec<String>",
        false,
        ComponentsBehaviour::Custom,
    )];
    const CUSTOM_SHAPE: GpuiFormShape =
        GpuiFormShape::new("Demo", &CUSTOM_FIELDS, "src/demo.rs", false);

    fn compact(input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn custom_generator_initializes_state_entity() {
        let generator = CustomCodeGenerator;
        let field = crate::implementations::ResolvedField::new(&CUSTOM_FIELDS[0]).unwrap();
        let tokens = generator
            .generate_cx_new_call(&field, &CUSTOM_SHAPE)
            .expect("custom fields should generate cx.new initialization");
        let compact = compact(&tokens.to_string());

        assert!(
            compact
                .contains("lettags_custom=cx.new(|cx|DemoFormComponents::tags_custom(window,cx));"),
            "cx initialization should call generated FormComponents constructor for custom field"
        );
    }

    #[test]
    fn custom_generator_initializes_form_fields_struct() {
        let generator = CustomCodeGenerator;
        let field = crate::implementations::ResolvedField::new(&CUSTOM_FIELDS[0]).unwrap();
        let tokens = generator
            .generate_field_initializers(&field, &CUSTOM_SHAPE)
            .expect("custom fields should be included in FormFields initializer");
        let compact = compact(&tokens.to_string());

        assert!(
            compact.contains("tags_custom,"),
            "field initializer should include custom state entity"
        );
    }

    #[test]
    fn custom_generator_emits_component_call_when_component_known() {
        const FIELDS_WITH_COMPONENT: [FieldVariant; 1] =
            [
                FieldVariant::new("tags", "Vec<String>", false, ComponentsBehaviour::Custom)
                    .with_custom_component("TagsInput"),
            ];
        const SHAPE: GpuiFormShape =
            GpuiFormShape::new("Demo", &FIELDS_WITH_COMPONENT, "src/demo.rs", false);

        let generator = CustomCodeGenerator;
        let field = crate::implementations::ResolvedField::new(&FIELDS_WITH_COMPONENT[0]).unwrap();
        let tokens = generator.generate_render_child(&field, &SHAPE);
        let compact = compact(&tokens.to_string());

        assert!(
            compact.contains("TagsInput::new(&self.fields.tags_custom)"),
            "render should emit Component::new(&entity): got {compact}"
        );
    }

    #[test]
    fn custom_generator_wires_opt_in_value_binding() {
        const FIELDS: [FieldVariant; 1] =
            [
                FieldVariant::new("country", "CountryCode", false, ComponentsBehaviour::Custom)
                    .with_wraps_in_option(true)
                    .with_custom_shape("crate::shapes::CountryShape")
                    .with_custom_value_binding(true),
            ];
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", false);

        let generator = CustomCodeGenerator;
        let field = crate::implementations::ResolvedField::new(&FIELDS[0]).unwrap();
        let generated = generator
            .generate_subscription(&field, &SHAPE)
            .expect("value-bound custom fields should generate subscriptions");
        let compact_handler = compact(&generated.handlers[0].to_string());

        assert!(
            compact_handler.contains(
                "<crate::shapes::CountryShapeas::gpui_form::custom::CustomComponentValueAdapter<CountryCode>>::Event"
            ),
            "custom event handler should use the shape's value adapter event type: {compact_handler}"
        );
        assert!(
            compact_handler.contains("self.current_data.country=Some(value);"),
            "optional custom value binding should assign Some(value): {compact_handler}"
        );

        let init = generator
            .generate_post_subscription_initialization(&field, &SHAPE)
            .expect("value-bound custom fields should seed state");
        let compact_init = compact(&init.to_string());
        assert!(
            compact_init
                .contains("set_state_value(state,current_data.country.as_ref(),window,cx,)"),
            "custom value binding should seed state from current_data: {compact_init}"
        );
    }
}
