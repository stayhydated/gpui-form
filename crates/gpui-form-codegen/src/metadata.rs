use component_shape_codegen::{
    optional_rust_expr_metadata_tokens, rust_expr_metadata_tokens, rust_path_metadata_tokens,
    rust_type_metadata_tokens,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::Expr;

pub fn rust_type_tokens(schema_crate: &syn::Path, ty: &syn::Type) -> TokenStream {
    rust_type_metadata_tokens(quote! { #schema_crate::schema::registry::RustType }, ty)
}

pub fn rust_path_tokens(schema_crate: &syn::Path, path: &syn::Path) -> TokenStream {
    rust_path_metadata_tokens(quote! { #schema_crate::schema::registry::RustPath }, path)
}

pub fn rust_expr_tokens(schema_crate: &syn::Path, expr: &syn::Expr) -> TokenStream {
    rust_expr_metadata_tokens(quote! { #schema_crate::schema::registry::RustExpr }, expr)
}

pub fn optional_rust_expr_tokens(
    schema_crate: &syn::Path,
    expr: Option<&syn::Expr>,
) -> TokenStream {
    optional_rust_expr_metadata_tokens(quote! { #schema_crate::schema::registry::RustExpr }, expr)
}

fn unwrap_default_expr(expr: &Expr) -> &Expr {
    match expr {
        Expr::Group(group) => unwrap_default_expr(&group.expr),
        Expr::Paren(paren) => unwrap_default_expr(&paren.expr),
        Expr::Block(block) if block.block.stmts.len() == 1 => {
            if let syn::Stmt::Expr(expr, None) = &block.block.stmts[0] {
                unwrap_default_expr(expr)
            } else {
                expr
            }
        },
        _ => expr,
    }
}

fn should_wrap_default_into(expr: &Expr) -> bool {
    matches!(unwrap_default_expr(expr), Expr::Lit(_))
}

/// Lower a user-authored default expression into the source-model value type.
///
/// Literal defaults are wrapped in `Into::into(...)` so common string and
/// numeric literal defaults infer through the field type instead of the literal
/// type alone. Non-literal expressions are emitted as written.
pub fn source_default_tokens(expr: &Expr, span: Span) -> TokenStream {
    let expr_tokens = quote_spanned! {span=> #expr };
    if should_wrap_default_into(expr) {
        quote_spanned! {span=> ::core::convert::Into::into(#expr_tokens) }
    } else {
        expr_tokens
    }
}

/// Apply a source-model-to-form-value conversion expression to `value`.
pub fn apply_source_to_form_conversion_tokens(
    from_source: Option<&Expr>,
    span: Span,
    value: TokenStream,
) -> TokenStream {
    apply_conversion_tokens(from_source, span, value)
}

/// Apply a form-value-to-source-model conversion expression to `value`.
pub fn apply_form_to_source_conversion_tokens(
    into_source: Option<&Expr>,
    span: Span,
    value: TokenStream,
) -> TokenStream {
    apply_conversion_tokens(into_source, span, value)
}

fn apply_conversion_tokens(
    conversion: Option<&Expr>,
    span: Span,
    value: TokenStream,
) -> TokenStream {
    if let Some(conversion) = conversion {
        quote_spanned! {span=> (#conversion)(#value) }
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn compact(tokens: TokenStream) -> String {
        tokens
            .to_string()
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect()
    }

    #[test]
    fn source_default_wraps_literals_and_unwraps_simple_blocks() {
        let string_literal: Expr = syn::parse_quote!("name");
        let numeric_literal: Expr = syn::parse_quote!(42);
        let block_literal: Expr = syn::parse_quote!({ "name" });

        assert_eq!(
            compact(source_default_tokens(&string_literal, Span::call_site())),
            "::core::convert::Into::into(\"name\")"
        );
        assert_eq!(
            compact(source_default_tokens(&numeric_literal, Span::call_site())),
            "::core::convert::Into::into(42)"
        );
        assert_eq!(
            compact(source_default_tokens(&block_literal, Span::call_site())),
            "::core::convert::Into::into({\"name\"})"
        );
    }

    #[test]
    fn source_default_leaves_non_literals_as_written() {
        let path_call: Expr = syn::parse_quote!(Timestamp::now());
        let multi_stmt_block: Expr = syn::parse_quote!({
            let value = Timestamp::now();
            value
        });

        assert_eq!(
            compact(source_default_tokens(&path_call, Span::call_site())),
            "Timestamp::now()"
        );
        assert_eq!(
            compact(source_default_tokens(&multi_stmt_block, Span::call_site())),
            "{letvalue=Timestamp::now();value}"
        );
    }

    #[test]
    fn conversion_helpers_apply_declared_mapping_or_return_value() {
        let from_source: Expr = syn::parse_quote!(to_form);
        let into_source: Expr = syn::parse_quote!(|value| to_model(value));

        assert_eq!(
            compact(apply_source_to_form_conversion_tokens(
                Some(&from_source),
                Span::call_site(),
                quote! { source_default }
            )),
            "(to_form)(source_default)"
        );
        assert_eq!(
            compact(apply_form_to_source_conversion_tokens(
                Some(&into_source),
                Span::call_site(),
                quote! { value }
            )),
            "(|value|to_model(value))(value)"
        );
        assert_eq!(
            compact(apply_source_to_form_conversion_tokens(
                None,
                Span::call_site(),
                quote! { value }
            )),
            "value"
        );
    }
}
