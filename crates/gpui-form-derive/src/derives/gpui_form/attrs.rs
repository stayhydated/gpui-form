use darling::{Error as DarlingError, FromMeta};
use proc_macro2::Span;
use quote::ToTokens as _;
use strum::EnumString;
use syn::{
    Expr, Ident, Lit, Meta, Path, Token, Type,
    parse::{Parse, ParseStream, Parser as _},
    punctuated::Punctuated,
};

use crate::derives::gpui_form::ir::{
    ConvertedValueIntent, DefaultExpr, RenderedFieldIntent, RenderedValueIntent, Spanned,
    TypeOverride,
};

mod mcp;

pub use mcp::{McpContextSubmitOptions, McpIconOptions, McpIconThemeOption, McpToolOptions};

mod kw {
    syn::custom_keyword!(component);
    syn::custom_keyword!(default);
    syn::custom_keyword!(description);
    syn::custom_keyword!(example);
    syn::custom_keyword!(from_source);
    syn::custom_keyword!(hidden);
    syn::custom_keyword!(into_source);
    syn::custom_keyword!(koruma_newtype);
    syn::custom_keyword!(label);
    syn::custom_keyword!(skip);
    syn::custom_keyword!(try_into_source);
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
            "expected `type = ...`, `from_source = ...`, `into_source = ...`, `try_into_source = ...`, or `koruma_newtype` in `value(...)`",
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
            if options.form_to_source.is_some() || options.try_form_to_source.is_some() {
                return Err(syn::Error::new_spanned(
                    key,
                    "duplicate reverse conversion option in `value(...)`; use only one of `into_source` or `try_into_source`",
                ));
            }
            options.form_to_source = Some(Spanned::new(input.parse()?, &key));
        } else if input.peek(kw::try_into_source) {
            let key = input.parse::<kw::try_into_source>()?;
            input.parse::<Token![=]>()?;
            if options.form_to_source.is_some() || options.try_form_to_source.is_some() {
                return Err(syn::Error::new_spanned(
                    key,
                    "duplicate reverse conversion option in `value(...)`; use only one of `into_source` or `try_into_source`",
                ));
            }
            options.try_form_to_source = Some(Spanned::new(input.parse()?, &key));
        } else if input.peek(kw::koruma_newtype) {
            let key = input.parse::<kw::koruma_newtype>()?;
            if options.koruma_newtype_span.is_some() {
                return Err(syn::Error::new_spanned(
                    key,
                    "duplicate `koruma_newtype` option in `value(...)`; remove the duplicate entry",
                ));
            }
            options.koruma_newtype_span = Some(key.span);
        } else {
            return Err(input.error(
                "expected `type = ...`, `from_source = ...`, `into_source = ...`, `try_into_source = ...`, or `koruma_newtype` in `value(...)`",
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
    try_form_to_source: Option<Spanned<Expr>>,
    source_to_form: Option<Spanned<Expr>>,
    koruma_newtype_span: Option<Span>,
}

impl ParsedValueOptions {
    fn into_intent(self) -> syn::Result<RenderedValueIntent> {
        if let Some(span) = self.koruma_newtype_span {
            if self.r#type.is_some()
                || self.source_to_form.is_some()
                || self.form_to_source.is_some()
                || self.try_form_to_source.is_some()
            {
                return Err(syn::Error::new(
                    span,
                    "`value(koruma_newtype)` cannot be combined with `type`, `from_source`, `into_source`, or `try_into_source`",
                ));
            }
            return Ok(RenderedValueIntent::KorumaNewtype { span });
        }

        match (
            self.r#type,
            self.source_to_form,
            self.form_to_source,
            self.try_form_to_source,
        ) {
            (None, None, None, None) => Err(syn::Error::new(
                Span::call_site(),
                "expected `type = ...`, `from_source = ...`, `into_source = ...`, `try_into_source = ...`, or `koruma_newtype` in `value(...)`",
            )),
            (Some(form_type), Some(from_source), Some(into_source), None) => {
                reject_option_form_type_override(&form_type.value.0, form_type.span)?;
                Ok(RenderedValueIntent::Converted(Box::new(
                    ConvertedValueIntent {
                        form_type,
                        from_source,
                        into_source,
                        into_source_is_fallible: false,
                    },
                )))
            },
            (Some(form_type), Some(from_source), None, Some(into_source)) => {
                reject_option_form_type_override(&form_type.value.0, form_type.span)?;
                Ok(RenderedValueIntent::Converted(Box::new(
                    ConvertedValueIntent {
                        form_type,
                        from_source,
                        into_source,
                        into_source_is_fallible: true,
                    },
                )))
            },
            (Some(_), Some(_), Some(into_source), Some(_)) => Err(syn::Error::new(
                into_source.span,
                "`value(...)` accepts only one reverse conversion; use `into_source = ...` for infallible conversion or `try_into_source = ...` for fallible conversion",
            )),
            (Some(form_type), None, _, _) => Err(syn::Error::new(
                form_type.span,
                "`value(type = ...)` requires an explicit `from_source = ...` conversion",
            )),
            (Some(form_type), _, None, None) => Err(syn::Error::new(
                form_type.span,
                "`value(type = ...)` requires an explicit `into_source = ...` or `try_into_source = ...` conversion",
            )),
            (None, Some(from_source), _, _) => Err(syn::Error::new(
                from_source.span,
                "`from_source = ...` requires `type = ...` in the same `value(...)` option",
            )),
            (None, _, Some(into_source), _) => Err(syn::Error::new(
                into_source.span,
                "`into_source = ...` requires `type = ...` in the same `value(...)` option",
            )),
            (None, _, _, Some(into_source)) => Err(syn::Error::new(
                into_source.span,
                "`try_into_source = ...` requires `type = ...` in the same `value(...)` option",
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
mod tests;
