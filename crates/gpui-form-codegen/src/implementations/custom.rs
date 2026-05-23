use crate::components::*;
use proc_macro2::TokenStream;
use quote::quote;

impl super::ComponentLayout for CustomComponent {
    fn field_tokens(
        &self,
        field_structure_tokens: &mut TokenStream,
        field_base_declarations_tokens: &mut TokenStream,
    ) {
        let FieldInformation::<CustomOptions> {
            options,
            name,
            r#type: _,
        } = &self.0;

        let field_name_ident = crate::component_field_name!(name);
        let shape = &options.shape;

        let state_type = quote! {
            <#shape as ::gpui_form_component::custom::CustomComponentShape>::State
        };

        let field_structure_definition = quote! {
            pub #field_name_ident: ::gpui::Entity<#state_type>,
        };

        let field_base_declaration = quote! {
            pub fn #field_name_ident(
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::Context<'_, #state_type>,
            ) -> #state_type {
                <#shape as ::gpui_form_component::custom::CustomComponentShape>::new(window, cx)
            }
        };

        field_structure_tokens.extend(field_structure_definition);
        field_base_declarations_tokens.extend(field_base_declaration);
    }
}
