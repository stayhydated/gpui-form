use darling::{Error as DarlingError, FromField, FromMeta};
use gpui_form_codegen::components::{RequiredValue, ShapeMetadataCall, ShapeOptions};
use koruma_derive_core::ValidationInfo;
use proc_macro2::TokenStream;
use syn::{
    Expr, Ident, Path, Token, Type, parenthesized,
    parse::{Parse, ParseStream, Parser as _},
    punctuated::Punctuated,
};

mod kw {
    syn::custom_keyword!(default);
    syn::custom_keyword!(from);
    syn::custom_keyword!(hidden);
    syn::custom_keyword!(into);
    syn::custom_keyword!(skip);
}

#[derive(Clone, Debug)]
pub struct TypeOverride(pub Type);

#[derive(Clone, Debug)]
pub struct DefaultExpr(pub Expr);

/// Precomputed field facts shared by expansion, inventory, and value holders.
pub struct AnalyzedField {
    pub field_name: Ident,
    #[allow(dead_code)]
    pub original_type: Type,
    #[allow(dead_code)]
    pub source_value_type: Type,
    pub form_type: Type,
    pub was_optional: bool,
    pub required_value: RequiredValue,
    pub validation: ValidationInfo,
    pub default_expr: Option<Expr>,
    pub override_type: Option<Type>,
    pub into_expr: Option<Expr>,
    pub from_expr: Option<Expr>,
    pub component: Option<ShapeOptions>,
    pub skip: bool,
}

impl AnalyzedField {
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
    Hidden { span: kw::hidden },
    Type { span: Token![type], ty: Type },
    Into { span: kw::into, expr: Expr },
    From { span: kw::from, expr: Expr },
    Default { span: kw::default, expr: Expr },
    Component(FieldShapeExpr),
}

#[derive(Debug)]
struct FieldShapeExpr {
    path: Path,
    metadata: Vec<ShapeMetadataCall>,
}

impl Parse for FieldShapeExpr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut shape = if input.peek(syn::token::Paren) {
            let content;
            parenthesized!(content in input);
            let shape = content.parse::<Self>()?;
            if !content.is_empty() {
                return Err(content
                    .error("component syntax expects a shape path or shape metadata expression"));
            }
            shape
        } else {
            Self {
                path: input.parse()?,
                metadata: Vec::new(),
            }
        };

        while input.peek(Token![.]) {
            input.parse::<Token![.]>()?;
            let method: Ident = input.parse()?;
            let content;
            parenthesized!(content in input);
            let args = Punctuated::<Expr, Token![,]>::parse_terminated(&content)?
                .into_iter()
                .collect();
            shape.metadata.push(ShapeMetadataCall {
                method: gpui_form_codegen::components::ShapeMetadataMethod::parse_syn(&method)?,
                span: method,
                args,
            });
        }

        if input.peek(syn::token::Paren) {
            return Err(input.error(
                "component metadata must use method-call syntax, such as `Shape.field_suffix(\"input\")`",
            ));
        }

        Ok(shape)
    }
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

        if input.peek(kw::hidden) {
            let key = input.parse::<kw::hidden>()?;
            return Ok(Self::Hidden { span: key });
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
    pub ident: Ident,
    pub ty: Type,
    intent: ParsedFieldIntent,
}

#[derive(Debug, Default)]
struct FieldOptions {
    r#type: Option<TypeOverride>,
    into: Option<Expr>,
    from: Option<Expr>,
    default: Option<DefaultExpr>,
}

impl FieldOptions {
    fn first_conflict(&self) -> Option<&'static str> {
        [
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
    kind: Option<FieldIntentKind>,
    options: Box<FieldOptions>,
}

#[derive(Debug)]
enum FieldIntentKind {
    Component(ShapeOptions),
    Hidden,
    Skipped,
}

impl ParsedFieldIntent {
    fn pending() -> Self {
        Self {
            kind: None,
            options: Box::default(),
        }
    }

    fn is_skipped(&self) -> bool {
        matches!(self.kind, Some(FieldIntentKind::Skipped))
    }

    fn options_mut(&mut self) -> Option<&mut FieldOptions> {
        match self.kind {
            Some(FieldIntentKind::Skipped) => None,
            _ => Some(&mut self.options),
        }
    }
}

pub struct SkippedField;

pub struct HiddenField;

pub struct RenderedField<'a> {
    pub r#type: Option<&'a TypeOverride>,
    pub into: Option<&'a Expr>,
    pub from: Option<&'a Expr>,
    pub default: Option<&'a DefaultExpr>,
}

pub struct ComponentRenderedField<'a> {
    pub r#type: Option<&'a TypeOverride>,
    pub into: Option<&'a Expr>,
    pub from: Option<&'a Expr>,
    pub component: &'a ShapeOptions,
    pub default: Option<&'a DefaultExpr>,
}

pub enum ComponentFieldIntent<'a> {
    Skipped(SkippedField),
    Hidden(HiddenField, RenderedField<'a>),
    Component(ComponentRenderedField<'a>),
}

impl ComponentField {
    pub fn intent(&self) -> ComponentFieldIntent<'_> {
        match &self.intent.kind {
            Some(FieldIntentKind::Component(component)) => {
                ComponentFieldIntent::Component(ComponentRenderedField {
                    r#type: self.intent.options.r#type.as_ref(),
                    into: self.intent.options.into.as_ref(),
                    from: self.intent.options.from.as_ref(),
                    component,
                    default: self.intent.options.default.as_ref(),
                })
            },
            Some(FieldIntentKind::Hidden) => ComponentFieldIntent::Hidden(
                HiddenField,
                RenderedField {
                    r#type: self.intent.options.r#type.as_ref(),
                    into: self.intent.options.into.as_ref(),
                    from: self.intent.options.from.as_ref(),
                    default: self.intent.options.default.as_ref(),
                },
            ),
            Some(FieldIntentKind::Skipped) | None => ComponentFieldIntent::Skipped(SkippedField),
        }
    }

    pub fn rendered(&self) -> Option<RenderedField<'_>> {
        match self.intent() {
            ComponentFieldIntent::Skipped(_) => None,
            ComponentFieldIntent::Hidden(_, rendered) => Some(rendered),
            ComponentFieldIntent::Component(component) => Some(RenderedField {
                r#type: component.r#type,
                into: component.into,
                from: component.from,
                default: component.default,
            }),
        }
    }

    pub fn component(&self) -> Option<ComponentRenderedField<'_>> {
        match self.intent() {
            ComponentFieldIntent::Component(component) => Some(component),
            ComponentFieldIntent::Skipped(_) | ComponentFieldIntent::Hidden(_, _) => None,
        }
    }

    pub fn skip(&self) -> bool {
        matches!(self.intent(), ComponentFieldIntent::Skipped(_))
    }
}

impl FromField for ComponentField {
    fn from_field(field: &syn::Field) -> darling::Result<Self> {
        let ident = field.ident.clone().ok_or_else(|| {
            DarlingError::custom("GpuiForm only supports named struct fields").with_span(field)
        })?;
        let mut parsed = ComponentField {
            ident,
            ty: field.ty.clone(),
            intent: ParsedFieldIntent::pending(),
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

        validate_field_intent(&parsed)?;

        Ok(parsed)
    }
}

fn parse_gpui_form_item(
    field: &mut ComponentField,
    item: GpuiFormFieldOption,
) -> darling::Result<()> {
    match item {
        GpuiFormFieldOption::Skip { span } => set_skip(field, true, &span),
        GpuiFormFieldOption::Hidden { span } => set_hidden(field, &span),
        GpuiFormFieldOption::Type { span, ty } => {
            let options = field_options_mut(field, "type", &span)?;
            set_once(&mut options.r#type, TypeOverride(ty), "type", &span)
        },
        GpuiFormFieldOption::Into { span, expr } => {
            let options = field_options_mut(field, "into", &span)?;
            set_once(&mut options.into, expr, "into", &span)
        },
        GpuiFormFieldOption::From { span, expr } => {
            let options = field_options_mut(field, "from", &span)?;
            set_once(&mut options.from, expr, "from", &span)
        },
        GpuiFormFieldOption::Default { span, expr } => {
            let options = field_options_mut(field, "default", &span)?;
            set_once(&mut options.default, DefaultExpr(expr), "default", &span)
        },
        GpuiFormFieldOption::Component(shape) => parse_gpui_form_shape(field, shape),
    }
}

fn parse_gpui_form_shape(field: &mut ComponentField, shape: FieldShapeExpr) -> darling::Result<()> {
    if let Some(ident) = single_segment_path_ident(&shape.path) {
        if ident == "skip" {
            set_skip(field, true, &shape.path)?;
            return Ok(());
        }

        if !is_upper_camel_ident(ident) {
            return Err(DarlingError::custom(format!(
                "unknown gpui_form field option `{ident}`; bare field options only support `hidden` and `skip`; \
                 use an UpperCamel shape identifier such as `MyShape` or a qualified shape path \
                 such as `my::Shape`"
            ))
            .with_span(ident));
        }
    }

    parse_positional_component(field, shape)
}

fn single_segment_path_ident(path: &Path) -> Option<&Ident> {
    if path.segments.len() == 1 {
        path.segments.last().map(|segment| &segment.ident)
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

fn parse_positional_component(
    field: &mut ComponentField,
    shape: FieldShapeExpr,
) -> darling::Result<()> {
    let span = shape.path.clone();
    let component = ShapeOptions::from_shape_and_metadata(shape.path, shape.metadata)?;
    set_component(field, component, &span)
}

fn set_component<T: syn::spanned::Spanned>(
    field: &mut ComponentField,
    component: ShapeOptions,
    span: &T,
) -> darling::Result<()> {
    match field.intent.kind {
        Some(FieldIntentKind::Component(_)) => {
            return Err(DarlingError::custom(
                "multiple component expressions were provided; keep a single shape \
                 expression, such as `#[gpui_form(my::Shape)]`",
            )
            .with_span(span));
        },
        Some(FieldIntentKind::Hidden) => {
            return Err(intent_conflict_error("hidden", "component").with_span(span));
        },
        Some(FieldIntentKind::Skipped) => {
            return Err(skip_conflict_error("component").with_span(span));
        },
        None => {},
    }

    field.intent.kind = Some(FieldIntentKind::Component(component));
    Ok(())
}

fn set_hidden<S: syn::spanned::Spanned>(
    field: &mut ComponentField,
    span: &S,
) -> darling::Result<()> {
    match field.intent.kind {
        Some(FieldIntentKind::Hidden) => {
            return Err(DarlingError::custom(
                "duplicate `hidden` option in `gpui_form` field attribute; remove the duplicate `hidden` entry",
            )
            .with_span(span));
        },
        Some(FieldIntentKind::Component(_)) => {
            return Err(intent_conflict_error("component", "hidden").with_span(span));
        },
        Some(FieldIntentKind::Skipped) => {
            return Err(skip_conflict_error("hidden").with_span(span));
        },
        None => {},
    }

    field.intent.kind = Some(FieldIntentKind::Hidden);
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

    if matches!(field.intent.kind, Some(FieldIntentKind::Component(_))) {
        return Err(skip_conflict_error("component").with_span(span));
    }

    if matches!(field.intent.kind, Some(FieldIntentKind::Hidden)) {
        return Err(skip_conflict_error("hidden").with_span(span));
    }

    if let Some(option) = field.intent.options.first_conflict() {
        return Err(skip_conflict_error(option).with_span(span));
    }

    field.intent.kind = Some(FieldIntentKind::Skipped);
    Ok(())
}

fn intent_conflict_error(first: &str, second: &str) -> DarlingError {
    DarlingError::custom(format!(
        "`{first}` cannot be combined with `{second}` in a `gpui_form` field attribute; \
         choose exactly one of component, hidden, or skip"
    ))
}

fn skip_conflict_error(option: &str) -> DarlingError {
    DarlingError::custom(format!(
        "`skip` cannot be combined with `{option}` in a `gpui_form` field attribute; \
         remove `skip` or remove the `{option}` option"
    ))
}

fn field_options_mut<'a, S: syn::spanned::Spanned>(
    field: &'a mut ComponentField,
    option: &str,
    span: &S,
) -> darling::Result<&'a mut FieldOptions> {
    field
        .intent
        .options_mut()
        .ok_or_else(|| skip_conflict_error(option).with_span(span))
}

fn validate_field_intent(field: &ComponentField) -> darling::Result<()> {
    if field.intent.kind.is_some() {
        return Ok(());
    }

    Err(DarlingError::custom(format!(
        "field `{}` must choose a gpui_form field intent; add a component shape, `hidden`, or `skip`",
        field.ident
    ))
    .with_span(&field.ident))
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
