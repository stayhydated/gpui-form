use super::*;
use quote::quote;
use syn::parse_quote;
use syn::punctuated::Punctuated;

fn parse_options(
    tokens: proc_macro2::TokenStream,
) -> syn::Result<Punctuated<GpuiFormFieldOption, syn::Token![,]>> {
    Punctuated::<GpuiFormFieldOption, syn::Token![,]>::parse_terminated.parse2(tokens)
}

#[test]
fn raw_field_options_parse_structured_component_intent() {
    let parsed = parse_options(quote! {
        component(
            crate::Input,
            value(
                type = String,
                from_source = to_form,
                into_source = to_source,
            ),
            default = "name"
        )
    })
    .expect("component option should parse");

    let Some(GpuiFormFieldOption::Component { shape, options, .. }) = parsed.first() else {
        panic!("expected component option");
    };

    assert_eq!(
        quote::ToTokens::to_token_stream(shape).to_string(),
        "crate :: Input"
    );
    assert!(options.default.is_some());
    assert!(matches!(&options.value, RenderedValueIntent::Converted(_)));
}

#[test]
fn raw_field_options_parse_fallible_value_conversion() {
    let parsed = parse_options(quote! {
        hidden(value(
            type = String,
            from_source = to_form,
            try_into_source = try_to_source,
        ))
    })
    .expect("fallible value conversion should parse");

    let Some(GpuiFormFieldOption::Hidden { options, .. }) = parsed.first() else {
        panic!("expected hidden option");
    };
    let RenderedValueIntent::Converted(converted) = &options.value else {
        panic!("expected converted value intent");
    };

    assert!(converted.into_source_is_fallible);
    assert_eq!(
        converted.into_source.value.to_token_stream().to_string(),
        "try_to_source"
    );
}

#[test]
fn raw_field_options_parse_koruma_newtype_shortcut() {
    let parsed = parse_options(quote! {
        hidden(value(koruma_newtype))
    })
    .expect("koruma newtype shortcut should parse");

    let Some(GpuiFormFieldOption::Hidden { options, .. }) = parsed.first() else {
        panic!("expected hidden option");
    };

    assert!(matches!(
        options.value,
        RenderedValueIntent::KorumaNewtype { .. }
    ));
}

#[test]
fn raw_field_options_reject_unknown_option() {
    let error =
        parse_options(quote! { field_suffix = "input" }).expect_err("unknown options should fail");

    assert!(
        error.to_string().contains("unknown gpui_form field option"),
        "unexpected error: {error}"
    );
}

#[test]
fn raw_value_options_reject_incomplete_conversion_pair() {
    let error = parse_options(quote! {
        hidden(value(type = String, from_source = to_form))
    })
    .expect_err("incomplete value conversion should fail");

    assert!(
        error.to_string().contains(
            "requires an explicit `into_source = ...` or `try_into_source = ...` conversion"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn raw_value_options_reject_option_type_override() {
    let error = parse_options(quote! {
        hidden(value(
            type = Option<String>,
            from_source = to_form,
            into_source = to_source,
        ))
    })
    .expect_err("Option<T> form type override should fail");

    assert!(
        error
            .to_string()
            .contains("expects the form-side base value type, not Option<T>"),
        "unexpected error: {error}"
    );
}

#[test]
fn mcp_tool_options_parse_word_and_list_forms() {
    let parsed_word = McpToolOptions::from_word().expect("mcp word form should parse");
    assert!(parsed_word.name.is_none());
    assert!(parsed_word.title.is_none());
    assert!(parsed_word.description.is_none());

    let parsed_list = McpToolOptions::from_list(&[
        parse_quote!(name = "submit_contact"),
        parse_quote!(title = "Submit Contact"),
        parse_quote!(description = "Submit a contact request"),
        parse_quote!(read_only = false),
        parse_quote!(destructive = true),
        parse_quote!(idempotent = false),
        parse_quote!(open_world = true),
        parse_quote!(icon(
            src = "https://example.com/contact.png",
            mime_type = "image/png",
            size = "48x48",
            theme = "light"
        )),
        parse_quote!(context(crate::SubmitContext)),
        parse_quote!(response(crate::SubmitResponse)),
        parse_quote!(error(crate::SubmitError)),
        parse_quote!(submit(crate::submit_request)),
        parse_quote!(map_response(crate::map_response)),
    ])
    .expect("mcp list form should parse");

    assert_eq!(parsed_list.name.as_deref(), Some("submit_contact"));
    assert_eq!(parsed_list.title.as_deref(), Some("Submit Contact"));
    assert_eq!(
        parsed_list.description.as_deref(),
        Some("Submit a contact request")
    );
    assert_eq!(parsed_list.read_only, Some(false));
    assert_eq!(parsed_list.destructive, Some(true));
    assert_eq!(parsed_list.idempotent, Some(false));
    assert_eq!(parsed_list.open_world, Some(true));
    assert_eq!(parsed_list.icons.len(), 1);
    assert_eq!(parsed_list.icons[0].src, "https://example.com/contact.png");
    assert_eq!(parsed_list.icons[0].mime_type.as_deref(), Some("image/png"));
    assert_eq!(parsed_list.icons[0].sizes, vec!["48x48".to_string()]);
    assert_eq!(parsed_list.icons[0].theme, Some(McpIconThemeOption::Light));
    let McpContextSubmitOptions::Explicit(submit) = parsed_list
        .context_submit
        .expect("context submit options should parse")
    else {
        panic!("context submit options should use the explicit form");
    };
    assert_eq!(
        submit.context.to_token_stream().to_string(),
        "crate :: SubmitContext"
    );
    assert_eq!(
        submit
            .response
            .expect("response should parse")
            .to_token_stream()
            .to_string(),
        "crate :: SubmitResponse"
    );
    assert_eq!(
        submit
            .error
            .expect("error should parse")
            .to_token_stream()
            .to_string(),
        "crate :: SubmitError"
    );
    assert_eq!(
        submit
            .submit
            .expect("submit should parse")
            .to_token_stream()
            .to_string(),
        "crate :: submit_request"
    );
    assert_eq!(
        submit
            .map_response
            .expect("map_response should parse")
            .to_token_stream()
            .to_string(),
        "crate :: map_response"
    );

    let parsed_context_only =
        McpToolOptions::from_list(&[parse_quote!(context(crate::SubmitContext))])
            .expect("context-only mcp list form should parse");
    let McpContextSubmitOptions::Explicit(context_only) = parsed_context_only
        .context_submit
        .expect("context submit options should parse")
    else {
        panic!("context submit options should use the explicit form");
    };
    assert_eq!(
        context_only.context.to_token_stream().to_string(),
        "crate :: SubmitContext"
    );
    assert!(context_only.response.is_none());
    assert!(context_only.error.is_none());
    assert!(context_only.submit.is_none());
    assert!(context_only.map_response.is_none());

    let parsed_submitter =
        McpToolOptions::from_list(&[parse_quote!(submitter(crate::traits::TmZmqMcpSubmit))])
            .expect("submitter-backed mcp list form should parse");
    let McpContextSubmitOptions::Submitter(submitter) = parsed_submitter
        .context_submit
        .expect("submitter options should parse")
    else {
        panic!("context submit options should use the submitter form");
    };
    assert_eq!(
        submitter.submitter.to_token_stream().to_string(),
        "crate :: traits :: TmZmqMcpSubmit"
    );
    assert!(submitter.response.is_none());
    assert!(submitter.map_response.is_none());

    let parsed_mapped_submitter = McpToolOptions::from_list(&[
        parse_quote!(submitter(crate::traits::TmZmqMcpSubmit)),
        parse_quote!(response(crate::SubmitResponse)),
        parse_quote!(map_response(crate::map_response)),
    ])
    .expect("mapped submitter mcp list form should parse");
    let McpContextSubmitOptions::Submitter(mapped_submitter) = parsed_mapped_submitter
        .context_submit
        .expect("mapped submitter options should parse")
    else {
        panic!("context submit options should use the submitter form");
    };
    assert_eq!(
        mapped_submitter
            .response
            .expect("mapped submitter response should parse")
            .to_token_stream()
            .to_string(),
        "crate :: SubmitResponse"
    );
    assert_eq!(
        mapped_submitter
            .map_response
            .expect("mapped submitter map_response should parse")
            .to_token_stream()
            .to_string(),
        "crate :: map_response"
    );
}

#[test]
fn mcp_tool_options_rejects_invalid_icons() {
    let error = McpToolOptions::from_list(&[parse_quote!(icon(mime_type = "image/png"))])
        .expect_err("missing icon src should fail");
    assert!(
        error.to_string().contains("src"),
        "unexpected error: {error}"
    );
}

#[test]
fn mcp_tool_options_rejects_unknown_fields() {
    let error = McpToolOptions::from_list(&[parse_quote!(invalid = "value")])
        .expect_err("invalid mcp option should fail");
    assert!(
        error.to_string().contains("Unknown field"),
        "unexpected error: {error}"
    );
}

#[test]
fn mcp_tool_options_rejects_invalid_name() {
    let error = McpToolOptions::from_list(&[parse_quote!(name = "bad name")])
        .expect_err("invalid mcp name should fail");

    assert!(
        error.to_string().contains("tool name may only contain"),
        "unexpected error: {error}"
    );
}

#[test]
fn mcp_tool_options_rejects_blank_metadata() {
    let error =
        McpToolOptions::from_list(&[parse_quote!(title = ""), parse_quote!(name = "ok-name")])
            .expect_err("empty title should fail");

    assert!(
        error.to_string().contains("tool title cannot be empty"),
        "unexpected error: {error}"
    );

    let error = McpToolOptions::from_list(&[
        parse_quote!(description = "   "),
        parse_quote!(name = "ok-name"),
    ])
    .expect_err("empty description should fail");

    assert!(
        error
            .to_string()
            .contains("tool description cannot be empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn mcp_tool_options_rejects_conflicting_annotation_hints() {
    let error = McpToolOptions::from_list(&[
        parse_quote!(read_only = true),
        parse_quote!(destructive = true),
    ])
    .expect_err("conflicting annotation hints should fail");

    assert!(
        error
            .to_string()
            .contains("cannot be both read-only and destructive"),
        "unexpected error: {error}"
    );
}

#[test]
fn mcp_tool_options_rejects_submitter_with_explicit_context_submit_options() {
    let error = McpToolOptions::from_list(&[
        parse_quote!(submitter(crate::traits::TmZmqMcpSubmit)),
        parse_quote!(context(crate::SubmitContext)),
    ])
    .expect_err("submitter and explicit context should conflict");

    assert!(
        error
            .to_string()
            .contains("`submitter(...)` cannot be combined"),
        "unexpected error: {error}"
    );

    let error = McpToolOptions::from_list(&[
        parse_quote!(submitter(crate::traits::TmZmqMcpSubmit)),
        parse_quote!(response(crate::SubmitResponse)),
    ])
    .expect_err("submitter response without map_response should fail");

    assert!(
        error
            .to_string()
            .contains("`response(...)` requires `map_response(...)`"),
        "unexpected error: {error}"
    );
}

#[test]
fn mcp_tool_options_accept_string_path_syntax_for_typed_context_handlers() {
    let parsed = McpToolOptions::from_list(&[
        parse_quote!(context = "crate::SubmitContext"),
        parse_quote!(response = "crate::SubmitResponse"),
        parse_quote!(error = "crate::SubmitError"),
        parse_quote!(submit = "crate::submit_request"),
        parse_quote!(map_response = "crate::map_response"),
        parse_quote!(icon(src = "icon.svg", sizes = "32x32", theme = "dark")),
    ])
    .expect("string path syntax should parse");

    let McpContextSubmitOptions::Explicit(context) = parsed.context_submit.unwrap() else {
        panic!("expected explicit context options");
    };
    assert_eq!(
        context.context.to_token_stream().to_string(),
        "crate :: SubmitContext"
    );
    assert_eq!(
        context.response.unwrap().to_token_stream().to_string(),
        "crate :: SubmitResponse"
    );
    assert_eq!(
        context.error.unwrap().to_token_stream().to_string(),
        "crate :: SubmitError"
    );
    assert_eq!(
        context.submit.unwrap().to_token_stream().to_string(),
        "crate :: submit_request"
    );
    assert_eq!(
        context.map_response.unwrap().to_token_stream().to_string(),
        "crate :: map_response"
    );
    assert_eq!(parsed.icons[0].theme, Some(McpIconThemeOption::Dark));

    let submitter = McpToolOptions::from_list(&[
        parse_quote!(submitter = "crate::Submitter"),
        parse_quote!(response = "crate::SubmitResponse"),
        parse_quote!(map_response = "crate::map_response"),
    ])
    .expect("string submitter syntax should parse");
    let McpContextSubmitOptions::Submitter(submitter) = submitter.context_submit.unwrap() else {
        panic!("expected submitter context options");
    };
    assert_eq!(
        submitter.submitter.to_token_stream().to_string(),
        "crate :: Submitter"
    );
}

#[test]
fn mcp_tool_options_reject_duplicates_and_orphan_context_options() {
    let duplicate =
        McpToolOptions::from_list(&[parse_quote!(name = "first"), parse_quote!(name = "second")])
            .expect_err("duplicate metadata should fail");
    assert!(duplicate.to_string().contains("Duplicate field: `name`"));

    for (items, expected) in [
        (
            vec![parse_quote!(response(crate::Response))],
            "`response(...)` requires `context(...)`",
        ),
        (
            vec![parse_quote!(error(crate::Error))],
            "`error(...)` requires `context(...)`",
        ),
        (
            vec![parse_quote!(submit(crate::submit))],
            "`submit(...)` requires `context(...)`",
        ),
        (
            vec![parse_quote!(map_response(crate::map))],
            "`map_response(...)` requires `context(...)`",
        ),
        (
            vec![
                parse_quote!(context(crate::Context)),
                parse_quote!(map_response(crate::map)),
            ],
            "`map_response(...)` requires `submit(...)`",
        ),
        (
            vec![
                parse_quote!(context(crate::Context)),
                parse_quote!(error(crate::Error)),
            ],
            "`error(...)` requires `submit(...)`",
        ),
    ] {
        let error = McpToolOptions::from_list(&items).expect_err("orphan option should fail");
        assert!(
            error.to_string().contains(expected),
            "expected {expected:?}, got {error}"
        );
    }
}

#[test]
fn mcp_icon_options_reject_invalid_metadata_and_unknown_fields() {
    for (meta, expected) in [
        (parse_quote!(icon(src = "icon", theme = "auto")), "theme"),
        (
            parse_quote!(icon(src = "icon", unknown = "value")),
            "Unknown field",
        ),
        (parse_quote!(icon(src = "")), "icon src cannot be empty"),
        (
            parse_quote!(icon(src = "icon", mime_type = " ")),
            "icon mime_type cannot be empty",
        ),
        (
            parse_quote!(icon(src = "icon", size = " ")),
            "icon size cannot be empty",
        ),
    ] {
        let error = McpToolOptions::from_list(&[meta]).expect_err("invalid icon should fail");
        assert!(
            error.to_string().contains(expected),
            "expected {expected:?}, got {error}"
        );
    }
}
