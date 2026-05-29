use darling::{Error as DarlingError, FromField, FromMeta};
use gpui_form_codegen::components::{RequiredValue, ShapeOptions};
use koruma_derive_core::ValidationInfo;
use proc_macro2::TokenStream;
use syn::{
    Expr, ExprGroup, ExprParen, ExprPath, Ident, Token, Type,
    parse::{Parse, ParseStream, Parser as _},
    punctuated::Punctuated,
};

mod kw {
    syn::custom_keyword!(default);
    syn::custom_keyword!(from);
    syn::custom_keyword!(into);
    syn::custom_keyword!(skip);
}

#[derive(Clone, Debug)]
pub struct TypeOverride(pub Type);

#[derive(Clone, Debug)]
pub struct DefaultExpr(pub Expr);

/// Information about a field for value holder generation.
pub struct FieldOptionality {
    pub field_name: Ident,
    #[allow(dead_code)]
    pub original_type: Type,
    #[allow(dead_code)]
    pub inner_type: Type,
    pub was_optional: bool,
    pub required_value: RequiredValue,
    pub validation: ValidationInfo,
    pub default_expr: Option<Expr>,
    pub override_type: Option<Type>,
    pub into_expr: Option<Expr>,
    pub from_expr: Option<Expr>,
    pub skip: bool,
}

impl FieldOptionality {
    /// Returns true if this field needs the `RequiredValidation` koruma validator.
    /// This applies to fields that:
    /// - Can be missing in the form holder
    /// - Must be present in the source struct
    /// - Are not nested structs (nested fields have their own validation)
    pub fn needs_required_validation(&self) -> bool {
        !self.skip
            && matches!(
                self.required_value,
                RequiredValue::Explicit(true) | RequiredValue::Shape(_)
            )
            && !self.was_optional
            && !self.validation.is_nested
    }
}

#[derive(Clone, Debug)]
pub struct KorumaOptions {
    pub fluent: bool,
}

#[derive(Clone, Debug)]
pub struct KorumaField(pub KorumaOptions);

impl FromMeta for KorumaField {
    fn from_word() -> darling::Result<Self> {
        Ok(KorumaField(KorumaOptions { fluent: false }))
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let [darling::ast::NestedMeta::Meta(syn::Meta::Path(path))] = items else {
            return Err(DarlingError::custom(
                "`koruma(...)` only accepts `fluent`, as in `#[gpui_form(koruma(fluent))]`",
            ));
        };

        if path.is_ident("fluent") {
            Ok(KorumaField(KorumaOptions { fluent: true }))
        } else {
            Err(DarlingError::custom(
                "`koruma(...)` only accepts `fluent`, as in `#[gpui_form(koruma(fluent))]`",
            )
            .with_span(path))
        }
    }
}

#[derive(Clone, Debug)]
pub struct EmptyForm;

impl FromMeta for EmptyForm {
    fn from_word() -> darling::Result<Self> {
        Ok(Self)
    }
}

#[derive(Debug)]
enum GpuiFormFieldOption {
    Skip { span: kw::skip },
    Type { span: Token![type], ty: Type },
    Into { span: kw::into, expr: Expr },
    From { span: kw::from, expr: Expr },
    Default { span: kw::default, expr: Expr },
    Component(Expr),
}

impl Parse for GpuiFormFieldOption {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(Token![type]) {
            let key = input.parse::<Token![type]>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::Type {
                span: key,
                ty: input.parse()?,
            });
        }

        if input.peek(kw::into) {
            let key = input.parse::<kw::into>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::Into {
                span: key,
                expr: input.parse()?,
            });
        }

        if input.peek(kw::from) {
            let key = input.parse::<kw::from>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::From {
                span: key,
                expr: input.parse()?,
            });
        }

        if input.peek(kw::default) {
            let key = input.parse::<kw::default>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::Default {
                span: key,
                expr: input.parse()?,
            });
        }

        if input.peek(kw::skip) {
            let key = input.parse::<kw::skip>()?;
            return Ok(Self::Skip { span: key });
        }

        if input.peek(Ident) {
            let fork = input.fork();
            let key: Ident = fork.parse()?;

            if fork.peek(Token![=]) || fork.peek(syn::token::Paren) {
                return Err(syn::Error::new_spanned(
                    key.clone(),
                    format!("unknown gpui_form field option `{key}`"),
                ));
            }
        }

        Ok(Self::Component(input.parse()?))
    }
}

#[derive(Debug)]
pub struct ComponentField {
    pub ident: Option<Ident>,
    pub ty: Type,
    intent: ParsedFieldIntent,
}

#[derive(Debug, Default)]
struct RenderedOptions {
    r#type: Option<TypeOverride>,
    into: Option<Expr>,
    from: Option<Expr>,
    component: Option<ShapeOptions>,
    default: Option<DefaultExpr>,
}

impl RenderedOptions {
    fn first_conflict(&self) -> Option<&'static str> {
        [
            ("component", self.component.is_some()),
            ("default", self.default.is_some()),
            ("type", self.r#type.is_some()),
            ("from", self.from.is_some()),
            ("into", self.into.is_some()),
        ]
        .into_iter()
        .find_map(|(option, present)| present.then_some(option))
    }
}

#[derive(Debug)]
struct ParsedFieldIntent {
    rendered: Option<Box<RenderedOptions>>,
}

impl ParsedFieldIntent {
    fn rendered() -> Self {
        Self {
            rendered: Some(Box::default()),
        }
    }

    fn is_skipped(&self) -> bool {
        self.rendered.is_none()
    }

    fn rendered_options(&self) -> Option<&RenderedOptions> {
        self.rendered.as_deref()
    }

    fn rendered_options_mut(&mut self) -> Option<&mut RenderedOptions> {
        self.rendered.as_deref_mut()
    }

    fn skip(&mut self) {
        self.rendered = None;
    }
}

pub struct SkippedField;

pub struct RenderedField<'a> {
    pub r#type: Option<&'a TypeOverride>,
    pub into: Option<&'a Expr>,
    pub from: Option<&'a Expr>,
    pub component: Option<&'a ShapeOptions>,
    pub default: Option<&'a DefaultExpr>,
}

pub enum ComponentFieldIntent<'a> {
    Skipped(SkippedField),
    Rendered(RenderedField<'a>),
}

impl ComponentField {
    pub fn intent(&self) -> ComponentFieldIntent<'_> {
        match self.intent.rendered_options() {
            Some(rendered) => ComponentFieldIntent::Rendered(RenderedField {
                r#type: rendered.r#type.as_ref(),
                into: rendered.into.as_ref(),
                from: rendered.from.as_ref(),
                component: rendered.component.as_ref(),
                default: rendered.default.as_ref(),
            }),
            None => ComponentFieldIntent::Skipped(SkippedField),
        }
    }

    pub fn rendered(&self) -> Option<RenderedField<'_>> {
        match self.intent() {
            ComponentFieldIntent::Skipped(_) => None,
            ComponentFieldIntent::Rendered(rendered) => Some(rendered),
        }
    }

    pub fn skip(&self) -> bool {
        matches!(self.intent(), ComponentFieldIntent::Skipped(_))
    }
}

impl FromField for ComponentField {
    fn from_field(field: &syn::Field) -> darling::Result<Self> {
        let mut parsed = ComponentField {
            ident: field.ident.clone(),
            ty: field.ty.clone(),
            intent: ParsedFieldIntent::rendered(),
        };

        for attr in &field.attrs {
            if !attr.path().is_ident("gpui_form") {
                continue;
            }

            let list = attr.meta.require_list().map_err(|_| {
                DarlingError::custom(
                    "`gpui_form` field attribute must be a list, for example \
                     `#[gpui_form(my::Shape)]`",
                )
                .with_span(attr)
            })?;

            let items = Punctuated::<GpuiFormFieldOption, syn::Token![,]>::parse_terminated
                .parse2(list.tokens.clone())
                .map_err(DarlingError::from)?;

            for item in items {
                parse_gpui_form_item(&mut parsed, item)?;
            }
        }

        Ok(parsed)
    }
}

fn parse_gpui_form_item(
    field: &mut ComponentField,
    item: GpuiFormFieldOption,
) -> darling::Result<()> {
    match item {
        GpuiFormFieldOption::Skip { span } => set_skip(field, true, &span),
        GpuiFormFieldOption::Type { span, ty } => {
            let options = rendered_options_mut(field, "type", &span)?;
            set_once(&mut options.r#type, TypeOverride(ty), "type", &span)
        },
        GpuiFormFieldOption::Into { span, expr } => {
            let options = rendered_options_mut(field, "into", &span)?;
            set_once(&mut options.into, expr, "into", &span)
        },
        GpuiFormFieldOption::From { span, expr } => {
            let options = rendered_options_mut(field, "from", &span)?;
            set_once(&mut options.from, expr, "from", &span)
        },
        GpuiFormFieldOption::Default { span, expr } => {
            let options = rendered_options_mut(field, "default", &span)?;
            set_once(&mut options.default, DefaultExpr(expr), "default", &span)
        },
        GpuiFormFieldOption::Component(expr) => parse_gpui_form_expression(field, expr),
    }
}

fn parse_gpui_form_expression(field: &mut ComponentField, expr: Expr) -> darling::Result<()> {
    let expr = unwrap_grouped_expr(expr);

    match expr {
        Expr::Path(path) => parse_path_keyword(field, path),
        _ => {
            let component = ShapeOptions::from_expr(&expr)?;
            set_component(field, component, &expr)
        },
    }
}

fn parse_path_keyword(field: &mut ComponentField, path: ExprPath) -> darling::Result<()> {
    if let Some(ident) = single_segment_path_ident(&path) {
        if ident == "skip" {
            set_skip(field, true, &path)?;
            return Ok(());
        }

        if !is_upper_camel_ident(ident) {
            return Err(DarlingError::custom(format!(
                "unknown gpui_form field option `{ident}`; bare field options only support `skip`; \
                 use an UpperCamel shape identifier such as `MyShape` or a qualified shape path \
                 such as `my::Shape`"
            ))
            .with_span(ident));
        }
    }

    parse_positional_component(field, Expr::Path(path))
}

fn single_segment_path_ident(path: &ExprPath) -> Option<&Ident> {
    if path.qself.is_none() && path.path.segments.len() == 1 {
        path.path.segments.last().map(|segment| &segment.ident)
    } else {
        None
    }
}

fn is_upper_camel_ident(ident: &Ident) -> bool {
    let ident = ident.to_string();
    let ident = ident.strip_prefix("r#").unwrap_or(&ident);
    ident
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
}

fn parse_positional_component(field: &mut ComponentField, expr: Expr) -> darling::Result<()> {
    let component = ShapeOptions::from_expr(&expr)?;
    set_component(field, component, &expr)
}

fn set_component<T: syn::spanned::Spanned>(
    field: &mut ComponentField,
    component: ShapeOptions,
    span: &T,
) -> darling::Result<()> {
    let options = rendered_options_mut(field, "component", span)?;

    if options.component.is_some() {
        return Err(DarlingError::custom(
            "multiple component expressions were provided; keep a single shape \
             expression, such as `#[gpui_form(my::Shape)]`",
        )
        .with_span(span));
    }

    options.component = Some(component);
    Ok(())
}

fn set_once<T, S: syn::spanned::Spanned>(
    slot: &mut Option<T>,
    value: T,
    key: &str,
    span: &S,
) -> darling::Result<()> {
    if slot.is_some() {
        return Err(DarlingError::custom(format!(
            "duplicate `{key}` option in `gpui_form` field attribute; remove the duplicate `{key}` entry"
        ))
        .with_span(span));
    }

    *slot = Some(value);
    Ok(())
}

fn set_skip<S: syn::spanned::Spanned>(
    field: &mut ComponentField,
    value: bool,
    span: &S,
) -> darling::Result<()> {
    if !value {
        return Ok(());
    }

    if field.intent.is_skipped() {
        return Err(DarlingError::custom(
            "duplicate `skip` option in `gpui_form` field attribute; remove the duplicate `skip` entry",
        )
        .with_span(span));
    }

    let options = field
        .intent
        .rendered_options()
        .expect("non-skipped field intent should have rendered options");
    if let Some(option) = options.first_conflict() {
        return Err(skip_conflict_error(option).with_span(span));
    }

    field.intent.skip();
    Ok(())
}

fn skip_conflict_error(option: &str) -> DarlingError {
    DarlingError::custom(format!(
        "`skip` cannot be combined with `{option}` in a `gpui_form` field attribute; \
         remove `skip` or remove the `{option}` option"
    ))
}

fn rendered_options_mut<'a, S: syn::spanned::Spanned>(
    field: &'a mut ComponentField,
    option: &str,
    span: &S,
) -> darling::Result<&'a mut RenderedOptions> {
    field
        .intent
        .rendered_options_mut()
        .ok_or_else(|| skip_conflict_error(option).with_span(span))
}

fn unwrap_grouped_expr(expr: Expr) -> Expr {
    let mut expr = expr;
    loop {
        match expr {
            Expr::Group(ExprGroup { expr: inner, .. }) => expr = *inner,
            Expr::Paren(ExprParen { expr: inner, .. }) => expr = *inner,
            _ => return expr,
        }
    }
}

#[derive(Debug, darling::FromDeriveInput)]
#[darling(attributes(gpui_form), supports(struct_named, struct_unit))]
pub struct ComponentStruct {
    pub ident: Ident,
    pub data: darling::ast::Data<(), ComponentField>,
    #[darling(default)]
    pub empty: Option<EmptyForm>,
    #[darling(default)]
    pub koruma: Option<KorumaField>,
}

pub struct ComponentFieldContent {
    pub field_structure_tokens: TokenStream,
    pub field_base_declarations_tokens: TokenStream,
    pub required_value: (String, RequiredValue),
}

pub struct GpuiFormOptions {
    pub generate_shape: bool,
}
