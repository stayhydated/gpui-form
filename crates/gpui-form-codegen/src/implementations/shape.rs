use crate::components::*;
use proc_macro2::TokenStream;
use quote::quote;

impl super::ComponentLayout for ShapeComponent {
    fn field_tokens(
        &self,
        field_structure_tokens: &mut TokenStream,
        field_base_declarations_tokens: &mut TokenStream,
    ) {
        let FieldInformation::<ShapeOptions> {
            options,
            name,
            r#type,
        } = &self.0;

        let field_name_ident =
            crate::names::ComponentFieldName::new(&options.component_suffix(name), name);
        let shape = options.runtime_shape(r#type);

        let state_type = quote! {
            <#shape as ::gpui_form_runtime::shape::ComponentShape>::State
        };
        let constructor_tokens = options.constructor_tokens(r#type);

        let field_structure_definition = quote! {
            pub #field_name_ident: ::gpui::Entity<#state_type>,
        };

        let field_base_declaration = quote! {
            pub fn #field_name_ident(
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::Context<'_, #state_type>,
            ) -> #state_type {
                #constructor_tokens
            }
        };

        field_structure_tokens.extend(field_structure_definition);
        field_base_declarations_tokens.extend(field_base_declaration);
    }
}
