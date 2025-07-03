use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[derive(FromDeriveInput)]
#[darling(supports(enum_any))]
struct DropdownItemArgs {
    ident: syn::Ident,
}

pub fn from(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let args = match DropdownItemArgs::from_derive_input(&input) {
        Ok(args) => args,
        Err(err) => return err.write_errors().into(),
    };

    let item_ident = &args.ident;

    let expanded = quote! {
        impl gpui_component::dropdown::DropdownItem for #item_ident {
            type Value = Self;

            fn title(&self) -> gpui::SharedString {
                self.to_string().into()
            }

            fn value(&self) -> &Self::Value {
                self
            }
        }
    };

    expanded.into()
}
