use heck::ToPascalCase as _;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::fs;
use std::path::PathBuf;
use syn::{File, Item};

#[proc_macro]
pub fn storybook(input: TokenStream) -> TokenStream {
    let suffix = if input.is_empty() {
        panic!("suffix parameter is required, e.g., storybook!(\"Form\")")
    } else {
        let input_str = input.to_string();
        let trimmed = input_str.trim_matches('"');
        trimmed.to_string()
    };

    let span = proc_macro::Span::call_site();
    let file_path = PathBuf::from(span.file());

    let dir_path = match file_path.parent() {
        Some(path) => path,
        None => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "Could not determine parent directory",
            )
            .to_compile_error()
            .into();
        },
    };

    let mod_rs_path = dir_path.join("mod.rs");
    let mod_rs_content = match fs::read_to_string(&mod_rs_path) {
        Ok(content) => content,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to read mod.rs at {:?}: {}", mod_rs_path, e),
            )
            .to_compile_error()
            .into();
        },
    };

    let ast: File = match syn::parse_file(&mod_rs_content) {
        Ok(file) => file,
        Err(e) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Failed to parse mod.rs: {}", e),
            )
            .to_compile_error()
            .into();
        },
    };

    let mut module_names = Vec::new();
    for item in ast.items {
        if let Item::Mod(item_mod) = item {
            module_names.push(item_mod.ident.to_string());
        }
    }

    module_names.sort();

    let init_calls: Vec<TokenStream2> = module_names
        .iter()
        .map(|name| {
            let ident = format_ident!("{}", name);
            quote! { #ident::init(cx) }
        })
        .collect();

    let story_calls: Vec<TokenStream2> = module_names
        .iter()
        .map(|name| {
            let ident = format_ident!("{}", name);
            let form_ident = format_ident!("{}{}", ident.to_string().to_pascal_case(), suffix);
            quote! { ::story_container::story::StoryContainer::panel::<#ident::#form_ident>(window, cx) }
        })
        .collect();

    let expanded = quote! {
        pub fn init(cx: &mut gpui::App) {
          #(#init_calls;)*
        }

        pub fn generate_stories(window: &mut ::gpui::Window, cx: &mut ::gpui::App) -> Vec<gpui::Entity<story_container::story::StoryContainer>> {
            vec![
                #(#story_calls,)*
            ]
        }
    };

    expanded.into()
}
