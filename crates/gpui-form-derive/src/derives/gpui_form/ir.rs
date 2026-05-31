use gpui_form_codegen::components::{ComponentFieldIr, RequiredValue, ResolvedComponentShape};
use koruma_derive_core::ValidationInfo;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Expr, Ident, Path, Type};

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
    pub paths: gpui_form_codegen::CratePaths,
}

impl DeriveContext {
    pub fn new(original_ident: Ident, paths: gpui_form_codegen::CratePaths) -> Self {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HolderConversionMode {
    Infallible,
    FallibleRequired,
    SkippedFields,
}

#[derive(Clone, Debug)]
enum ShapePolicyPredicate {
    RequiresValue { shape: Path },
}

impl ShapePolicyPredicate {
    fn to_tokens(&self, context: &DeriveContext) -> TokenStream {
        let runtime_crate = context.paths.gpui_form_facade_runtime();
        match self {
            Self::RequiresValue { shape } => {
                quote! {
                    <<#shape as #runtime_crate::shape::ComponentShape>::ValueStoragePolicy
                        as #runtime_crate::shape::ComponentValueStoragePolicy>::REQUIRES_VALUE
                }
            },
        }
    }
}

#[derive(Clone, Debug)]
enum ConversionFallibility {
    Never,
    Always,
    DependsOnShape(Vec<ShapePolicyPredicate>),
}

impl ConversionFallibility {
    fn to_tokens(&self, context: &DeriveContext) -> TokenStream {
        match self {
            Self::Never => quote! { false },
            Self::Always => quote! { true },
            Self::DependsOnShape(predicates) => {
                let predicates = predicates
                    .iter()
                    .map(|predicate| predicate.to_tokens(context))
                    .collect::<Vec<_>>();
                quote! { false #(|| #predicates)* }
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct HolderConversionPlan {
    mode: HolderConversionMode,
    fallibility: ConversionFallibility,
}

impl HolderConversionPlan {
    pub fn mode(&self) -> HolderConversionMode {
        self.mode
    }

    pub fn can_fail_tokens(&self, context: &DeriveContext) -> TokenStream {
        self.fallibility.to_tokens(context)
    }

    pub fn metadata_tokens(&self, context: &DeriveContext) -> TokenStream {
        let facade_crate = &context.paths.gpui_form;
        let shape = match self.mode {
            HolderConversionMode::Infallible => {
                quote! { #facade_crate::schema::registry::HolderConversionShape::Infallible }
            },
            HolderConversionMode::FallibleRequired => {
                quote! { #facade_crate::schema::registry::HolderConversionShape::FallibleRequired }
            },
            HolderConversionMode::SkippedFields => {
                quote! { #facade_crate::schema::registry::HolderConversionShape::NeedsSkippedFields }
            },
        };
        let runtime_can_fail = self.can_fail_tokens(context);

        quote! {
            #facade_crate::schema::registry::HolderConversionMetadata::new(
                #shape,
                #runtime_can_fail
            )
        }
    }
}

pub fn holder_conversion_plan(fields: &[FieldPlan]) -> syn::Result<HolderConversionPlan> {
    let mut predicates = Vec::new();
    let mut has_maybe_fallible_field = false;
    let mut has_hard_fallible_field = false;
    let has_skipped_fields = fields.iter().any(FieldPlan::is_skipped);

    for field in fields.iter().filter_map(FieldPlan::shared) {
        if field.default_expr().is_some() {
            continue;
        }

        if matches!(field.storage(), HolderStoragePlan::RequiredValue) {
            has_maybe_fallible_field = true;
            has_hard_fallible_field = true;
        } else if let Some(shape) = field.storage().shape_policy() {
            has_maybe_fallible_field = true;
            predicates.push(ShapePolicyPredicate::RequiresValue {
                shape: shape.clone(),
            });
        }
    }

    let fallibility = if has_skipped_fields || has_hard_fallible_field {
        ConversionFallibility::Always
    } else if predicates.is_empty() {
        ConversionFallibility::Never
    } else {
        ConversionFallibility::DependsOnShape(predicates)
    };
    let mode = if has_skipped_fields {
        HolderConversionMode::SkippedFields
    } else if has_maybe_fallible_field {
        HolderConversionMode::FallibleRequired
    } else {
        HolderConversionMode::Infallible
    };

    Ok(HolderConversionPlan { mode, fallibility })
}
