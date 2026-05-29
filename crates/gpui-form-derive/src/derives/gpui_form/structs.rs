use darling::{Error as DarlingError, FromField, FromMeta};
use gpui_form_codegen::components::{RequiredValue, ShapeOptions};
use koruma_derive_core::ValidationInfo;
use proc_macro2::{Span, TokenStream};
use syn::{
    Expr, Ident, LitStr, Path, Token, Type,
    parse::{Parse, ParseStream, Parser as _},
    punctuated::Punctuated,
};

mod kw {
    syn::custom_keyword!(default);
    syn::custom_keyword!(component);
    syn::custom_keyword!(field_suffix);
    syn::custom_keyword!(from);
    syn::custom_keyword!(hidden);
    syn::custom_keyword!(into);
    syn::custom_keyword!(shape);
    syn::custom_keyword!(skip);
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
    ShapePolicy(Path),
}

impl StoragePlan {
    pub fn from_required_value(was_optional: bool, required_value: RequiredValue) -> Self {
        if was_optional {
            Self::OriginallyOptional
        } else {
            match required_value {
                RequiredValue::Explicit(true) => Self::RequiredValue,
                RequiredValue::Explicit(false) => Self::Direct,
                RequiredValue::Shape(shape) => Self::ShapePolicy(shape),
            }
        }
    }

    pub fn required_value(&self) -> RequiredValue {
        match self {
            Self::OriginallyOptional | Self::Direct => RequiredValue::explicit(false),
            Self::RequiredValue => RequiredValue::explicit(true),
            Self::ShapePolicy(shape) => RequiredValue::Shape(shape.clone()),
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
    pub fn registry_name(&self) -> &str {
        match self {
            Self::Validator(name) => name,
            Self::Required => "RequiredValidation",
            Self::Newtype => "NewtypeValidation",
            Self::Nested => "NestedValidation",
        }
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

    pub fn has_registry_name(&self, name: &str) -> bool {
        self.rules.iter().any(|rule| rule.registry_name() == name)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ValidationRule> {
        self.rules.iter()
    }
}

/// Precomputed field facts shared by expansion, inventory, and value holders.
pub struct FieldPlan {
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
    pub override_type: Option<Type>,
    pub into_expr: Option<Expr>,
    pub from_expr: Option<Expr>,
    pub component: Option<ShapeOptions>,
    pub component_layout: Option<ComponentFieldContent>,
    pub value_storage_policy_ident: Option<Ident>,
    pub skip: bool,
}

impl FieldPlan {
    /// Returns true if this field needs the `RequiredValidation` koruma validator.
    /// This applies to fields that:
    /// - Can be missing in the form holder
    /// - Must be present in the source struct
    /// - Are not nested structs (nested fields have their own validation)
    pub fn needs_required_validation(&self) -> bool {
        !self.skip
            && matches!(
                self.storage,
                StoragePlan::RequiredValue | StoragePlan::ShapePolicy(_)
            )
            && !self.was_optional
            && !self.validation.is_nested
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
    Into {
        span: kw::into,
        expr: Expr,
    },
    From {
        span: kw::from,
        expr: Expr,
    },
    Default {
        span: kw::default,
        expr: Expr,
    },
    Shape {
        span: kw::shape,
        path: Path,
    },
    Component {
        span: kw::component,
        ty: Type,
    },
    FieldSuffix {
        span: kw::field_suffix,
        suffix: LitStr,
    },
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

        if input.peek(kw::shape) {
            let key = input.parse::<kw::shape>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::Shape {
                span: key,
                path: input.parse()?,
            });
        }

        if input.peek(kw::component) {
            let key = input.parse::<kw::component>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::Component {
                span: key,
                ty: input.parse()?,
            });
        }

        if input.peek(kw::field_suffix) {
            let key = input.parse::<kw::field_suffix>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::FieldSuffix {
                span: key,
                suffix: input.parse()?,
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
                     `shape = MyShape`"
                ),
            ));
        }

        Err(input.error(
            "expected gpui_form field option such as `shape = MyShape`, `hidden`, or `skip`",
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
    pending_component: Option<(Type, Span)>,
    pending_field_suffix: Option<(LitStr, Span)>,
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
            pending_component: None,
            pending_field_suffix: None,
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
            || self.pending_component.is_some()
            || self.pending_field_suffix.is_some()
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
        let mut errors = DarlingError::accumulator();
        let mut had_attribute_errors = false;

        for attr in &field.attrs {
            if !attr.path().is_ident("gpui_form") {
                continue;
            }

            let list = match attr.meta.require_list().map_err(|_| {
                DarlingError::custom(
                    "`gpui_form` field attribute must be a list, for example \
                     `#[gpui_form(shape = my::Shape)]`",
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
        GpuiFormFieldOption::Shape { span, path } => set_shape(field, path, &span),
        GpuiFormFieldOption::Component { span, ty } => set_component_type(field, ty, &span),
        GpuiFormFieldOption::FieldSuffix { span, suffix } => set_field_suffix(field, suffix, &span),
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

    let component = field
        .intent
        .pending_component
        .take()
        .map(|(component, _)| component);
    let field_suffix = field
        .intent
        .pending_field_suffix
        .take()
        .map(|(field_suffix, _)| field_suffix);
    let component = ShapeOptions::from_explicit_metadata(shape, component, field_suffix)?;
    field.intent.kind = Some(FieldIntentKind::Component(component));
    Ok(())
}

fn set_component_type<S: syn::spanned::Spanned>(
    field: &mut ComponentField,
    component: Type,
    span: &S,
) -> darling::Result<()> {
    match &mut field.intent.kind {
        Some(FieldIntentKind::Component(shape)) => shape.set_component(component, span),
        Some(FieldIntentKind::Hidden) => {
            Err(intent_conflict_error("hidden", "component").with_span(span))
        },
        Some(FieldIntentKind::Skipped) => Err(skip_conflict_error("component").with_span(span)),
        None => set_pending_once(
            &mut field.intent.pending_component,
            component,
            span.span(),
            "component",
            span,
        ),
    }
}

fn set_field_suffix<S: syn::spanned::Spanned>(
    field: &mut ComponentField,
    field_suffix: LitStr,
    span: &S,
) -> darling::Result<()> {
    match &mut field.intent.kind {
        Some(FieldIntentKind::Component(shape)) => shape.set_field_suffix(field_suffix, span),
        Some(FieldIntentKind::Hidden) => {
            Err(intent_conflict_error("hidden", "field_suffix").with_span(span))
        },
        Some(FieldIntentKind::Skipped) => Err(skip_conflict_error("field_suffix").with_span(span)),
        None => set_pending_once(
            &mut field.intent.pending_field_suffix,
            field_suffix,
            span.span(),
            "field_suffix",
            span,
        ),
    }
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

fn set_pending_once<T, S: syn::spanned::Spanned>(
    slot: &mut Option<(T, Span)>,
    value: T,
    value_span: Span,
    key: &str,
    span: &S,
) -> darling::Result<()> {
    if slot.is_some() {
        return Err(DarlingError::custom(format!(
            "duplicate `{key}` option in `gpui_form` field attribute; remove the duplicate `{key}` entry"
        ))
        .with_span(span));
    }

    *slot = Some((value, value_span));
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
    if field.intent.kind.is_some() {
        return Ok(());
    }

    if field.intent.pending_component.is_some() || field.intent.pending_field_suffix.is_some() {
        return Err(DarlingError::custom(format!(
            "field `{}` has component metadata but no shape; add `shape = ...`",
            field.ident
        ))
        .with_span(&field.ident));
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
    pub required_value: RequiredValue,
}

pub struct GpuiFormOptions {
    pub generate_shape: bool,
}
