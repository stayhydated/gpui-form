use darling::{Error as DarlingError, FromMeta};
use proc_macro2::Span;
use quote::ToTokens as _;
use syn::{
    Expr, Ident, Lit, Meta, Token, Type,
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
};

use crate::derives::gpui_form::ir::{
    ConvertedValueIntent, DefaultExpr, RenderedFieldIntent, RenderedValueIntent, Spanned,
    TypeOverride,
};

mod kw {
    syn::custom_keyword!(component);
    syn::custom_keyword!(default);
    syn::custom_keyword!(description);
    syn::custom_keyword!(example);
    syn::custom_keyword!(from_source);
    syn::custom_keyword!(hidden);
    syn::custom_keyword!(into_source);
    syn::custom_keyword!(label);
    syn::custom_keyword!(skip);
    syn::custom_keyword!(value);
}

#[derive(Clone, Debug)]
pub struct EmptyForm;

impl FromMeta for EmptyForm {
    fn from_word() -> darling::Result<Self> {
        Ok(Self)
    }
}

#[derive(Clone, Debug)]
pub struct NoInventory;

impl FromMeta for NoInventory {
    fn from_word() -> darling::Result<Self> {
        Ok(Self)
    }
}

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
    pub task_support: Option<McpTaskSupportOption>,
    pub context_submit: Option<McpContextSubmitOptions>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpIconOptions {
    pub src: String,
    pub mime_type: Option<String>,
    pub sizes: Vec<String>,
    pub theme: Option<McpIconThemeOption>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum McpIconThemeOption {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum McpTaskSupportOption {
    Forbidden,
    Optional,
    Required,
}

#[derive(Clone, Debug)]
pub struct McpContextSubmitOptions {
    pub context: Type,
    pub response: Option<Type>,
    pub error: Option<Type>,
    pub submit: Option<Expr>,
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
            Meta::NameValue(name_value) if name_value.path.is_ident("task_support") => {
                assign_once(
                    "task_support",
                    &mut options.task_support,
                    parse_mcp_task_support(String::from_meta(meta)?)?,
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

    if context.is_none() {
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
    if submit.is_none() && map_response.is_some() {
        return Err(DarlingError::custom(
            "`map_response(...)` requires `submit(...)` in the same `gpui_form(mcp(...))` option",
        ));
    }
    if submit.is_none() && error.is_some() {
        return Err(DarlingError::custom(
            "`error(...)` requires `submit(...)` in the same `gpui_form(mcp(...))` option",
        ));
    }

    options.context_submit = context.map(|context| McpContextSubmitOptions {
        context,
        response,
        error,
        submit,
        map_response,
    });

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
    match value.as_str() {
        "light" => Ok(McpIconThemeOption::Light),
        "dark" => Ok(McpIconThemeOption::Dark),
        _ => Err(DarlingError::custom(
            "`theme` must be either \"light\" or \"dark\"",
        )),
    }
}

fn parse_mcp_task_support(value: String) -> darling::Result<McpTaskSupportOption> {
    match value.as_str() {
        "forbidden" => Ok(McpTaskSupportOption::Forbidden),
        "optional" => Ok(McpTaskSupportOption::Optional),
        "required" => Ok(McpTaskSupportOption::Required),
        _ => Err(DarlingError::custom(
            "`task_support` must be \"forbidden\", \"optional\", or \"required\"",
        )),
    }
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

#[derive(Debug)]
pub(super) enum GpuiFormFieldOption {
    Label {
        span: Span,
        value: String,
    },
    Description {
        span: Span,
        value: String,
    },
    Example {
        value: String,
    },
    Skip {
        span: Span,
    },
    Hidden {
        span: Span,
        options: Box<RenderedFieldIntent>,
    },
    Component {
        span: Span,
        shape: Box<Expr>,
        options: Box<RenderedFieldIntent>,
    },
}

impl Parse for GpuiFormFieldOption {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(kw::label) {
            let key = input.parse::<kw::label>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::Label {
                span: key.span,
                value: parse_metadata_string(input, "label")?,
            });
        }

        if input.peek(kw::description) {
            let key = input.parse::<kw::description>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::Description {
                span: key.span,
                value: parse_metadata_string(input, "description")?,
            });
        }

        if input.peek(kw::example) {
            input.parse::<kw::example>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::Example {
                value: parse_metadata_string(input, "example")?,
            });
        }

        if input.peek(kw::component) {
            let key = input.parse::<kw::component>()?;
            let content;
            syn::parenthesized!(content in input);
            let shape = content.parse()?;
            let options = parse_structured_field_options(&content, true)?;
            return Ok(Self::Component {
                span: key.span,
                shape: Box::new(shape),
                options: Box::new(options),
            });
        }

        if input.peek(kw::skip) {
            let key = input.parse::<kw::skip>()?;
            return Ok(Self::Skip { span: key.span });
        }

        if input.peek(kw::hidden) {
            let key = input.parse::<kw::hidden>()?;
            let options = if input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                parse_structured_field_options(&content, false)?
            } else {
                RenderedFieldIntent::default()
            };
            return Ok(Self::Hidden {
                span: key.span,
                options: Box::new(options),
            });
        }

        if input.peek(Ident) {
            let fork = input.fork();
            let key: Ident = fork.parse()?;

            return Err(syn::Error::new_spanned(
                key.clone(),
                format!(
                    "unknown gpui_form field option `{key}`; component-backed fields must use \
                     `component(MyShape)` where `MyShape` is declared with \
                     `component_shape_gpui::component_shape!` or \
                     `#[derive(component_shape_gpui::GpuiComponentShape)]`"
                ),
            ));
        }

        Err(input.error(
            "expected gpui_form field option such as `component(MyShape)`, `hidden`, `skip`, `label = ...`, `description = ...`, or `example = ...`",
        ))
    }
}

fn parse_metadata_string(input: ParseStream<'_>, field: &str) -> syn::Result<String> {
    let value = input.parse::<syn::LitStr>()?;
    let value = value.value();
    component_shape::validate_mcp_tool_metadata_text(field, &value)
        .map_err(|error| input.error(error.to_string()))?;
    Ok(value)
}

fn parse_structured_field_options(
    input: ParseStream<'_>,
    needs_leading_comma: bool,
) -> syn::Result<RenderedFieldIntent> {
    let mut options = ParsedFieldIntentOptions::default();
    let mut first = true;
    while !input.is_empty() {
        if needs_leading_comma || !first {
            input.parse::<Token![,]>()?;
        }
        first = false;
        if input.peek(kw::value) {
            let key = input.parse::<kw::value>()?;
            let content;
            syn::parenthesized!(content in input);
            if options.value.is_some() {
                return Err(syn::Error::new_spanned(
                    key,
                    "duplicate `value(...)` option in structured `gpui_form` field option; remove the duplicate `value(...)` entry",
                ));
            }
            options.value = Some(parse_structured_value_options(&content)?);
            continue;
        }
        if input.peek(kw::default) {
            let key = input.parse::<kw::default>()?;
            input.parse::<Token![=]>()?;
            if options.default.is_some() {
                return Err(syn::Error::new_spanned(
                    key,
                    "duplicate `default` option in structured `gpui_form` field option; remove the duplicate `default` entry",
                ));
            }
            options.default = Some(Spanned::new(DefaultExpr(input.parse()?), &key));
            continue;
        }
        return Err(input.error(
            "expected `value(...)` or `default = ...` in structured `gpui_form` field option",
        ));
    }
    Ok(RenderedFieldIntent {
        value: options.value.unwrap_or(RenderedValueIntent::Identity),
        default: options.default,
    })
}

fn parse_structured_value_options(input: ParseStream<'_>) -> syn::Result<RenderedValueIntent> {
    if input.is_empty() {
        return Err(input.error(
            "expected `type = ...`, `from_source = ...`, or `into_source = ...` in `value(...)`",
        ));
    }

    let mut options = ParsedValueOptions::default();
    while !input.is_empty() {
        if input.peek(Token![type]) {
            let key = input.parse::<Token![type]>()?;
            input.parse::<Token![=]>()?;
            if options.r#type.is_some() {
                return Err(syn::Error::new_spanned(
                    key,
                    "duplicate `type` option in `value(...)`; remove the duplicate `type` entry",
                ));
            }
            options.r#type = Some(Spanned::new(TypeOverride(input.parse()?), &key));
        } else if input.peek(kw::from_source) {
            let key = input.parse::<kw::from_source>()?;
            input.parse::<Token![=]>()?;
            if options.source_to_form.is_some() {
                return Err(syn::Error::new_spanned(
                    key,
                    "duplicate `from_source` option in `value(...)`; remove the duplicate `from_source` entry",
                ));
            }
            options.source_to_form = Some(Spanned::new(input.parse()?, &key));
        } else if input.peek(kw::into_source) {
            let key = input.parse::<kw::into_source>()?;
            input.parse::<Token![=]>()?;
            if options.form_to_source.is_some() {
                return Err(syn::Error::new_spanned(
                    key,
                    "duplicate `into_source` option in `value(...)`; remove the duplicate `into_source` entry",
                ));
            }
            options.form_to_source = Some(Spanned::new(input.parse()?, &key));
        } else {
            return Err(input.error(
                "expected `type = ...`, `from_source = ...`, or `into_source = ...` in `value(...)`",
            ));
        }

        if input.is_empty() {
            break;
        }
        input.parse::<Token![,]>()?;
    }

    options.into_intent()
}

#[derive(Clone, Debug, Default)]
struct ParsedFieldIntentOptions {
    value: Option<RenderedValueIntent>,
    default: Option<Spanned<DefaultExpr>>,
}

#[derive(Clone, Debug, Default)]
struct ParsedValueOptions {
    r#type: Option<Spanned<TypeOverride>>,
    form_to_source: Option<Spanned<Expr>>,
    source_to_form: Option<Spanned<Expr>>,
}

impl ParsedValueOptions {
    fn into_intent(self) -> syn::Result<RenderedValueIntent> {
        match (self.r#type, self.source_to_form, self.form_to_source) {
            (None, None, None) => Err(syn::Error::new(
                Span::call_site(),
                "expected `type = ...`, `from_source = ...`, or `into_source = ...` in `value(...)`",
            )),
            (Some(form_type), Some(from_source), Some(into_source)) => {
                reject_option_form_type_override(&form_type.value.0, form_type.span)?;
                Ok(RenderedValueIntent::Converted(Box::new(
                    ConvertedValueIntent {
                        form_type,
                        from_source,
                        into_source,
                    },
                )))
            },
            (Some(form_type), None, _) => Err(syn::Error::new(
                form_type.span,
                "`value(type = ...)` requires an explicit `from_source = ...` conversion",
            )),
            (Some(form_type), _, None) => Err(syn::Error::new(
                form_type.span,
                "`value(type = ...)` requires an explicit `into_source = ...` conversion",
            )),
            (None, Some(from_source), _) => Err(syn::Error::new(
                from_source.span,
                "`from_source = ...` requires `type = ...` in the same `value(...)` option",
            )),
            (None, _, Some(into_source)) => Err(syn::Error::new(
                into_source.span,
                "`into_source = ...` requires `type = ...` in the same `value(...)` option",
            )),
        }
    }
}

fn reject_option_form_type_override(ty: &Type, span: Span) -> syn::Result<()> {
    let ty = peel_type_wrappers(ty);
    if let Type::Path(type_path) = ty
        && type_path.qself.is_none()
        && type_path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "Option")
    {
        return Err(syn::Error::new(
            span,
            "`value(type = ...)` expects the form-side base value type, not Option<T>; use type = T and let the source field optionality control holder storage",
        ));
    }

    Ok(())
}

fn peel_type_wrappers(mut ty: &Type) -> &Type {
    loop {
        match ty {
            Type::Group(group) => ty = &group.elem,
            Type::Paren(paren) => ty = &paren.elem,
            _ => return ty,
        }
    }
}

#[cfg(test)]
mod tests {
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
    fn raw_field_options_reject_unknown_option() {
        let error = parse_options(quote! { field_suffix = "input" })
            .expect_err("unknown options should fail");

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
            error
                .to_string()
                .contains("requires an explicit `into_source = ...` conversion"),
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
            parse_quote!(task_support = "optional"),
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
        assert_eq!(
            parsed_list.task_support,
            Some(McpTaskSupportOption::Optional)
        );
        assert_eq!(parsed_list.icons.len(), 1);
        assert_eq!(parsed_list.icons[0].src, "https://example.com/contact.png");
        assert_eq!(parsed_list.icons[0].mime_type.as_deref(), Some("image/png"));
        assert_eq!(parsed_list.icons[0].sizes, vec!["48x48".to_string()]);
        assert_eq!(parsed_list.icons[0].theme, Some(McpIconThemeOption::Light));
        let submit = parsed_list
            .context_submit
            .expect("context submit options should parse");
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
        let context_only = parsed_context_only
            .context_submit
            .expect("context submit options should parse");
        assert_eq!(
            context_only.context.to_token_stream().to_string(),
            "crate :: SubmitContext"
        );
        assert!(context_only.response.is_none());
        assert!(context_only.error.is_none());
        assert!(context_only.submit.is_none());
        assert!(context_only.map_response.is_none());
    }

    #[test]
    fn mcp_tool_options_rejects_invalid_icon_and_task_support() {
        let error = McpToolOptions::from_list(&[parse_quote!(task_support = "sometimes")])
            .expect_err("invalid task support should fail");
        assert!(
            error
                .to_string()
                .contains("task_support` must be \"forbidden\", \"optional\", or \"required\""),
            "unexpected error: {error}"
        );

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
}
