use super::__crate_paths;
use crate::components::*;
use proc_macro2::TokenStream;
use quote::quote;

impl super::ComponentLayout for DropdownComponent {
    fn field_tokens(
        &self,
        field_structure_tokens: &mut TokenStream,
        field_base_declarations_tokens: &mut TokenStream,
    ) {
        let FieldInformation::<DropdownOptions> {
            options,
            name,
            r#type,
        } = &self.0;

        let field_name_ident = crate::component_field_name!(name);

        use __crate_paths::gpui::{Context, Entity, Window};
        use __crate_paths::gpui_component::IndexPath;
        use __crate_paths::gpui_component::dropdown::{DropdownState, SearchableVec};

        let vec_type = if options.behaviour.searchable {
            quote! { #SearchableVec }
        } else {
            quote! { Vec }
        };

        let state_type = quote! {
          #DropdownState<#vec_type<#r#type>>
        };

        let field_structure_definition = quote! {
            pub #field_name_ident: #Entity<#state_type>,
        };

        let index = if let Some(named_index) = options.named_index() {
            let path = named_index.clone();
            quote! {
              Some(
                #IndexPath::new(
                  #r#type::iter()
                    .position(|x| x == #path)
                    .unwrap()
                )
              )
            }
        } else if options.index_default() {
            quote! {
              Some(
                #IndexPath::new(
                  #r#type::iter()
                    .position(|x| x == #r#type::default())
                    .unwrap()
                )
              )
            }
        } else {
            quote! { None }
        };

        let field_base_declaration = if !options.behaviour.partial {
            quote! {
                pub fn #field_name_ident(window: &mut #Window, cx: &mut #Context<'_, #state_type>) -> #state_type {
                  use strum::IntoEnumIterator as _;
                  #DropdownState::new(#r#type::iter().collect::<Vec<#r#type>>().into(), #index, window, cx)
                }
            }
        } else {
            quote! {}
        };

        field_structure_tokens.extend(field_structure_definition);
        field_base_declarations_tokens.extend(field_base_declaration);
    }
}
