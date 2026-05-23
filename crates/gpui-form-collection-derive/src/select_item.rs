use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

#[derive(FromDeriveInput)]
#[darling(supports(enum_any), attributes(select_item))]
struct SelectItemArgs {
    ident: syn::Ident,
    #[darling(default)]
    fluent: bool,
}

pub fn from(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let args = match SelectItemArgs::from_derive_input(&input) {
        Ok(args) => args,
        Err(err) => return err.write_errors().into(),
    };

    let item_ident = &args.ident;
    let fallback_title_token = match &input.data {
        Data::Enum(data) => {
            let arms = data.variants.iter().map(|variant| {
                let ident = &variant.ident;
                let title = ident.to_string();
                let pattern = match &variant.fields {
                    Fields::Named(_) => quote! { Self::#ident { .. } },
                    Fields::Unnamed(_) => quote! { Self::#ident(..) },
                    Fields::Unit => quote! { Self::#ident },
                };

                quote! { #pattern => #title.to_string(), }
            });

            quote! {
                match self {
                    #(#arms)*
                }
            }
        },
        _ => quote! { stringify!(#item_ident).to_string() },
    };

    let title_token = if args.fluent {
        quote! { #fallback_title_token.into() }
    } else {
        quote! { self.to_string().into() }
    };

    let expanded = quote! {
        impl gpui_component::select::SelectItem for #item_ident {
            type Value = Self;

            fn title(&self) -> gpui::SharedString {
                #title_token
            }

            fn value(&self) -> &Self::Value {
                self
            }
        }
    };

    expanded.into()
}
