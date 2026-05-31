use darling::{Error as DarlingError, FromField, FromMeta};
use gpui_form_codegen::{
    CratePaths,
    components::{ComponentFieldIr, RequiredValue, ResolvedComponentShape, ShapeOptions},
};
use koruma_derive_core::ValidationInfo;
use proc_macro2::Span;
use syn::{
    Expr, Ident, Path, Token, Type,
    parse::{Parse, ParseStream, Parser as _},
    punctuated::Punctuated,
};

mod kw {
    syn::custom_keyword!(component);
    syn::custom_keyword!(default);
    syn::custom_keyword!(from_source);
    syn::custom_keyword!(hidden);
    syn::custom_keyword!(into_source);
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
pub struct FieldAttrContext {
    pub attr_span: Span,
    pub option_span: Span,
}

impl FieldAttrContext {
    pub fn new(attr_span: Span, option_span: Span) -> Self {
        Self {
            attr_span,
            option_span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DeriveContext {
    pub original_ident: Ident,
    pub paths: CratePaths,
}

impl DeriveContext {
    pub fn new(original_ident: Ident, paths: CratePaths) -> Self {
        Self {
            original_ident,
            paths,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FieldContext {
    pub field_ident: Ident,
    pub field_ty: Type,
    pub field_span: Span,
}

impl FieldContext {
    pub fn new(field_ident: Ident, field_ty: Type, field_span: Span) -> Self {
        Self {
            field_ident,
            field_ty,
            field_span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypeOverride(pub Type);

#[derive(Clone, Debug)]
pub struct DefaultExpr(pub Expr);

#[derive(Clone, Debug)]
pub struct ConvertedValueIntent {
    pub form_type: Spanned<TypeOverride>,
    pub from_source: Spanned<Expr>,
    pub into_source: Spanned<Expr>,
}

#[derive(Clone, Debug)]
pub enum RenderedValueIntent {
    Identity,
    Converted(ConvertedValueIntent),
}

impl RenderedValueIntent {
    pub fn source_to_form(&self) -> Option<&Spanned<Expr>> {
        match self {
            Self::Identity => None,
            Self::Converted(converted) => Some(&converted.from_source),
        }
    }

    pub fn form_to_source(&self) -> Option<&Spanned<Expr>> {
        match self {
            Self::Identity => None,
            Self::Converted(converted) => Some(&converted.into_source),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RenderedFieldIntent {
    pub value: RenderedValueIntent,
    pub default: Option<Spanned<DefaultExpr>>,
}

impl Default for RenderedFieldIntent {
    fn default() -> Self {
        Self {
            value: RenderedValueIntent::Identity,
            default: None,
        }
    }
}

/// Value-holder storage selected during derive planning.
#[derive(Clone, Debug)]
pub enum HolderStoragePlan {
    OriginallyOptional,
    RequiredValue,
    Direct,
    ShapePolicy { shape: Path },
}

impl HolderStoragePlan {
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

    pub fn shape_policy(&self) -> Option<&Path> {
        match self {
            Self::ShapePolicy { shape } => Some(shape),
            Self::OriginallyOptional | Self::RequiredValue | Self::Direct => None,
        }
    }

    pub fn is_shape_policy(&self) -> bool {
        self.shape_policy().is_some()
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
pub struct HolderFieldIr {
    pub field_name: Ident,
    #[allow(dead_code)]
    pub original_type: Type,
    #[allow(dead_code)]
    pub source_value_type: Type,
    pub form_type: Type,
    pub was_optional: bool,
    pub storage: HolderStoragePlan,
    pub validation: ValidationInfo,
    pub validation_metadata: ValidationMetadata,
    pub default_expr: Option<Expr>,
    pub default_span: Option<Span>,
    pub value_mapping: RenderedValueIntent,
    pub context: FieldAttrContext,
}

impl HolderFieldIr {
    pub fn field_name(&self) -> &Ident {
        &self.field_name
    }

    pub fn original_type(&self) -> &Type {
        &self.original_type
    }

    pub fn form_type(&self) -> &Type {
        &self.form_type
    }

    pub fn storage(&self) -> &HolderStoragePlan {
        &self.storage
    }

    pub fn validation(&self) -> &ValidationInfo {
        &self.validation
    }

    pub fn default_expr(&self) -> Option<&Expr> {
        self.default_expr.as_ref()
    }

    pub fn default_expr_span(&self) -> Span {
        self.default_span.unwrap_or(self.context.attr_span)
    }

    pub fn source_to_form_expr(&self) -> Option<&Expr> {
        self.value_mapping
            .source_to_form()
            .map(|from_source| &from_source.value)
    }

    pub fn source_to_form_span(&self) -> Span {
        self.value_mapping
            .source_to_form()
            .map(|from_source| from_source.span)
            .unwrap_or(self.context.option_span)
    }

    pub fn form_to_source_expr(&self) -> Option<&Expr> {
        self.value_mapping
            .form_to_source()
            .map(|into_source| &into_source.value)
    }

    pub fn form_to_source_span(&self) -> Span {
        self.value_mapping
            .form_to_source()
            .map(|into_source| into_source.span)
            .unwrap_or(self.context.option_span)
    }

    /// Returns true if this rendered field needs the `RequiredValidation`
    /// koruma validator.
    pub fn needs_required_validation(&self) -> bool {
        matches!(
            self.storage,
            HolderStoragePlan::RequiredValue | HolderStoragePlan::ShapePolicy { .. }
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
    pub shared: HolderFieldIr,
}

#[derive(Clone, Debug)]
pub struct ComponentFieldPlan {
    pub shared: HolderFieldIr,
    pub component: ComponentFieldIr,
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

    pub fn hidden(shared: HolderFieldIr) -> Self {
        Self::Hidden(HiddenFieldPlan { shared })
    }

    pub fn component_field(shared: HolderFieldIr, component: ComponentFieldIr) -> Self {
        Self::Component(ComponentFieldPlan { shared, component })
    }

    pub fn shared(&self) -> Option<&HolderFieldIr> {
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
            Self::Component(plan) => Some(&plan.component.shape),
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
            HolderStoragePlan::RequiredValue | HolderStoragePlan::ShapePolicy { .. }
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
        options: RenderedFieldIntent,
    },
    Component {
        span: kw::component,
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
                span: key,
                shape,
                options,
            });
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
                RenderedFieldIntent::default()
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

#[derive(Debug)]
pub struct ComponentField {
    pub context: FieldContext,
    attr: FieldAttr,
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
        GpuiFormFieldOption::Skip { span } => set_attr(field, FieldAttr::Skipped, &span),
        GpuiFormFieldOption::Hidden { span, options } => {
            let context = FieldAttrContext::new(attr_span, span.span);
            set_attr(
                field,
                FieldAttr::Hidden {
                    rendered: options,
                    context,
                },
                &span,
            )
        },
        GpuiFormFieldOption::Component {
            span,
            shape,
            options,
        } => {
            let context = FieldAttrContext::new(attr_span, span.span);
            let component = ShapeOptions::from_shape_with_span(shape, context.option_span);
            set_attr(
                field,
                FieldAttr::Component(ComponentFieldOptions {
                    component,
                    rendered: options,
                    context,
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

pub struct GpuiFormOptions {
    pub generate_shape: bool,
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
