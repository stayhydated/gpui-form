use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

pub fn constructor_body_tokens(new_expr: &Expr) -> TokenStream {
    if accepts_window_cx_arguments(new_expr) {
        quote! { (#new_expr)(window, cx) }
    } else {
        quote! { #new_expr }
    }
}

fn accepts_window_cx_arguments(expr: &Expr) -> bool {
    match expr {
        Expr::Group(group) => accepts_window_cx_arguments(&group.expr),
        Expr::Paren(paren) => accepts_window_cx_arguments(&paren.expr),
        Expr::Closure(_) | Expr::Path(_) => true,
        _ => false,
    }
}
