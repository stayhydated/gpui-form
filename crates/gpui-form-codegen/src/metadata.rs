use proc_macro2::TokenStream;
use quote::quote;

fn rust_metadata_string<T: quote::ToTokens>(tokens: &T) -> String {
    tokens.to_token_stream().to_string()
}

pub fn rust_type_tokens(schema_crate: &syn::Path, ty: &syn::Type) -> TokenStream {
    let ty = rust_metadata_string(ty);
    quote! {
        #schema_crate::schema::registry::RustType::new_unchecked(#ty)
    }
}

pub fn rust_path_tokens(schema_crate: &syn::Path, path: &syn::Path) -> TokenStream {
    let path = rust_metadata_string(path);
    quote! {
        #schema_crate::schema::registry::RustPath::new_unchecked(#path)
    }
}

pub fn rust_expr_tokens(schema_crate: &syn::Path, expr: &syn::Expr) -> TokenStream {
    let expr = rust_metadata_string(expr);
    quote! {
        #schema_crate::schema::registry::RustExpr::new_unchecked(#expr)
    }
}

pub fn optional_rust_expr_tokens(
    schema_crate: &syn::Path,
    expr: &Option<syn::Expr>,
) -> TokenStream {
    match expr {
        Some(expr) => {
            let expr = rust_expr_tokens(schema_crate, expr);
            quote! { Some(#expr) }
        },
        None => quote! { None },
    }
}
