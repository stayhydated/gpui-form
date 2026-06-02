use darling::{Error as DarlingError, FromField};
use gpui_form_codegen::components::ShapeOptions;
use proc_macro2::Span;
use syn::{parse::Parser as _, punctuated::Punctuated};

use crate::derives::gpui_form::attrs::{EmptyForm, GpuiFormFieldOption, NoInventory};
use crate::derives::gpui_form::ir::{
    DefaultExpr, FieldAttrContext, FieldContext, RenderedFieldIntent, RenderedValueIntent, Spanned,
};
use crate::derives::gpui_form::validation::KorumaField;

#[derive(Debug)]
pub struct ComponentField {
    pub context: FieldContext,
    attr: FieldAttr,
}

#[derive(Debug)]
struct ComponentFieldOptions {
    component: ShapeOptions,
    rendered: RenderedFieldIntent,
    context: FieldAttrContext,
}

#[derive(Debug)]
enum FieldAttr {
    Component(ComponentFieldOptions),
    Hidden {
        rendered: RenderedFieldIntent,
        context: FieldAttrContext,
    },
    Skipped,
}

impl FieldAttr {
    const fn option_key(&self) -> FieldOptionKey {
        match self {
            Self::Component(_) => FieldOptionKey::Component,
            Self::Hidden { .. } => FieldOptionKey::Hidden,
            Self::Skipped => FieldOptionKey::Skip,
        }
    }
}

#[derive(Debug)]
struct ParsedComponentField {
    context: FieldContext,
    attr: Option<FieldAttr>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FieldOptionKey {
    Component,
    Hidden,
    Skip,
}

impl FieldOptionKey {
    const fn label(self) -> &'static str {
        match self {
            Self::Component => "component",
            Self::Hidden => "hidden",
            Self::Skip => "skip",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FieldOptionConflict {
    Duplicate(FieldOptionKey),
    Intent {
        first: FieldOptionKey,
        second: FieldOptionKey,
    },
    SkipWith(FieldOptionKey),
}

fn field_option_conflict(
    existing: FieldOptionKey,
    incoming: FieldOptionKey,
) -> Option<FieldOptionConflict> {
    if existing == incoming {
        return Some(FieldOptionConflict::Duplicate(incoming));
    }

    match (existing, incoming) {
        (FieldOptionKey::Skip, other) | (other, FieldOptionKey::Skip) => {
            Some(FieldOptionConflict::SkipWith(other))
        },
        (FieldOptionKey::Component, FieldOptionKey::Hidden)
        | (FieldOptionKey::Hidden, FieldOptionKey::Component) => {
            Some(FieldOptionConflict::Intent {
                first: existing,
                second: incoming,
            })
        },
        _ => None,
    }
}

pub struct SkippedField;

pub struct RenderedField<'a> {
    pub value: &'a RenderedValueIntent,
    pub default: Option<&'a Spanned<DefaultExpr>>,
    pub context: &'a FieldAttrContext,
}

pub struct ComponentRenderedField<'a> {
    pub rendered: RenderedField<'a>,
    pub component: &'a ShapeOptions,
}

pub enum ComponentFieldIntent<'a> {
    Skipped(SkippedField),
    Hidden(RenderedField<'a>),
    Component(ComponentRenderedField<'a>),
}

impl ComponentField {
    pub fn intent(&self) -> ComponentFieldIntent<'_> {
        match &self.attr {
            FieldAttr::Component(component) => {
                ComponentFieldIntent::Component(ComponentRenderedField {
                    rendered: RenderedField {
                        value: &component.rendered.value,
                        default: component.rendered.default.as_ref(),
                        context: &component.context,
                    },
                    component: &component.component,
                })
            },
            FieldAttr::Hidden { rendered, context } => {
                ComponentFieldIntent::Hidden(RenderedField {
                    value: &rendered.value,
                    default: rendered.default.as_ref(),
                    context,
                })
            },
            FieldAttr::Skipped => ComponentFieldIntent::Skipped(SkippedField),
        }
    }

    pub fn rendered(&self) -> Option<RenderedField<'_>> {
        match self.intent() {
            ComponentFieldIntent::Skipped(_) => None,
            ComponentFieldIntent::Hidden(rendered) => Some(rendered),
            ComponentFieldIntent::Component(component) => Some(component.rendered),
        }
    }

    pub fn component(&self) -> Option<ComponentRenderedField<'_>> {
        match self.intent() {
            ComponentFieldIntent::Component(component) => Some(component),
            ComponentFieldIntent::Skipped(_) | ComponentFieldIntent::Hidden(_) => None,
        }
    }
}

impl FromField for ComponentField {
    fn from_field(field: &syn::Field) -> darling::Result<Self> {
        let ident = field.ident.clone().ok_or_else(|| {
            DarlingError::custom("GpuiForm only supports named struct fields").with_span(field)
        })?;
        let field_context = FieldContext::new(
            ident.clone(),
            field.ty.clone(),
            syn::spanned::Spanned::span(field),
        );
        let mut parsed = ParsedComponentField {
            context: field_context.clone(),
            attr: None,
        };
        let mut errors = DarlingError::accumulator();
        let mut had_attribute_errors = false;

        for attr in &field.attrs {
            if !attr.path().is_ident("gpui_form") {
                continue;
            }

            let list = match attr.meta.require_list().map_err(|_| {
                DarlingError::custom(
                    "`gpui_form` field attribute must be a list, for example \
                     `#[gpui_form(component(my::Shape))]`",
                )
                .with_span(attr)
            }) {
                Ok(list) => list,
                Err(error) => {
                    errors.push(error);
                    had_attribute_errors = true;
                    continue;
                },
            };

            let items = match Punctuated::<GpuiFormFieldOption, syn::Token![,]>::parse_terminated
                .parse2(list.tokens.clone())
                .map_err(DarlingError::from)
            {
                Ok(items) => items,
                Err(error) => {
                    errors.push(error);
                    had_attribute_errors = true;
                    continue;
                },
            };

            let attr_span = syn::spanned::Spanned::span(attr);
            for item in items {
                if errors
                    .handle(parse_gpui_form_item(&mut parsed, item, attr_span))
                    .is_none()
                {
                    had_attribute_errors = true;
                }
            }
        }

        let attr = if !had_attribute_errors {
            match validate_field_intent(&parsed).map_err(DarlingError::from) {
                Ok(()) => parsed.attr,
                Err(error) => {
                    errors.push(error);
                    None
                },
            }
        } else {
            parsed.attr
        };

        match attr {
            Some(attr) => errors.finish_with(ComponentField {
                context: parsed.context,
                attr,
            }),
            None => errors.finish_with(ComponentField {
                context: parsed.context.clone(),
                attr: FieldAttr::Skipped,
            }),
        }
    }
}

fn parse_gpui_form_item(
    field: &mut ParsedComponentField,
    item: GpuiFormFieldOption,
    attr_span: Span,
) -> darling::Result<()> {
    match item {
        GpuiFormFieldOption::Skip { span } => set_attr(field, FieldAttr::Skipped, span),
        GpuiFormFieldOption::Hidden { span, options } => {
            let context = FieldAttrContext::new(attr_span, span);
            set_attr(
                field,
                FieldAttr::Hidden {
                    rendered: options,
                    context,
                },
                span,
            )
        },
        GpuiFormFieldOption::Component {
            span,
            shape,
            options,
        } => {
            let context = FieldAttrContext::new(attr_span, span);
            let component =
                ShapeOptions::from_constructor_expr_with_span(shape, context.option_span)
                    .map_err(DarlingError::from)?;
            set_attr(
                field,
                FieldAttr::Component(ComponentFieldOptions {
                    component,
                    rendered: options,
                    context,
                }),
                span,
            )
        },
    }
}

fn set_attr(
    field: &mut ParsedComponentField,
    incoming_attr: FieldAttr,
    span: Span,
) -> darling::Result<()> {
    let incoming = incoming_attr.option_key();
    if let Some(existing) = field.attr.as_ref().map(FieldAttr::option_key) {
        if let Some(conflict) = field_option_conflict(existing, incoming) {
            return Err(DarlingError::from(syn::Error::new(
                span,
                field_conflict_message(conflict),
            )));
        }
    }

    field.attr = Some(incoming_attr);
    Ok(())
}

fn field_conflict_message(conflict: FieldOptionConflict) -> String {
    match conflict {
        FieldOptionConflict::Duplicate(key) => {
            let key = key.label();
            format!(
                "duplicate `{key}` option in `gpui_form` field attribute; remove the duplicate `{key}` entry"
            )
        },
        FieldOptionConflict::Intent { first, second } => {
            let first = first.label();
            let second = second.label();
            format!(
                "`{first}` cannot be combined with `{second}` in a `gpui_form` field attribute; \
                 choose exactly one of component, hidden, or skip"
            )
        },
        FieldOptionConflict::SkipWith(option) => {
            let option = option.label();
            format!(
                "`skip` cannot be combined with `{option}` in a `gpui_form` field attribute; \
                 remove `skip` or remove the `{option}` option"
            )
        },
    }
}

fn validate_field_intent(field: &ParsedComponentField) -> darling::Result<()> {
    if field.attr.is_none() {
        return Err(DarlingError::from(syn::Error::new(
            field.context.field_span,
            format!(
                "field `{}` must choose a gpui_form field intent; add `component(...)`, `hidden`, or `skip`",
                field.context.field_ident
            ),
        )));
    };

    Ok(())
}

#[derive(Debug, darling::FromDeriveInput)]
#[darling(attributes(gpui_form), supports(struct_named, struct_unit))]
pub struct ComponentStruct {
    pub data: darling::ast::Data<(), ComponentField>,
    #[darling(default)]
    pub empty: Option<EmptyForm>,
    #[darling(default)]
    pub no_inventory: Option<NoInventory>,
    #[darling(default)]
    pub koruma: Option<KorumaField>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens as _;

    #[test]
    fn parsed_component_field_uses_typed_converted_value_intent() {
        let field: syn::Field = syn::parse_quote! {
            #[gpui_form(component(
                crate::Input,
                value(
                    type = String,
                    from_source = to_form,
                    into_source = to_source,
                )
            ))]
            value: u64
        };

        let parsed = ComponentField::from_field(&field).expect("field should parse");
        let ComponentFieldIntent::Component(component) = parsed.intent() else {
            panic!("expected component field intent");
        };
        let RenderedValueIntent::Converted(converted) = component.rendered.value else {
            panic!("expected converted value intent");
        };

        assert_eq!(
            converted.form_type.value.0.to_token_stream().to_string(),
            "String"
        );
        assert_eq!(
            converted.from_source.value.to_token_stream().to_string(),
            "to_form"
        );
        assert_eq!(
            converted.into_source.value.to_token_stream().to_string(),
            "to_source"
        );
    }

    #[test]
    fn parsed_value_intent_rejects_incomplete_conversion_pair() {
        let field: syn::Field = syn::parse_quote! {
            #[gpui_form(component(
                crate::Input,
                value(type = String, from_source = to_form)
            ))]
            value: u64
        };

        let error = ComponentField::from_field(&field).expect_err("field should fail");

        assert!(
            error
                .to_string()
                .contains("requires an explicit `into_source = ...` conversion"),
            "unexpected error: {error}"
        );
    }
}
