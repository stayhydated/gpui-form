use super::__crate_paths;
use crate::components::*;
use proc_macro2::TokenStream;
use quote::quote;

impl super::ComponentLayout for NumberInputComponent {
    fn field_tokens(
        &self,
        field_structure_tokens: &mut TokenStream,
        field_base_declarations_tokens: &mut TokenStream,
    ) {
        let FieldInformation::<NumberInputOptions> {
            options: _,
            name,
            r#type,
        } = &self.0;

        let field_name_ident = crate::component_field_name!(name);

        use __crate_paths::gpui::{Context, Entity, Window};
        use __crate_paths::gpui_component::input::InputState;

        let field_structure_definition = quote! {
            pub #field_name_ident: #Entity<#InputState>,
        };

        let field_base_declaration = quote! {
            pub fn #field_name_ident(window: &mut #Window, cx: &mut #Context<'_, #InputState>) -> #InputState {
              use ::gpui_form::NumRegex;
                #InputState::new(window, cx).pattern(#r#type::validation_regex().clone())
            }
        };

        field_structure_tokens.extend(field_structure_definition);
        field_base_declarations_tokens.extend(field_base_declaration);
    }
}
