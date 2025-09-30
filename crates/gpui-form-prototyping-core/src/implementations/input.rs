use gpui_form_core::registry::FieldVariant;
use heck::ToPascalCase as _;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{code_gen::ShapeIdentities, implementations::ComponentIdentities as _};

use super::{FieldCodeGenerator, GeneratedSubscription};

pub struct InputCodeGenerator;

impl FieldCodeGenerator for InputCodeGenerator {
    fn generate_cx_new_call(
        &self,
        field: &FieldVariant,
        component: &ShapeIdentities,
    ) -> Option<TokenStream> {
        let form_components_struct_ident = component.struct_form_components_ident();
        let suffix = field.behaviour.to_string();
        let var_name_ident =
            syn::parse_str::<syn::Ident>(&format!("{}_{}", field.field_name, suffix)).unwrap();
        let fn_name_ident =
            syn::parse_str::<syn::Ident>(&format!("{}_{}", field.field_name, suffix)).unwrap();

        Some(quote! {
            let #var_name_ident =
                cx.new(|cx| #form_components_struct_ident::#fn_name_ident(window, cx));
        })
    }

    fn generate_field_initializers(
        &self,
        field: &FieldVariant,
        _component: &ShapeIdentities,
    ) -> Option<TokenStream> {
        let suffix = field.behaviour.to_string();
        let field_var_name_str = format!("{}_{}", field.field_name, suffix);
        let field_var_name_ident = syn::parse_str::<syn::Ident>(&field_var_name_str).unwrap();

        Some(quote! { #field_var_name_ident, })
    }

    fn generate_render_child(
        &self,
        field: &FieldVariant,
        component: &ShapeIdentities,
    ) -> TokenStream {
        let ftl_label_ident = component.ftl_label_ident();
        let ftl_description_ident = component.ftl_description_ident();
        let field_name_pascal_case_ident =
            syn::parse_str::<syn::Ident>(&field.field_name.to_pascal_case()).unwrap();
        let suffix = field.behaviour.to_string();

        let component_gpui_type = field.behaviour.as_component_ident();

        let field_in_struct_name_str = format!("{}_{}", field.field_name, suffix);
        let field_in_struct_name_ident =
            syn::parse_str::<syn::Ident>(&field_in_struct_name_str).unwrap();

        quote! {
            .child(
                form_field()
                    .label(#ftl_label_ident::#field_name_pascal_case_ident.to_string())
                    .description(#ftl_description_ident::#field_name_pascal_case_ident.to_string())
                    .child(#component_gpui_type::new(&self.fields.#field_in_struct_name_ident))
            )
        }
    }

    fn generate_focusable_cycle(
        &self,
        field: &FieldVariant,
        _component: &ShapeIdentities,
    ) -> Option<TokenStream> {
        let suffix = field.behaviour.to_string();
        let field_var_name_str = format!("{}_{}", field.field_name, suffix);
        let field_var_name_ident = syn::parse_str::<syn::Ident>(&field_var_name_str).unwrap();
        let x = quote! {
          self.fields.#field_var_name_ident.focus_handle(cx),
        };
        Some(x)
    }

    fn generate_subscription(
        &self,
        field: &FieldVariant,
        _component: &ShapeIdentities,
    ) -> Option<GeneratedSubscription> {
        let suffix = field.behaviour.to_string();
        let field_var_name_str = format!("{}_{}", field.field_name, suffix);
        let field_var_name_ident = syn::parse_str::<syn::Ident>(&field_var_name_str).unwrap();

        let event_handler_fn_name = format!("on_{}_input_event", field.field_name);
        let event_handler_fn_name_ident =
            syn::parse_str::<syn::Ident>(&event_handler_fn_name).unwrap();

        let calls = vec![
            quote! { cx.subscribe_in(&#field_var_name_ident, window, Self::#event_handler_fn_name_ident) },
        ];

        let field_name_ident = syn::parse_str::<syn::Ident>(field.field_name).unwrap();

        let handler = quote! {
            fn #event_handler_fn_name_ident(
                &mut self,
                state: &Entity<InputState>,
                event: &InputEvent,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) {
                match event {
                    InputEvent::Change => {
                      let text = state.read(_cx).value();
                      self.current_data.#field_name_ident = text.to_owned().into();
                    }
                    _ => {}
                }
            }
        };

        Some(GeneratedSubscription {
            calls,
            handlers: vec![handler],
        })
    }
}
