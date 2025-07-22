use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, ItemStruct, parse_macro_input};

/// Attribute macro to register a story struct
#[proc_macro_attribute]
pub fn story(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &input_struct.ident;
    let struct_name_str = struct_name.to_string();

    let expanded = quote! {
        #input_struct

        inventory::submit! {
            ::story_container_core::registry::StoryEntry {
                name: #struct_name_str,
                create_fn: |window, cx| {
                    ::story_container_core::story::StoryContainer::panel::<#struct_name>(window, cx)
                },
            }
        }
    };

    expanded.into()
}

/// Attribute macro to register an init function
#[proc_macro_attribute]
pub fn story_init(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &input_fn.sig.ident;

    let expanded = quote! {
        #input_fn

        inventory::submit! {
            ::story_container_core::registry::::InitEntry {
                init_fn: #fn_name,
            }
        }
    };

    expanded.into()
}

#[proc_macro]
pub fn story_registry(_input: TokenStream) -> TokenStream {
    inventory::collect!(story_container_core::registry::StoryEntry);
    inventory::collect!(story_container_core::registry::InitEntry);

    let init_fn = story_container_core::registry::generate_init();
    let stories_fn = story_container_core::registry::generate_stories();

    let expanded = quote! {
        #init_fn
        #stories_fn
    };

    expanded.into()
}
