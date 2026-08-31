use super::*;

pub(super) fn tool_metadata_tokens(
    context: &DeriveContext,
    original_input: &DeriveInput,
    options: Option<&McpToolOptions>,
) -> McpResult<TokenStream> {
    let facade_crate = &context.paths.gpui_form;
    let mcp_crate: Path = syn::parse_quote!(#facade_crate::mcp);
    let span = original_input.ident.span();
    let mut tokens = mcp_tool_metadata_tokens(
        &mcp_crate,
        &original_input.attrs,
        McpToolMetadataParts {
            name: options.and_then(|options| options.name.as_deref()),
            title: options.and_then(|options| options.title.as_deref()),
            description: options.and_then(|options| options.description.as_deref()),
            read_only: options.and_then(|options| options.read_only),
            destructive: options.and_then(|options| options.destructive),
            idempotent: options.and_then(|options| options.idempotent),
            open_world: options.and_then(|options| options.open_world),
        },
        span,
    )?;

    if let Some(options) = options
        && !options.icons.is_empty()
    {
        let icons = options
            .icons
            .iter()
            .map(|icon| icon_tokens(&mcp_crate, icon, span))
            .collect::<Vec<_>>();
        tokens = quote! {{
            const MCP_TOOL_ICONS: &[#mcp_crate::McpToolIcon] = &[#(#icons),*];
            (#tokens).with_icons(MCP_TOOL_ICONS)
        }};
    }

    Ok(tokens)
}

fn icon_tokens(
    mcp_crate: &Path,
    icon: &crate::derives::gpui_form::attrs::McpIconOptions,
    span: Span,
) -> TokenStream {
    let src = LitStr::new(&icon.src, span);
    let mut tokens = quote! { #mcp_crate::McpToolIcon::new(#src) };
    if let Some(mime_type) = icon.mime_type.as_deref() {
        let mime_type = LitStr::new(mime_type, span);
        tokens = quote! { #tokens.with_mime_type(#mime_type) };
    }
    if !icon.sizes.is_empty() {
        let sizes = icon
            .sizes
            .iter()
            .map(|size| LitStr::new(size, span))
            .collect::<Vec<_>>();
        tokens = quote! { #tokens.with_sizes(&[#(#sizes),*]) };
    }
    if let Some(theme) = icon.theme {
        let theme = match theme {
            McpIconThemeOption::Light => quote! { #mcp_crate::McpIconTheme::Light },
            McpIconThemeOption::Dark => quote! { #mcp_crate::McpIconTheme::Dark },
        };
        tokens = quote! { #tokens.with_theme(#theme) };
    }
    tokens
}
