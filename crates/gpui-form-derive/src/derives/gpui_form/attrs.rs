use darling::FromMeta;
use proc_macro2::Span;
use syn::{
    Expr, Ident, Path, Token, Type,
    parse::{Parse, ParseStream},
};

use crate::derives::gpui_form::ir::{
    ConvertedValueIntent, DefaultExpr, RenderedFieldIntent, RenderedValueIntent, Spanned,
    TypeOverride,
};

mod kw {
    syn::custom_keyword!(component);
    syn::custom_keyword!(default);
    syn::custom_keyword!(from_source);
    syn::custom_keyword!(hidden);
    syn::custom_keyword!(into_source);
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

#[derive(Debug)]
pub(super) enum GpuiFormFieldOption {
    Skip {
        span: Span,
    },
    Hidden {
        span: Span,
        options: RenderedFieldIntent,
    },
    Component {
        span: Span,
        shape: Path,
        options: RenderedFieldIntent,
    },
}

impl Parse for GpuiFormFieldOption {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(kw::component) {
            let key = input.parse::<kw::component>()?;
            let content;
            syn::parenthesized!(content in input);
            let shape = content.parse::<Path>()?;
            let options = parse_structured_field_options(&content, true)?;
            return Ok(Self::Component {
                span: key.span,
                shape,
                options,
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
                options,
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
            "expected gpui_form field option such as `component(MyShape)`, `hidden`, or `skip`",
        ))
    }
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
                Ok(RenderedValueIntent::Converted(ConvertedValueIntent {
                    form_type,
                    from_source,
                    into_source,
                }))
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
    use syn::{parse::Parser as _, punctuated::Punctuated};

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
        assert!(matches!(options.value, RenderedValueIntent::Converted(_)));
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
}
