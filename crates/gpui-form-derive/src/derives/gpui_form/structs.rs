use darling::{Error as DarlingError, FromField, FromMeta};
use gpui_form_codegen::components::{RequiredValue, ShapeOptions};
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
    syn::custom_keyword!(hidden);
    syn::custom_keyword!(source_to_form);
    syn::custom_keyword!(shape);
    syn::custom_keyword!(skip);
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
    ShapePolicy { shape: Path, policy_ident: Ident },
}

impl StoragePlan {
    pub fn from_required_value(
        field_name: &Ident,
        was_optional: bool,
        required_value: RequiredValue,
    ) -> syn::Result<Self> {
        if was_optional {
            Ok(Self::OriginallyOptional)
        } else {
            match required_value {
                RequiredValue::Explicit(true) => Ok(Self::RequiredValue),
                RequiredValue::Explicit(false) => Ok(Self::Direct),
                RequiredValue::Shape(shape) => Ok(Self::ShapePolicy {
                    shape,
                    policy_ident: value_storage_policy_ident_for_field(field_name)?,
                }),
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

    pub fn shape_policy_ident(&self) -> Option<&Ident> {
        match self {
            Self::ShapePolicy { policy_ident, .. } => Some(policy_ident),
            _ => None,
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

    pub fn shape_policy_ident(&self) -> Option<&Ident> {
        self.storage.shape_policy_ident()
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
    pub component: ShapeOptions,
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
        component: ShapeOptions,
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

    pub fn component(&self) -> Option<&ShapeOptions> {
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

pub fn value_storage_policy_ident_for_field(field_name: &Ident) -> syn::Result<Ident> {
    let field_name_ident = field_name;
    let field_name = field_name_ident.to_string();
    let field_name = field_name.strip_prefix("r#").unwrap_or(&field_name);
    let mut pascal = String::new();
    let mut uppercase_next = true;
    for ch in field_name.chars() {
        if ch == '_' {
            uppercase_next = true;
        } else if uppercase_next {
            pascal.extend(ch.to_uppercase());
            uppercase_next = false;
        } else {
            pascal.push(ch);
        }
    }

    let ident = format!("__GpuiForm{pascal}ValueStoragePolicy");
    syn::parse_str::<Ident>(&ident).map_err(|err| {
        syn::Error::new_spanned(
            field_name_ident,
            format!("failed to generate value-storage policy identifier `{ident}`: {err}"),
        )
    })
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
    },
    Type {
        span: Token![type],
        ty: Type,
    },
    FormToSource {
        span: kw::form_to_source,
        expr: Expr,
    },
    SourceToForm {
        span: kw::source_to_form,
        expr: Expr,
    },
    Default {
        span: kw::default,
        expr: Expr,
    },
    Component {
        span: kw::component,
        shape: Path,
    },
}

impl Parse for GpuiFormFieldOption {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(kw::component) {
            let key = input.parse::<kw::component>()?;
            let content;
            syn::parenthesized!(content in input);
            let shape = content.parse::<Path>()?;
            if !content.is_empty() {
                return Err(content.error(
                    "`component(...)` only accepts a single component shape path, for example `component(MyShape)`",
                ));
            }
            return Ok(Self::Component { span: key, shape });
        }

        if input.peek(Token![type]) {
            let key = input.parse::<Token![type]>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::Type {
                span: key,
                ty: input.parse()?,
            });
        }

        if input.peek(kw::form_to_source) {
            let key = input.parse::<kw::form_to_source>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::FormToSource {
                span: key,
                expr: input.parse()?,
            });
        }

        if input.peek(kw::source_to_form) {
            let key = input.parse::<kw::source_to_form>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::SourceToForm {
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

#[derive(Debug)]
pub struct ComponentField {
    pub ident: Ident,
    pub ty: Type,
    intent: ParsedFieldIntent,
}

#[derive(Debug, Default)]
struct FieldOptions {
    r#type: Option<Spanned<TypeOverride>>,
    form_to_source: Option<Spanned<Expr>>,
    source_to_form: Option<Spanned<Expr>>,
    default: Option<Spanned<DefaultExpr>>,
}

impl FieldOptions {
    fn first_conflict(&self) -> Option<&'static str> {
        [
            ("default", self.default.is_some()),
            ("type", self.r#type.is_some()),
            ("source_to_form", self.source_to_form.is_some()),
            ("form_to_source", self.form_to_source.is_some()),
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

    fn has_component_configuration(&self) -> bool {
        matches!(self.kind, Some(FieldIntentKind::Component(_)))
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
        match &self.intent.kind {
            Some(FieldIntentKind::Component(component)) => {
                ComponentFieldIntent::Component(ComponentRenderedField {
                    r#type: self.intent.options.r#type.as_ref(),
                    form_to_source: self.intent.options.form_to_source.as_ref(),
                    source_to_form: self.intent.options.source_to_form.as_ref(),
                    component,
                    default: self.intent.options.default.as_ref(),
                })
            },
            Some(FieldIntentKind::Hidden) => ComponentFieldIntent::Hidden(
                HiddenField,
                RenderedField {
                    r#type: self.intent.options.r#type.as_ref(),
                    form_to_source: self.intent.options.form_to_source.as_ref(),
                    source_to_form: self.intent.options.source_to_form.as_ref(),
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
        let mut parsed = ComponentField {
            ident,
            ty: field.ty.clone(),
            intent: ParsedFieldIntent::pending(),
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

        if !had_attribute_errors {
            errors.handle(validate_field_intent(&parsed));
        }
        errors.finish_with(parsed)
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
            set_once(
                &mut options.r#type,
                Spanned::new(TypeOverride(ty), &span),
                "type",
                &span,
            )
        },
        GpuiFormFieldOption::FormToSource { span, expr } => {
            let options = field_options_mut(field, "form_to_source", &span)?;
            set_once(
                &mut options.form_to_source,
                Spanned::new(expr, &span),
                "form_to_source",
                &span,
            )
        },
        GpuiFormFieldOption::SourceToForm { span, expr } => {
            let options = field_options_mut(field, "source_to_form", &span)?;
            set_once(
                &mut options.source_to_form,
                Spanned::new(expr, &span),
                "source_to_form",
                &span,
            )
        },
        GpuiFormFieldOption::Default { span, expr } => {
            let options = field_options_mut(field, "default", &span)?;
            set_once(
                &mut options.default,
                Spanned::new(DefaultExpr(expr), &span),
                "default",
                &span,
            )
        },
        GpuiFormFieldOption::Component { span, shape } => set_shape(field, shape, &span),
    }
}

fn set_shape<S: syn::spanned::Spanned>(
    field: &mut ComponentField,
    shape: Path,
    span: &S,
) -> darling::Result<()> {
    match field.intent.kind {
        Some(FieldIntentKind::Component(_)) => {
            return Err(DarlingError::custom(
                "duplicate `shape` option in `gpui_form` field attribute; remove the duplicate `shape` entry",
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

    let component = ShapeOptions::from_shape_with_span(shape, span.span());
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

    if field.intent.has_component_configuration() {
        return Err(intent_conflict_error("component", "hidden").with_span(span));
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

    if field.intent.has_component_configuration() {
        return Err(skip_conflict_error("component").with_span(span));
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
    if field.intent.kind.is_none() {
        return Err(DarlingError::custom(format!(
            "field `{}` must choose a gpui_form field intent; add `component(...)`, `hidden`, or `skip`",
            field.ident
        ))
        .with_span(&field.ident));
    }

    if let Some(type_override) = &field.intent.options.r#type {
        if field.intent.options.source_to_form.is_none() {
            return Err(DarlingError::custom(
                "`type = ...` requires an explicit `source_to_form = ...` conversion",
            )
            .with_span(&type_override.span));
        }

        if field.intent.options.form_to_source.is_none() {
            return Err(DarlingError::custom(
                "`type = ...` requires an explicit `form_to_source = ...` conversion",
            )
            .with_span(&type_override.span));
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
    pub field_structure_tokens: TokenStream,
    pub field_base_declarations_tokens: TokenStream,
    pub required_value: RequiredValue,
}

pub struct GpuiFormOptions {
    pub generate_shape: bool,
}
