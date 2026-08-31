use super::*;

#[derive(Clone, Debug, Default)]
pub struct McpToolOptions {
    pub name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub read_only: Option<bool>,
    pub destructive: Option<bool>,
    pub idempotent: Option<bool>,
    pub open_world: Option<bool>,
    pub icons: Vec<McpIconOptions>,
    pub context_submit: Option<McpContextSubmitOptions>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpIconOptions {
    pub src: String,
    pub mime_type: Option<String>,
    pub sizes: Vec<String>,
    pub theme: Option<McpIconThemeOption>,
}

#[derive(Clone, Copy, Debug, EnumString, Eq, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum McpIconThemeOption {
    Light,
    Dark,
}

#[derive(Clone, Debug)]
pub enum McpContextSubmitOptions {
    Explicit(Box<McpExplicitContextSubmitOptions>),
    Submitter(Box<McpSubmitterOptions>),
}

#[derive(Clone, Debug)]
pub struct McpExplicitContextSubmitOptions {
    pub context: Type,
    pub response: Option<Type>,
    pub error: Option<Type>,
    pub submit: Option<Expr>,
    pub map_response: Option<Expr>,
}

#[derive(Clone, Debug)]
pub struct McpSubmitterOptions {
    pub submitter: Path,
    pub response: Option<Type>,
    pub map_response: Option<Expr>,
}

impl FromMeta for McpToolOptions {
    fn from_word() -> darling::Result<Self> {
        Ok(Self::default())
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let options = parse_mcp_tool_options(items)?;
        validate_mcp_tool_options(&options)?;
        Ok(options)
    }
}

fn parse_mcp_tool_options(items: &[darling::ast::NestedMeta]) -> darling::Result<McpToolOptions> {
    let mut options = McpToolOptions::default();
    let mut context = None;
    let mut response = None;
    let mut error = None;
    let mut submit = None;
    let mut map_response = None;
    let mut submitter = None;

    for item in items {
        let darling::ast::NestedMeta::Meta(meta) = item else {
            return Err(DarlingError::unsupported_format("literal"));
        };

        match meta {
            Meta::NameValue(name_value) if name_value.path.is_ident("name") => {
                assign_once(
                    "name",
                    &mut options.name,
                    String::from_meta(meta)?,
                    name_value.path.to_token_stream(),
                )?;
            },
            Meta::NameValue(name_value) if name_value.path.is_ident("title") => {
                assign_once(
                    "title",
                    &mut options.title,
                    String::from_meta(meta)?,
                    name_value.path.to_token_stream(),
                )?;
            },
            Meta::NameValue(name_value) if name_value.path.is_ident("description") => {
                assign_once(
                    "description",
                    &mut options.description,
                    String::from_meta(meta)?,
                    name_value.path.to_token_stream(),
                )?;
            },
            Meta::NameValue(name_value) if name_value.path.is_ident("read_only") => {
                assign_once(
                    "read_only",
                    &mut options.read_only,
                    bool::from_meta(meta)?,
                    name_value.path.to_token_stream(),
                )?;
            },
            Meta::NameValue(name_value) if name_value.path.is_ident("destructive") => {
                assign_once(
                    "destructive",
                    &mut options.destructive,
                    bool::from_meta(meta)?,
                    name_value.path.to_token_stream(),
                )?;
            },
            Meta::NameValue(name_value) if name_value.path.is_ident("idempotent") => {
                assign_once(
                    "idempotent",
                    &mut options.idempotent,
                    bool::from_meta(meta)?,
                    name_value.path.to_token_stream(),
                )?;
            },
            Meta::NameValue(name_value) if name_value.path.is_ident("open_world") => {
                assign_once(
                    "open_world",
                    &mut options.open_world,
                    bool::from_meta(meta)?,
                    name_value.path.to_token_stream(),
                )?;
            },
            Meta::List(list) if list.path.is_ident("icon") => {
                options.icons.push(parse_mcp_icon_options(list)?);
            },
            Meta::List(list) if list.path.is_ident("context") => {
                assign_once(
                    "context",
                    &mut context,
                    syn::parse2::<Type>(list.tokens.clone())
                        .map_err(|error| DarlingError::custom(error.to_string()))?,
                    list.path.to_token_stream(),
                )?;
            },
            Meta::List(list) if list.path.is_ident("response") => {
                assign_once(
                    "response",
                    &mut response,
                    syn::parse2::<Type>(list.tokens.clone())
                        .map_err(|error| DarlingError::custom(error.to_string()))?,
                    list.path.to_token_stream(),
                )?;
            },
            Meta::List(list) if list.path.is_ident("error") => {
                assign_once(
                    "error",
                    &mut error,
                    syn::parse2::<Type>(list.tokens.clone())
                        .map_err(|error| DarlingError::custom(error.to_string()))?,
                    list.path.to_token_stream(),
                )?;
            },
            Meta::List(list) if list.path.is_ident("submit") => {
                assign_once(
                    "submit",
                    &mut submit,
                    syn::parse2::<Expr>(list.tokens.clone())
                        .map_err(|error| DarlingError::custom(error.to_string()))?,
                    list.path.to_token_stream(),
                )?;
            },
            Meta::List(list) if list.path.is_ident("map_response") => {
                assign_once(
                    "map_response",
                    &mut map_response,
                    syn::parse2::<Expr>(list.tokens.clone())
                        .map_err(|error| DarlingError::custom(error.to_string()))?,
                    list.path.to_token_stream(),
                )?;
            },
            Meta::List(list) if list.path.is_ident("submitter") => {
                assign_once(
                    "submitter",
                    &mut submitter,
                    syn::parse2::<Path>(list.tokens.clone())
                        .map_err(|error| DarlingError::custom(error.to_string()))?,
                    list.path.to_token_stream(),
                )?;
            },
            Meta::NameValue(name_value) if name_value.path.is_ident("context") => {
                assign_once(
                    "context",
                    &mut context,
                    parse_lit_str_type(meta, "context")?,
                    name_value.path.to_token_stream(),
                )?;
            },
            Meta::NameValue(name_value) if name_value.path.is_ident("error") => {
                assign_once(
                    "error",
                    &mut error,
                    parse_lit_str_type(meta, "error")?,
                    name_value.path.to_token_stream(),
                )?;
            },
            Meta::NameValue(name_value) if name_value.path.is_ident("submit") => {
                assign_once(
                    "submit",
                    &mut submit,
                    parse_lit_str_expr(meta, "submit")?,
                    name_value.path.to_token_stream(),
                )?;
            },
            Meta::NameValue(name_value) if name_value.path.is_ident("map_response") => {
                assign_once(
                    "map_response",
                    &mut map_response,
                    parse_lit_str_expr(meta, "map_response")?,
                    name_value.path.to_token_stream(),
                )?;
            },
            Meta::NameValue(name_value) if name_value.path.is_ident("submitter") => {
                assign_once(
                    "submitter",
                    &mut submitter,
                    parse_lit_str_path(meta, "submitter")?,
                    name_value.path.to_token_stream(),
                )?;
            },
            Meta::NameValue(name_value) if name_value.path.is_ident("response") => {
                assign_once(
                    "response",
                    &mut response,
                    parse_lit_str_type(meta, "response")?,
                    name_value.path.to_token_stream(),
                )?;
            },
            Meta::Path(path)
            | Meta::List(syn::MetaList { path, .. })
            | Meta::NameValue(syn::MetaNameValue { path, .. }) => {
                return Err(DarlingError::custom(format!(
                    "Unknown field: `{}`",
                    path.to_token_stream()
                )));
            },
        }
    }

    if submitter.is_some() && (context.is_some() || error.is_some() || submit.is_some()) {
        return Err(DarlingError::custom(
            "`submitter(...)` cannot be combined with `context(...)`, `error(...)`, or `submit(...)`",
        ));
    }
    if submitter.is_some() && response.is_some() && map_response.is_none() {
        return Err(DarlingError::custom(
            "`response(...)` requires `map_response(...)` when used with `submitter(...)`",
        ));
    }

    if context.is_none() && submitter.is_none() {
        if response.is_some() {
            return Err(DarlingError::custom(
                "`response(...)` requires `context(...)` in the same `gpui_form(mcp(...))` option",
            ));
        }
        if error.is_some() {
            return Err(DarlingError::custom(
                "`error(...)` requires `context(...)` in the same `gpui_form(mcp(...))` option",
            ));
        }
        if submit.is_some() {
            return Err(DarlingError::custom(
                "`submit(...)` requires `context(...)` in the same `gpui_form(mcp(...))` option",
            ));
        }
        if map_response.is_some() {
            return Err(DarlingError::custom(
                "`map_response(...)` requires `context(...)` in the same `gpui_form(mcp(...))` option",
            ));
        }
    }
    if submitter.is_none() && submit.is_none() && map_response.is_some() {
        return Err(DarlingError::custom(
            "`map_response(...)` requires `submit(...)` in the same `gpui_form(mcp(...))` option",
        ));
    }
    if submit.is_none() && error.is_some() {
        return Err(DarlingError::custom(
            "`error(...)` requires `submit(...)` in the same `gpui_form(mcp(...))` option",
        ));
    }

    options.context_submit = if let Some(submitter) = submitter {
        Some(McpContextSubmitOptions::Submitter(Box::new(
            McpSubmitterOptions {
                submitter,
                response,
                map_response,
            },
        )))
    } else {
        context.map(|context| {
            McpContextSubmitOptions::Explicit(Box::new(McpExplicitContextSubmitOptions {
                context,
                response,
                error,
                submit,
                map_response,
            }))
        })
    };

    Ok(options)
}

fn parse_mcp_icon_options(list: &syn::MetaList) -> darling::Result<McpIconOptions> {
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let items = parser
        .parse2(list.tokens.clone())
        .map_err(|error| DarlingError::custom(error.to_string()))?;
    let mut src = None;
    let mut mime_type = None;
    let mut sizes = Vec::new();
    let mut theme = None;

    for meta in items {
        match &meta {
            Meta::NameValue(name_value) if name_value.path.is_ident("src") => {
                assign_once(
                    "src",
                    &mut src,
                    String::from_meta(&meta)?,
                    name_value.path.to_token_stream(),
                )?;
            },
            Meta::NameValue(name_value) if name_value.path.is_ident("mime_type") => {
                assign_once(
                    "mime_type",
                    &mut mime_type,
                    String::from_meta(&meta)?,
                    name_value.path.to_token_stream(),
                )?;
            },
            Meta::NameValue(name_value)
                if name_value.path.is_ident("size") || name_value.path.is_ident("sizes") =>
            {
                sizes.push(String::from_meta(&meta)?);
            },
            Meta::NameValue(name_value) if name_value.path.is_ident("theme") => {
                assign_once(
                    "theme",
                    &mut theme,
                    parse_mcp_icon_theme(String::from_meta(&meta)?)?,
                    name_value.path.to_token_stream(),
                )?;
            },
            _ => {
                return Err(DarlingError::custom(format!(
                    "Unknown field: `{}`",
                    meta.path().to_token_stream()
                )));
            },
        }
    }

    let src = src.ok_or_else(|| DarlingError::missing_field("src"))?;
    let icon = McpIconOptions {
        src,
        mime_type,
        sizes,
        theme,
    };
    validate_mcp_icon_options(&icon)?;
    Ok(icon)
}

fn parse_mcp_icon_theme(value: String) -> darling::Result<McpIconThemeOption> {
    value
        .parse()
        .map_err(|_| DarlingError::custom("`theme` must be either \"light\" or \"dark\""))
}

fn assign_once<T>(
    field: &str,
    target: &mut Option<T>,
    value: T,
    tokens: proc_macro2::TokenStream,
) -> darling::Result<()> {
    if target.is_some() {
        return Err(DarlingError::custom(format!("Duplicate field: `{field}`")).with_span(&tokens));
    }

    *target = Some(value);
    Ok(())
}

fn parse_lit_str_type(meta: &Meta, field: &str) -> darling::Result<Type> {
    let Meta::NameValue(name_value) = meta else {
        return Err(DarlingError::unsupported_format(field));
    };
    let Expr::Lit(expr_lit) = &name_value.value else {
        return Err(DarlingError::custom(format!(
            "`{field} = ...` expects a string literal type; prefer `{field}(Type)`"
        )));
    };
    let Lit::Str(value) = &expr_lit.lit else {
        return Err(DarlingError::custom(format!(
            "`{field} = ...` expects a string literal type; prefer `{field}(Type)`"
        )));
    };

    value
        .parse::<Type>()
        .map_err(|error| DarlingError::custom(error.to_string()))
}

fn parse_lit_str_expr(meta: &Meta, field: &str) -> darling::Result<Expr> {
    let Meta::NameValue(name_value) = meta else {
        return Err(DarlingError::unsupported_format(field));
    };
    let Expr::Lit(expr_lit) = &name_value.value else {
        return Err(DarlingError::custom(format!(
            "`{field} = ...` expects a string literal expression; prefer `{field}(path::to::function)`"
        )));
    };
    let Lit::Str(value) = &expr_lit.lit else {
        return Err(DarlingError::custom(format!(
            "`{field} = ...` expects a string literal expression; prefer `{field}(path::to::function)`"
        )));
    };

    value
        .parse::<Expr>()
        .map_err(|error| DarlingError::custom(error.to_string()))
}

fn parse_lit_str_path(meta: &Meta, field: &str) -> darling::Result<Path> {
    let Meta::NameValue(name_value) = meta else {
        return Err(DarlingError::unsupported_format(field));
    };
    let Expr::Lit(expr_lit) = &name_value.value else {
        return Err(DarlingError::custom(format!(
            "`{field} = ...` expects a string literal path; prefer `{field}(path::to::Trait)`"
        )));
    };
    let Lit::Str(value) = &expr_lit.lit else {
        return Err(DarlingError::custom(format!(
            "`{field} = ...` expects a string literal path; prefer `{field}(path::to::Trait)`"
        )));
    };

    value
        .parse::<Path>()
        .map_err(|error| DarlingError::custom(error.to_string()))
}

fn validate_mcp_tool_options(options: &McpToolOptions) -> darling::Result<()> {
    if options.read_only == Some(true) && options.destructive == Some(true) {
        return Err(DarlingError::custom(
            "MCP tool annotation hints cannot be both read-only and destructive",
        ));
    }
    if let Some(name) = options.name.as_deref() {
        component_shape::validate_mcp_tool_name(name)
            .map_err(|error| DarlingError::custom(error.to_string()))?;
    }
    if let Some(title) = options.title.as_deref() {
        component_shape::validate_mcp_tool_metadata_text("title", title)
            .map_err(|error| DarlingError::custom(error.to_string()))?;
    }
    if let Some(description) = options.description.as_deref() {
        component_shape::validate_mcp_tool_metadata_text("description", description)
            .map_err(|error| DarlingError::custom(error.to_string()))?;
    }
    for icon in &options.icons {
        validate_mcp_icon_options(icon)?;
    }
    Ok(())
}

fn validate_mcp_icon_options(icon: &McpIconOptions) -> darling::Result<()> {
    component_shape::validate_mcp_tool_metadata_text("icon src", &icon.src)
        .map_err(|error| DarlingError::custom(error.to_string()))?;
    if let Some(mime_type) = icon.mime_type.as_deref() {
        component_shape::validate_mcp_tool_metadata_text("icon mime_type", mime_type)
            .map_err(|error| DarlingError::custom(error.to_string()))?;
    }
    for size in &icon.sizes {
        component_shape::validate_mcp_tool_metadata_text("icon size", size)
            .map_err(|error| DarlingError::custom(error.to_string()))?;
    }
    Ok(())
}
