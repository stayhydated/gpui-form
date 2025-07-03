use gpui_form_core::registry::FieldVariant;
use heck::{ToKebabCase as _, ToPascalCase as _};
use proc_macro2::TokenStream;
use quote::quote;

use crate::{code_gen::ShapeIdentities, implementations::ComponentIdentities as _};

use super::{FieldCodeGenerator, GeneratedSubscription};

pub struct CheckboxCodeGenerator;

impl FieldCodeGenerator for CheckboxCodeGenerator {
    fn generate_cx_new_call(
        &self,
        _field: &FieldVariant,

        _component: &ShapeIdentities,
    ) -> Option<TokenStream> {
        None
    }

    fn generate_field_initializers(
        &self,
        _field: &FieldVariant,
        _component: &ShapeIdentities,
    ) -> Option<TokenStream> {
        None
    }

    fn generate_render_child(
        &self,
        field: &FieldVariant,
        component: &ShapeIdentities,
    ) -> TokenStream {
        let ftl_label_ident = component.ftl_label_ident();
        let ftl_description_ident = component.ftl_description_ident();
        let field_name_ident = syn::parse_str::<syn::Ident>(field.field_name).unwrap();
        let field_name_pascal_case_ident =
            syn::parse_str::<syn::Ident>(&field.field_name.to_pascal_case()).unwrap();
        let suffix = field.behaviour.to_string();

        let component_gpui_type = field.behaviour.as_component_ident();

        let checkbox_id_str = format!("{}_{}", field.field_name, suffix).to_kebab_case();

        quote! {
            .child(
                form_field()
                    .label(#ftl_label_ident::#field_name_pascal_case_ident.to_string())
                    .description(#ftl_description_ident::#field_name_pascal_case_ident.to_string())
                    .child(#component_gpui_type::new(#checkbox_id_str)
                    .checked(self.current_data.#field_name_ident)
                    .on_click(cx.listener(|v, _, _, _| {
                        v.current_data.#field_name_ident = !v.current_data.#field_name_ident;
                    })),
                )
            )
        }
    }

    fn generate_focusable_cycle(
        &self,
        _field: &FieldVariant,
        _component: &ShapeIdentities,
    ) -> Option<TokenStream> {
        None
    }

    fn generate_subscription(
        &self,
        _field: &FieldVariant,
        _component: &ShapeIdentities,
    ) -> Option<GeneratedSubscription> {
        None
    }
}
