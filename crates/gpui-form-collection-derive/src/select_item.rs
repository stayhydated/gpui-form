use darling::FromDeriveInput;
use gpui_form_codegen::resolve_crate_path;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

#[derive(FromDeriveInput)]
#[darling(supports(enum_any), attributes(select_item))]
struct SelectItemArgs {
    #[darling(default)]
    fluent: bool,
    #[darling(default)]
    display: bool,
}

pub fn from(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let args = match SelectItemArgs::from_derive_input(&input) {
        Ok(args) => args,
        Err(err) => return err.write_errors().into(),
    };
    if args.fluent && args.display {
        return syn::Error::new_spanned(
            &input.ident,
            "`#[select_item(fluent)]` and `#[select_item(display)]` are mutually exclusive",
        )
        .to_compile_error()
        .into();
    }

    let gpui_crate = resolve_crate_path("gpui", "::gpui");
    let gpui_component_crate = resolve_crate_path("gpui-component", "::gpui_component");
    let item_ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
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

    let title_token = if args.display {
        quote! { self.to_string().into() }
    } else {
        quote! { #fallback_title_token.into() }
    };

    let expanded = quote! {
        impl #impl_generics #gpui_component_crate::select::SelectItem
            for #item_ident #ty_generics #where_clause
        {
            type Value = Self;

            fn title(&self) -> #gpui_crate::SharedString {
                #title_token
            }

            fn value(&self) -> &Self::Value {
                self
            }
        }
    };

    expanded.into()
}
