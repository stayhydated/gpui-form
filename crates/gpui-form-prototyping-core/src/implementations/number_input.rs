use gpui_form_core::registry::FieldVariant;
use heck::ToPascalCase as _;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{code_gen::ShapeIdentities, implementations::ComponentIdentities as _};

use super::{FieldCodeGenerator, GeneratedSubscription};

pub struct NumberInputCodeGenerator;

impl FieldCodeGenerator for NumberInputCodeGenerator {
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

        let state_initializer = syn::parse_str::<syn::Ident>(&field_var_name_str).unwrap();

        Some(quote! {
          #state_initializer,
        })
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

        let on_input_event_handler_fn_name = format!("on_{}_input_event", field.field_name);
        let on_input_event_handler_fn_name_ident =
            syn::parse_str::<syn::Ident>(&on_input_event_handler_fn_name).unwrap();
        let on_number_input_event_handler_fn_name =
            format!("on_{}_number_input_event", field.field_name);
        let on_number_input_event_handler_fn_name_ident =
            syn::parse_str::<syn::Ident>(&on_number_input_event_handler_fn_name).unwrap();

        let calls = vec![
            quote! { cx.subscribe_in(&#field_var_name_ident, window, Self::#on_input_event_handler_fn_name_ident) },
            quote! { cx.subscribe_in(&#field_var_name_ident, window, Self::#on_number_input_event_handler_fn_name_ident) },
        ];

        let mut handlers = vec![];

        let field_name_ident = syn::parse_str::<syn::Ident>(field.field_name).unwrap();

        let field_type_path = syn::parse_str::<syn::Type>(field.field_type).unwrap();

        let on_input_event_handler = quote! {
            fn #on_input_event_handler_fn_name_ident(
                &mut self,
                _this: &Entity<InputState>,
                event: &InputEvent,
                _: &mut Window,
                _: &mut Context<Self>,
            ) {
                match event {
                    InputEvent::Change(text) => {
                        if let Ok(value) = text.parse::<#field_type_path>() {
                            self.current_data.#field_name_ident = value.into();
                        }
                    }
                    _ => {}
                }
            }
        };
        handlers.push(on_input_event_handler);

        let field_type_ty = syn::parse_str::<syn::Type>(field.field_type).unwrap();

        let (decrement_logic, increment_logic) = if field.field_type.starts_with('f') {
            // f*
            (
                quote! {
                    let new_value = self.current_data.#field_name_ident - 1 as #field_type_ty;
                    self.current_data.#field_name_ident = new_value;
                },
                quote! {
                    let new_value = self.current_data.#field_name_ident + 1 as #field_type_ty;
                    self.current_data.#field_name_ident = new_value;
                },
            )
        } else if field.field_type.starts_with('u') || field.field_type.starts_with('i') {
            // i*, u*,
            (
                quote! {
                    let new_value = self.current_data.#field_name_ident.saturating_sub(1 as #field_type_ty);
                    self.current_data.#field_name_ident = new_value;
                },
                quote! {
                    let new_value = self.current_data.#field_name_ident.saturating_add(1 as #field_type_ty);
                    self.current_data.#field_name_ident = new_value;
                },
            )
        } else {
            // external types (assuming they impl `impl_saturating!`)
            (
                quote! {
                    let new_value = self.current_data.#field_name_ident.saturating_sub(#field_type_ty::from(1));
                    self.current_data.#field_name_ident = new_value;
                },
                quote! {
                    let new_value = self.current_data.#field_name_ident.saturating_add(#field_type_ty::from(1));
                    self.current_data.#field_name_ident = new_value;
                },
            )
        };

        let on_number_input_event_handler = quote! {
            fn #on_number_input_event_handler_fn_name_ident(
                &mut self,
                this: &Entity<InputState>,
                event: &NumberInputEvent,
                window: &mut Window,
                cx: &mut Context<Self>,
            ) {
                match event {
                    NumberInputEvent::Step(step_action) => match step_action {
                        StepAction::Decrement => {
                            #decrement_logic
                            this.update(cx, |input, cx| {
                                input.set_value(self.current_data.#field_name_ident.to_string(), window, cx);
                            });
                        }
                        StepAction::Increment => {
                            #increment_logic
                            this.update(cx, |input, cx| {
                                input.set_value(self.current_data.#field_name_ident.to_string(), window, cx);
                            });
                        }
                    },
                }
            }
        };
        handlers.push(on_number_input_event_handler);

        Some(GeneratedSubscription { calls, handlers })
    }
}
