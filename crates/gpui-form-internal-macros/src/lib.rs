use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[proc_macro_derive(ComponentDefinitions)]
pub fn derive_component_definition(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let enum_name = &input.ident;
    let discriminant_name = format_ident!("{}Discriminants", enum_name);
    let mut struct_definitions = Vec::new();

    if let syn::Data::Enum(data_enum) = &input.data {
        for variant in &data_enum.variants {
            let variant_name = &variant.ident;
            let struct_name = format_ident!("{}Component", variant_name);
            let option = format_ident!("{}Options", variant_name);

            struct_definitions.push(quote! {
                #[allow(dead_code)]
                pub struct #struct_name(pub crate::components::FieldInformation::<#option>);

                impl crate::components::ComponentDefinition for #struct_name {
                    fn component_name() -> &'static str {
                        crate::components::#discriminant_name::#variant_name.into()
                    }
                }

            });
        }
    }

    let expanded = quote! {
        #(#struct_definitions)*
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(ComponentOption)]
pub fn derive_component_option(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let struct_name = &input.ident;
    let expanded = quote! {
        impl crate::components::ComponentOption for #struct_name {}
    };
    expanded.into()
}
