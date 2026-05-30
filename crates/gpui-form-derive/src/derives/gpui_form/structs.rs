use darling::{Error as DarlingError, FromField, FromMeta};
use gpui_form_codegen::components::{RequiredValue, ResolvedComponentShape, ShapeOptions};
use koruma_derive_core::ValidationInfo;
use proc_macro2::{Span, TokenStream};
use syn::{
    Expr, Ident, Path, Token, Type,
    parse::{Parse, ParseStream, Parser as _},
    punctuated::Punctuated,
};

mod kw {
    syn::custom_keyword!(component);
    syn::custom_keyword!(default);
    syn::custom_keyword!(form_to_source);
    syn::custom_keyword!(from_source);
    syn::custom_keyword!(hidden);
    syn::custom_keyword!(into_source);
    syn::custom_keyword!(source_to_form);
    syn::custom_keyword!(shape);
    syn::custom_keyword!(skip);
    syn::custom_keyword!(value);
}

#[derive(Clone, Debug)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new<S: syn::spanned::Spanned>(value: T, span: &S) -> Self {
        Self {
            value,
            span: span.span(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypeOverride(pub Type);

#[derive(Clone, Debug)]
pub struct DefaultExpr(pub Expr);

/// Value-holder storage selected during derive planning.
#[derive(Clone, Debug)]
pub enum StoragePlan {
    OriginallyOptional,
    RequiredValue,
    Direct,
    ShapePolicy { shape: Path },
}

impl StoragePlan {
    pub fn from_required_value(
        _field_name: &Ident,
        was_optional: bool,
        required_value: RequiredValue,
    ) -> syn::Result<Self> {
        if was_optional {
            Ok(Self::OriginallyOptional)
        } else {
            match required_value {
                RequiredValue::Explicit(true) => Ok(Self::RequiredValue),
                RequiredValue::Explicit(false) => Ok(Self::Direct),
                RequiredValue::Shape(shape) => Ok(Self::ShapePolicy { shape }),
            }
        }
    }

    pub fn required_value(&self) -> RequiredValue {
        match self {
            Self::OriginallyOptional | Self::Direct => RequiredValue::explicit(false),
            Self::RequiredValue => RequiredValue::explicit(true),
            Self::ShapePolicy { shape, .. } => RequiredValue::Shape(shape.clone()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ValidationRule {
    Validator(String),
    Required,
    Newtype,
    Nested,
}

impl ValidationRule {
    pub fn is_required(&self) -> bool {
        matches!(self, Self::Required)
    }
}

#[derive(Clone, Debug, Default)]
pub struct ValidationMetadata {
    rules: Vec<ValidationRule>,
}

impl ValidationMetadata {
    pub fn push(&mut self, rule: ValidationRule) {
        self.rules.push(rule);
    }

    pub fn has_required(&self) -> bool {
        self.rules.iter().any(ValidationRule::is_required)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ValidationRule> {
        self.rules.iter()
    }
}

#[derive(Clone, Debug)]
pub struct SharedFieldPlan {
    pub field_name: Ident,
    #[allow(dead_code)]
    pub original_type: Type,
    #[allow(dead_code)]
    pub source_value_type: Type,
    pub form_type: Type,
    pub was_optional: bool,
    pub storage: StoragePlan,
    pub validation: ValidationInfo,
    pub validation_metadata: ValidationMetadata,
    pub default_expr: Option<Expr>,
    pub default_span: Option<Span>,
    pub override_type: Option<Type>,
    pub override_type_span: Option<Span>,
    pub into_expr: Option<Expr>,
    pub into_expr_span: Option<Span>,
    pub from_expr: Option<Expr>,
    pub from_expr_span: Option<Span>,
}

impl SharedFieldPlan {
    pub fn field_name(&self) -> &Ident {
        &self.field_name
    }

    pub fn original_type(&self) -> &Type {
        &self.original_type
    }

    pub fn form_type(&self) -> &Type {
        &self.form_type
    }

    pub fn storage(&self) -> &StoragePlan {
        &self.storage
    }

    pub fn validation(&self) -> &ValidationInfo {
        &self.validation
    }

    pub fn default_expr(&self) -> Option<&Expr> {
        self.default_expr.as_ref()
    }

    pub fn default_expr_span(&self) -> Option<Span> {
        self.default_span
    }

    pub fn override_type(&self) -> Option<&Type> {
        self.override_type.as_ref()
    }

    pub fn override_type_span(&self) -> Option<Span> {
        self.override_type_span
    }

    pub fn source_to_form_expr(&self) -> Option<&Expr> {
        self.from_expr.as_ref()
    }

    pub fn source_to_form_span(&self) -> Option<Span> {
        self.from_expr_span
    }

    pub fn form_to_source_expr(&self) -> Option<&Expr> {
        self.into_expr.as_ref()
    }

    pub fn form_to_source_span(&self) -> Option<Span> {
        self.into_expr_span
    }

    /// Returns true if this rendered field needs the `RequiredValidation`
    /// koruma validator.
    pub fn needs_required_validation(&self) -> bool {
        matches!(
            self.storage,
            StoragePlan::RequiredValue | StoragePlan::ShapePolicy { .. }
        ) && !self.was_optional
            && !self.validation.is_nested
    }
}

#[derive(Clone, Debug)]
pub struct SkippedFieldPlan {
    pub field_name: Ident,
    pub original_type: Type,
}

#[derive(Clone, Debug)]
pub struct HiddenFieldPlan {
    pub shared: SharedFieldPlan,
}

#[derive(Clone, Debug)]
pub struct ComponentFieldPlan {
    pub shared: SharedFieldPlan,
    pub component: ResolvedComponentShape,
    pub component_layout: ComponentFieldContent,
}

/// Precomputed field facts shared by expansion, inventory, and value holders.
#[derive(Clone, Debug)]
pub enum FieldPlan {
    Skipped(SkippedFieldPlan),
    Hidden(HiddenFieldPlan),
    Component(ComponentFieldPlan),
}

impl FieldPlan {
    pub fn skipped(field_name: Ident, original_type: Type) -> Self {
        Self::Skipped(SkippedFieldPlan {
            field_name,
            original_type,
        })
    }

    pub fn hidden(shared: SharedFieldPlan) -> Self {
        Self::Hidden(HiddenFieldPlan { shared })
    }

    pub fn component_field(
        shared: SharedFieldPlan,
        component: ResolvedComponentShape,
        component_layout: ComponentFieldContent,
    ) -> Self {
        Self::Component(ComponentFieldPlan {
            shared,
            component,
            component_layout,
        })
    }

    pub fn shared(&self) -> Option<&SharedFieldPlan> {
        match self {
            Self::Skipped(_) => None,
            Self::Hidden(plan) => Some(&plan.shared),
            Self::Component(plan) => Some(&plan.shared),
        }
    }

    pub fn is_skipped(&self) -> bool {
        matches!(self, Self::Skipped(_))
    }

    pub fn field_name(&self) -> &Ident {
        match self {
            Self::Skipped(plan) => &plan.field_name,
            Self::Hidden(plan) => &plan.shared.field_name,
            Self::Component(plan) => &plan.shared.field_name,
        }
    }

    pub fn original_type(&self) -> &Type {
        match self {
            Self::Skipped(plan) => &plan.original_type,
            Self::Hidden(plan) => &plan.shared.original_type,
            Self::Component(plan) => &plan.shared.original_type,
        }
    }

    pub fn component(&self) -> Option<&ResolvedComponentShape> {
        match self {
            Self::Component(plan) => Some(&plan.component),
            Self::Skipped(_) | Self::Hidden(_) => None,
        }
    }

    pub fn component_plan(&self) -> Option<&ComponentFieldPlan> {
        match self {
            Self::Component(plan) => Some(plan),
            Self::Skipped(_) | Self::Hidden(_) => None,
        }
    }

    /// Returns true if this field needs the `RequiredValidation` koruma validator.
    /// This applies to fields that:
    /// - Can be missing in the form holder
    /// - Must be present in the source struct
    /// - Are not nested structs (nested fields have their own validation)
    pub fn needs_required_validation(&self) -> bool {
        let Some(shared) = self.shared() else {
            return false;
        };

        matches!(
            shared.storage,
            StoragePlan::RequiredValue | StoragePlan::ShapePolicy { .. }
        ) && !shared.was_optional
            && !shared.validation.is_nested
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

#[derive(Clone, Debug)]
pub struct NoInventory;

impl FromMeta for NoInventory {
    fn from_word() -> darling::Result<Self> {
        Ok(Self)
    }
}

#[derive(Debug)]
enum GpuiFormFieldOption {
    Skip {
        span: kw::skip,
    },
    Hidden {
        span: kw::hidden,
        options: RenderedFieldOptions,
    },
    Component {
        span: kw::component,
        shape: Path,
        options: RenderedFieldOptions,
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
                span: key,
                shape,
                options,
            });
        }

        if input.peek(Token![type]) {
            let key = input.parse::<Token![type]>()?;
            return Err(syn::Error::new_spanned(
                key,
                "`type = ...` is no longer a top-level `gpui_form` field option; \
                 move it into `component(..., value(type = ..., from_source = ..., into_source = ...))` \
                 or `hidden(value(type = ..., from_source = ..., into_source = ...))`",
            ));
        }

        if input.peek(kw::form_to_source) {
            let key = input.parse::<kw::form_to_source>()?;
            return Err(syn::Error::new_spanned(
                key,
                "`form_to_source = ...` is no longer a top-level `gpui_form` field option; \
                 use `into_source = ...` inside `value(...)`",
            ));
        }

        if input.peek(kw::source_to_form) {
            let key = input.parse::<kw::source_to_form>()?;
            return Err(syn::Error::new_spanned(
                key,
                "`source_to_form = ...` is no longer a top-level `gpui_form` field option; \
                 use `from_source = ...` inside `value(...)`",
            ));
        }

        if input.peek(kw::default) {
            let key = input.parse::<kw::default>()?;
            return Err(syn::Error::new_spanned(
                key,
                "`default = ...` is no longer a top-level `gpui_form` field option; \
                 move it inside `component(...)` or `hidden(...)`",
            ));
        }

        if input.peek(kw::skip) {
            let key = input.parse::<kw::skip>()?;
            return Ok(Self::Skip { span: key });
        }

        if input.peek(kw::hidden) {
            let key = input.parse::<kw::hidden>()?;
            let options = if input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                parse_structured_field_options(&content, false)?
            } else {
                RenderedFieldOptions::default()
            };
            return Ok(Self::Hidden { span: key, options });
        }

        if input.peek(Ident) {
            let fork = input.fork();
            let key: Ident = fork.parse()?;

            return Err(syn::Error::new_spanned(
                key.clone(),
                format!(
                    "unknown gpui_form field option `{key}`; component-backed fields must use \
                     `component(MyShape)` where `MyShape` is declared with \
                     `gpui_form_derive::component_shape!` or \
                     `#[derive(gpui_form_derive::ComponentShape)]`"
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
) -> syn::Result<RenderedFieldOptions> {
    let mut options = RenderedFieldOptions::default();
    let mut first = true;
    while !input.is_empty() {
        if needs_leading_comma || !first {
            input.parse::<Token![,]>()?;
        }
        first = false;
        if input.peek(kw::value) {
            let _key = input.parse::<kw::value>()?;
            let content;
            syn::parenthesized!(content in input);
            parse_structured_value_options(&content, &mut options)?;
            if content.is_empty() {
                continue;
            }
            return Err(content.error("unexpected tokens in `value(...)`"));
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
    Ok(options)
}

fn parse_structured_value_options(
    input: ParseStream<'_>,
    options: &mut RenderedFieldOptions,
) -> syn::Result<()> {
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
    Ok(())
}

#[derive(Debug)]
pub struct ComponentField {
    pub ident: Ident,
    pub ty: Type,
    attr: FieldAttr,
}

#[derive(Clone, Debug, Default)]
struct RenderedFieldOptions {
    r#type: Option<Spanned<TypeOverride>>,
    form_to_source: Option<Spanned<Expr>>,
    source_to_form: Option<Spanned<Expr>>,
    default: Option<Spanned<DefaultExpr>>,
}

#[derive(Debug)]
struct ComponentFieldOptions {
    component: ShapeOptions,
    rendered: RenderedFieldOptions,
}

#[derive(Debug)]
enum FieldAttr {
    Component(ComponentFieldOptions),
    Hidden(RenderedFieldOptions),
    Skipped,
}

impl FieldAttr {
    const fn option_key(&self) -> FieldOptionKey {
        match self {
            Self::Component(_) => FieldOptionKey::Component,
            Self::Hidden(_) => FieldOptionKey::Hidden,
            Self::Skipped => FieldOptionKey::Skip,
        }
    }
}

#[derive(Debug)]
struct ParsedComponentField {
    ident: Ident,
    ty: Type,
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

pub struct HiddenField;

pub struct RenderedField<'a> {
    pub r#type: Option<&'a Spanned<TypeOverride>>,
    pub form_to_source: Option<&'a Spanned<Expr>>,
    pub source_to_form: Option<&'a Spanned<Expr>>,
    pub default: Option<&'a Spanned<DefaultExpr>>,
}

pub struct ComponentRenderedField<'a> {
    pub r#type: Option<&'a Spanned<TypeOverride>>,
    pub form_to_source: Option<&'a Spanned<Expr>>,
    pub source_to_form: Option<&'a Spanned<Expr>>,
    pub component: &'a ShapeOptions,
    pub default: Option<&'a Spanned<DefaultExpr>>,
}

pub enum ComponentFieldIntent<'a> {
    Skipped(SkippedField),
    Hidden(HiddenField, RenderedField<'a>),
    Component(ComponentRenderedField<'a>),
}

impl ComponentField {
    pub fn intent(&self) -> ComponentFieldIntent<'_> {
        match &self.attr {
            FieldAttr::Component(component) => {
                ComponentFieldIntent::Component(ComponentRenderedField {
                    r#type: component.rendered.r#type.as_ref(),
                    form_to_source: component.rendered.form_to_source.as_ref(),
                    source_to_form: component.rendered.source_to_form.as_ref(),
                    component: &component.component,
                    default: component.rendered.default.as_ref(),
                })
            },
            FieldAttr::Hidden(rendered) => ComponentFieldIntent::Hidden(
                HiddenField,
                RenderedField {
                    r#type: rendered.r#type.as_ref(),
                    form_to_source: rendered.form_to_source.as_ref(),
                    source_to_form: rendered.source_to_form.as_ref(),
                    default: rendered.default.as_ref(),
                },
            ),
            FieldAttr::Skipped => ComponentFieldIntent::Skipped(SkippedField),
        }
    }

    pub fn rendered(&self) -> Option<RenderedField<'_>> {
        match self.intent() {
            ComponentFieldIntent::Skipped(_) => None,
            ComponentFieldIntent::Hidden(_, rendered) => Some(rendered),
            ComponentFieldIntent::Component(component) => Some(RenderedField {
                r#type: component.r#type,
                form_to_source: component.form_to_source,
                source_to_form: component.source_to_form,
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
}

impl FromField for ComponentField {
    fn from_field(field: &syn::Field) -> darling::Result<Self> {
        let ident = field.ident.clone().ok_or_else(|| {
            DarlingError::custom("GpuiForm only supports named struct fields").with_span(field)
        })?;
        let mut parsed = ParsedComponentField {
            ident,
            ty: field.ty.clone(),
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

            for item in items {
                if errors
                    .handle(parse_gpui_form_item(&mut parsed, item))
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
                ident: parsed.ident,
                ty: parsed.ty,
                attr,
            }),
            None => errors.finish_with(ComponentField {
                ident: parsed.ident,
                ty: parsed.ty,
                attr: FieldAttr::Skipped,
            }),
        }
    }
}

fn parse_gpui_form_item(
    field: &mut ParsedComponentField,
    item: GpuiFormFieldOption,
) -> darling::Result<()> {
    match item {
        GpuiFormFieldOption::Skip { span } => set_attr(field, FieldAttr::Skipped, &span),
        GpuiFormFieldOption::Hidden { span, options } => {
            set_attr(field, FieldAttr::Hidden(options), &span)
        },
        GpuiFormFieldOption::Component {
            span,
            shape,
            options,
        } => {
            let component = ShapeOptions::from_shape_with_span(shape, span.span);
            set_attr(
                field,
                FieldAttr::Component(ComponentFieldOptions {
                    component,
                    rendered: options,
                }),
                &span,
            )
        },
    }
}

fn set_attr<S: syn::spanned::Spanned>(
    field: &mut ParsedComponentField,
    incoming_attr: FieldAttr,
    span: &S,
) -> darling::Result<()> {
    let incoming = incoming_attr.option_key();
    if let Some(existing) = field.attr.as_ref().map(FieldAttr::option_key) {
        if let Some(conflict) = field_option_conflict(existing, incoming) {
            return Err(field_conflict_error(conflict).with_span(span));
        }
    }

    field.attr = Some(incoming_attr);
    Ok(())
}

fn field_conflict_error(conflict: FieldOptionConflict) -> DarlingError {
    match conflict {
        FieldOptionConflict::Duplicate(key) => {
            let key = key.label();
            DarlingError::custom(format!(
                "duplicate `{key}` option in `gpui_form` field attribute; remove the duplicate `{key}` entry"
            ))
        },
        FieldOptionConflict::Intent { first, second } => {
            let first = first.label();
            let second = second.label();
            DarlingError::custom(format!(
                "`{first}` cannot be combined with `{second}` in a `gpui_form` field attribute; \
                 choose exactly one of component, hidden, or skip"
            ))
        },
        FieldOptionConflict::SkipWith(option) => {
            let option = option.label();
            DarlingError::custom(format!(
                "`skip` cannot be combined with `{option}` in a `gpui_form` field attribute; \
                 remove `skip` or remove the `{option}` option"
            ))
        },
    }
}

fn validate_field_intent(field: &ParsedComponentField) -> darling::Result<()> {
    let Some(attr) = &field.attr else {
        return Err(DarlingError::custom(format!(
            "field `{}` must choose a gpui_form field intent; add `component(...)`, `hidden`, or `skip`",
            field.ident
        ))
        .with_span(&field.ident));
    };

    let rendered = match attr {
        FieldAttr::Component(component) => Some(&component.rendered),
        FieldAttr::Hidden(rendered) => Some(rendered),
        FieldAttr::Skipped => None,
    };

    if let Some(rendered) = rendered {
        if let Some(type_override) = &rendered.r#type {
            if rendered.source_to_form.is_none() {
                return Err(DarlingError::custom(
                    "`value(type = ...)` requires an explicit `from_source = ...` conversion",
                )
                .with_span(&type_override.span));
            }

            if rendered.form_to_source.is_none() {
                return Err(DarlingError::custom(
                    "`value(type = ...)` requires an explicit `into_source = ...` conversion",
                )
                .with_span(&type_override.span));
            }
        } else if let Some(from_source) = &rendered.source_to_form {
            return Err(DarlingError::custom(
                "`from_source = ...` requires `type = ...` in the same `value(...)` option",
            )
            .with_span(&from_source.span));
        } else if let Some(into_source) = &rendered.form_to_source {
            return Err(DarlingError::custom(
                "`into_source = ...` requires `type = ...` in the same `value(...)` option",
            )
            .with_span(&into_source.span));
        }
    }

    Ok(())
}

#[derive(Debug, darling::FromDeriveInput)]
#[darling(attributes(gpui_form), supports(struct_named, struct_unit))]
pub struct ComponentStruct {
    pub ident: Ident,
    pub data: darling::ast::Data<(), ComponentField>,
    #[darling(default)]
    pub empty: Option<EmptyForm>,
    #[darling(default)]
    pub no_inventory: Option<NoInventory>,
    #[darling(default)]
    pub koruma: Option<KorumaField>,
}

#[derive(Clone, Debug)]
pub struct ComponentFieldContent {
    pub component: ResolvedComponentShape,
    pub field_structure_tokens: TokenStream,
    pub field_base_declarations_tokens: TokenStream,
    pub required_value: RequiredValue,
}

pub struct GpuiFormOptions {
    pub generate_shape: bool,
}
