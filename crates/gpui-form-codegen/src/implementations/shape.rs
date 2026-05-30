use crate::CratePaths;
use crate::components::*;
use proc_macro2::TokenStream;
use quote::quote;

impl super::ComponentLayout for ShapeComponent {
    fn field_tokens(
        &self,
        field_structure_tokens: &mut TokenStream,
        field_base_declarations_tokens: &mut TokenStream,
    ) {
        let FieldInformation::<ResolvedComponentShape> {
            options,
            name,
            r#type: _,
        } = &self.0;

        let field_name_ident =
            match crate::names::ComponentFieldName::try_new(options.component_suffix(), name) {
                Ok(ident) => ident,
                Err(error) => {
                    field_structure_tokens.extend(error.to_compile_error());
                    return;
                },
            };
        let shape = options.shape();
        let crate_paths = CratePaths::resolve();
        let runtime_crate = crate_paths.gpui_form_facade_runtime();
        let gpui_crate = crate_paths.gpui;

        let state_type = quote! {
            <#shape as #runtime_crate::shape::ComponentShape>::State
        };
        let constructor_tokens = options.constructor_tokens();

        let field_structure_definition = quote! {
            pub #field_name_ident: #gpui_crate::Entity<#state_type>,
        };

        let field_base_declaration = quote! {
            pub fn #field_name_ident(
                window: &mut #gpui_crate::Window,
                cx: &mut #gpui_crate::Context<'_, #state_type>,
            ) -> #state_type {
                #constructor_tokens
            }
        };

        field_structure_tokens.extend(field_structure_definition);
        field_base_declarations_tokens.extend(field_base_declaration);
    }
}
