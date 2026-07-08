use gpui_form_codegen::resolve_crate_path;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned as _;
use syn::{
    AngleBracketedGenericArguments, FnArg, GenericArgument, ItemFn, PatType, PathArguments,
    ReturnType, Type, TypePath,
};

pub fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let item_fn: ItemFn = syn::parse2(item)?;
    reject_attribute_arguments(&attr)?;
    let facade_crate = resolve_crate_path("gpui-form", "::gpui_form");
    let first_arg = first_typed_argument(&item_fn)?;
    let first_type = &first_arg.ty;
    reject_reference_argument(first_type)?;
    let (result_type, result_error_type) = parse_result_return_type(&item_fn.sig.output)?;

    let fn_ident = &item_fn.sig.ident;
    let register_ident = format_ident!("__gpui_form_mcp_register_{fn_ident}");
    let register_submit_ident = format_ident!("__gpui_form_mcp_register_submit_{fn_ident}");
    let descriptor_ident = format_ident!("__gpui_form_mcp_descriptor_{fn_ident}");
    let tool_definitions_ident = format_ident!("__gpui_form_mcp_tool_definitions_{fn_ident}");
    let submit_tool_definitions_ident =
        format_ident!("__gpui_form_mcp_submit_tool_definitions_{fn_ident}");
    let is_async = item_fn.sig.asyncness.is_some();

    let register_call = if is_async {
        quote! {
            <#first_type as #facade_crate::mcp::McpSubmitArgument>::register_async_with_editor(
                server,
                #fn_ident,
            )
        }
    } else {
        quote! {
            <#first_type as #facade_crate::mcp::McpSubmitArgument>::register_sync_with_editor(
                server,
                #fn_ident,
            )
        }
    };
    let register_submit_call = if is_async {
        quote! {
            <#first_type as #facade_crate::mcp::McpSubmitArgument>::register_async(
                server,
                #fn_ident,
            )
        }
    } else {
        quote! {
            <#first_type as #facade_crate::mcp::McpSubmitArgument>::register_sync(
                server,
                #fn_ident,
            )
        }
    };

    Ok(quote! {
        #item_fn

        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #register_ident(
            server: &mut #facade_crate::mcp::McpServer
        ) -> Result<(), #facade_crate::mcp::McpToolError> {
            fn __gpui_form_mcp_submit_assert_argument<T: #facade_crate::mcp::McpSubmitArgument>() {}
            __gpui_form_mcp_submit_assert_argument::<#first_type>();
            fn __gpui_form_mcp_submit_assert_error<T: ::core::fmt::Display>() {}
            __gpui_form_mcp_submit_assert_error::<#result_error_type>();
            fn __gpui_form_mcp_submit_assert_serialize<T: #facade_crate::mcp::Serialize>() {}
            __gpui_form_mcp_submit_assert_serialize::<#result_type>();
            fn __gpui_form_mcp_submit_assert_schema<T: #facade_crate::mcp::McpJsonSchema>() {}
            __gpui_form_mcp_submit_assert_schema::<#result_type>();
            #register_call
        }

        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #register_submit_ident(
            server: &mut #facade_crate::mcp::McpServer
        ) -> Result<(), #facade_crate::mcp::McpToolError> {
            fn __gpui_form_mcp_submit_assert_argument<T: #facade_crate::mcp::McpSubmitArgument>() {}
            __gpui_form_mcp_submit_assert_argument::<#first_type>();
            fn __gpui_form_mcp_submit_assert_error<T: ::core::fmt::Display>() {}
            __gpui_form_mcp_submit_assert_error::<#result_error_type>();
            fn __gpui_form_mcp_submit_assert_serialize<T: #facade_crate::mcp::Serialize>() {}
            __gpui_form_mcp_submit_assert_serialize::<#result_type>();
            fn __gpui_form_mcp_submit_assert_schema<T: #facade_crate::mcp::McpJsonSchema>() {}
            __gpui_form_mcp_submit_assert_schema::<#result_type>();
            #register_submit_call
        }

        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #descriptor_ident() -> #facade_crate::mcp::McpFormDescriptor {
            <
                <#first_type as #facade_crate::mcp::McpSubmitArgument>::Form
                as #facade_crate::mcp::McpForm
            >::descriptor()
        }

        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #tool_definitions_ident(
        ) -> Result<Vec<#facade_crate::mcp::ToolDefinition>, #facade_crate::mcp::McpToolError> {
            <#first_type as #facade_crate::mcp::McpSubmitArgument>::tool_definitions_with_editor::<#result_type>()
        }

        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #submit_tool_definitions_ident(
        ) -> Result<Vec<#facade_crate::mcp::ToolDefinition>, #facade_crate::mcp::McpToolError> {
            Ok(vec![
                #facade_crate::mcp::submit_tool_definition_for_response::<
                    <
                        #first_type as #facade_crate::mcp::McpSubmitArgument
                    >::Form,
                    #result_type,
                >()?
            ])
        }

        #facade_crate::mcp::registry::inventory::submit! {
            #facade_crate::mcp::registry::McpSubmitHandlerRegistration::new_with_submit_only(
                #descriptor_ident,
                #register_ident,
                #register_submit_ident,
                #tool_definitions_ident,
                #submit_tool_definitions_ident,
            )
        }
    })
}

fn reject_attribute_arguments(attr: &TokenStream) -> syn::Result<()> {
    if attr.is_empty() {
        return Ok(());
    }

    Err(syn::Error::new(
        attr.span(),
        "`gpui_form::mcp_submit` does not accept arguments; use `#[gpui_form::mcp_submit]` and let the owned handler parameter choose model vs holder submission",
    ))
}

fn reject_reference_argument(first_type: &Type) -> syn::Result<()> {
    if let Type::Reference(reference) = first_type {
        return Err(syn::Error::new_spanned(
            reference,
            "mcp_submit handlers require owned parameters; use `Form` or `FormValueHolder` directly",
        ));
    }

    Ok(())
}

fn parse_result_return_type(output: &ReturnType) -> syn::Result<(Type, Type)> {
    let ReturnType::Type(_, ty) = output else {
        return Err(syn::Error::new(
            output.span(),
            "gpui_form::mcp_submit handlers must return Result<T, E> for explicit MCP error handling",
        ));
    };

    parse_result_type(ty.as_ref()).ok_or_else(|| {
        syn::Error::new(
            ty.span(),
            "gpui_form::mcp_submit handlers must return Result<T, E> for explicit MCP error handling",
        )
    })
}

fn parse_result_type(ty: &Type) -> Option<(Type, Type)> {
    let Type::Path(TypePath { path, qself }) = ty else {
        return None;
    };
    if qself.is_some() {
        return None;
    }

    if !is_std_result_path(path) {
        return None;
    }

    let segment = path.segments.last()?;
    parse_result_type_arguments(&segment.arguments)
}

fn parse_result_type_arguments(arguments: &PathArguments) -> Option<(Type, Type)> {
    let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = arguments
    else {
        return None;
    };

    let mut type_arguments = args.iter();
    let GenericArgument::Type(ok_type) = type_arguments.next()? else {
        return None;
    };
    let GenericArgument::Type(error_type) = type_arguments.next()? else {
        return None;
    };
    if type_arguments.next().is_some() {
        return None;
    }

    Some((ok_type.clone(), error_type.clone()))
}

fn is_std_result_path(path: &syn::Path) -> bool {
    let segments: Vec<_> = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();

    match segments.as_slice() {
        [result] if result == "Result" => true,
        [first, second, third] => {
            matches!(
                (first.as_str(), second.as_str(), third.as_str()),
                ("std", "result", "Result")
                    | ("core", "result", "Result")
                    | ("alloc", "result", "Result")
            )
        },
        _ => false,
    }
}

fn first_typed_argument(item_fn: &ItemFn) -> syn::Result<&PatType> {
    if let Some(self_token_span) = item_fn
        .sig
        .inputs
        .iter()
        .find_map(|argument| match argument {
            FnArg::Receiver(receiver) => Some(receiver.self_token.span),
            _ => None,
        })
    {
        return Err(syn::Error::new(
            self_token_span,
            "mcp_submit handlers cannot be methods; use a free function",
        ));
    }

    let typed_arguments: Vec<_> = item_fn
        .sig
        .inputs
        .iter()
        .filter_map(|argument| match argument {
            FnArg::Typed(argument) => Some(argument),
            FnArg::Receiver(_) => None,
        })
        .collect();

    if typed_arguments.len() != 1 {
        return if typed_arguments.is_empty() {
            Err(syn::Error::new(
                item_fn.sig.ident.span(),
                "mcp_submit handlers require exactly one owned parameter of type `Form` or `FormValueHolder`",
            ))
        } else {
            Err(syn::Error::new_spanned(
                typed_arguments[1],
                "mcp_submit handlers must take exactly one typed parameter",
            ))
        };
    }

    Ok(typed_arguments[0])
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse2;

    fn expand_error(attr: proc_macro2::TokenStream, item: &str) -> String {
        let item: ItemFn = parse2(item.parse().expect("valid test function")).expect("parse item");
        expand(attr, quote! { #item }).unwrap_err().to_string()
    }

    #[test]
    fn rejects_non_owned_first_argument() {
        let error = expand_error(
            quote! {},
            "fn submit(request: &String) -> String { request.len().to_string() }",
        );
        assert!(
            error.contains("owned parameter") || error.contains("owned parameters"),
            "expected owned-parameter diagnostic, got: {error}"
        );
    }

    #[test]
    fn rejects_multiple_typed_arguments() {
        let error = expand_error(
            quote! {},
            "fn submit(request: String, extra: u32) -> String { request }",
        );
        assert!(error.contains("exactly one typed parameter"));
    }

    #[test]
    fn rejects_no_typed_arguments() {
        let error = expand_error(quote! {}, "fn submit() -> String { \"ok\".to_string() }");
        assert!(error.contains("exactly one owned parameter"));
    }

    #[test]
    fn rejects_attribute_arguments() {
        let error = expand_error(
            quote! { model },
            "fn submit(request: String) -> String { request }",
        );
        assert!(error.contains("does not accept arguments"));
    }

    #[test]
    fn rejects_non_result_return_type() {
        let error = expand_error(
            quote! {},
            "fn submit(request: String) -> serde_json::Value { serde_json::json!({}) }",
        );
        assert!(error.contains("must return Result<T, E>"));
    }

    #[test]
    fn rejects_result_with_wrong_number_of_type_arguments() {
        let error = expand_error(
            quote! {},
            "fn submit(request: String) -> Result<String> { Ok(request) }",
        );
        assert!(error.contains("must return Result<T, E>"));
    }

    #[test]
    fn accepts_explicit_std_result_paths() {
        let item_fn: ItemFn = parse2(
            "fn submit(request: String) -> std::result::Result<String, String> { Ok(request) }"
                .parse()
                .expect("valid function"),
        )
        .expect("parse item");
        assert!(expand(quote! {}, quote! { #item_fn }).is_ok());
    }

    #[test]
    fn requires_serializable_schema_success_type() {
        let item_fn: ItemFn = parse2(
            "fn submit(request: String) -> Result<SubmitResponse, String> { submit_response(request) }"
                .parse()
                .expect("valid function"),
        )
        .expect("parse item");
        let expanded = expand(quote! {}, quote! { #item_fn })
            .expect("mcp_submit should accept a serializable schema response type");
        let expanded = expanded.to_string();
        assert!(expanded.contains("mcp_submit_assert_serialize"));
        assert!(expanded.contains("Serialize"));
        assert!(expanded.contains("mcp_submit_assert_schema"));
        assert!(expanded.contains("McpJsonSchema"));
    }

    #[test]
    fn delegates_sync_registration_to_submit_argument_trait() {
        let item_fn: ItemFn = parse2(
            "fn submit(request: ContactRequest) -> Result<SubmitResponse, String> { submit_response(request) }"
                .parse()
                .expect("valid function"),
        )
        .expect("parse item");
        let expanded = expand(quote! {}, quote! { #item_fn }).expect("mcp_submit should expand");
        let expanded = expanded.to_string();

        assert!(expanded.contains("McpSubmitArgument"));
        assert!(expanded.contains("register_sync"));
    }

    #[test]
    fn delegates_async_registration_to_submit_argument_trait() {
        let item_fn: ItemFn = parse2(
            "async fn submit(holder: ContactRequestFormValueHolder) -> Result<SubmitResponse, String> { submit_response(holder).await }"
                .parse()
                .expect("valid function"),
        )
        .expect("parse item");
        let expanded = expand(quote! {}, quote! { #item_fn }).expect("mcp_submit should expand");
        let expanded = expanded.to_string();

        assert!(expanded.contains("McpSubmitArgument"));
        assert!(expanded.contains("register_async"));
    }

    #[test]
    fn rejects_result_with_extra_type_arguments() {
        let error = expand_error(
            quote! {},
            "fn submit(request: String) -> Result<String, String, usize> { Ok(request) }",
        );
        assert!(error.contains("must return Result<T, E>"));
    }

    #[test]
    fn rejects_result_with_non_type_generic_argument() {
        let error = expand_error(
            quote! {},
            "fn submit(request: String) -> Result<String, 'static> { Ok(request) }",
        );
        assert!(error.contains("must return Result<T, E>"));
    }

    #[test]
    fn rejects_non_standard_result_path() {
        let error = expand_error(
            quote! {},
            "fn submit(request: String) -> gpui_form::Result<String, String> { Ok(request) }",
        );
        assert!(error.contains("must return Result<T, E>"));
    }
}
