use proc_macro2::TokenStream;
use quote::quote;
use syn::Path;

use crate::derives::gpui_form::ir::{DeriveContext, FieldPlan, HolderFieldIr, SkippedFieldPlan};

#[derive(Clone, Debug)]
pub struct ValueHolderPlan<'a> {
    conversion: HolderConversionPlan,
    rendered_fields: Vec<HolderFieldPlan<'a>>,
    skipped_fields: Vec<SkippedHolderFieldPlan<'a>>,
    original_fields: Vec<HolderOriginalFieldPlan<'a>>,
    has_skipped_fields: bool,
    has_shape_policy_storage: bool,
    has_any_conversion: bool,
    has_any_required: bool,
    has_shape_policy_required: bool,
}

impl<'a> ValueHolderPlan<'a> {
    pub fn new(fields: &'a [FieldPlan]) -> syn::Result<Self> {
        let conversion = HolderConversionPlan::from_fields(fields)?;
        let mut rendered_fields = Vec::new();
        let mut skipped_fields = Vec::new();
        let mut original_fields = Vec::new();

        for field in fields {
            match field {
                FieldPlan::Skipped(plan) => {
                    skipped_fields.push(SkippedHolderFieldPlan::new(plan.as_ref()));
                    original_fields.push(HolderOriginalFieldPlan::Skipped(plan.as_ref()));
                },
                FieldPlan::Hidden(plan) => {
                    rendered_fields.push(HolderFieldPlan::new(&plan.shared));
                    original_fields.push(HolderOriginalFieldPlan::Rendered(&plan.shared));
                },
                FieldPlan::Component(plan) => {
                    rendered_fields.push(HolderFieldPlan::new(&plan.shared));
                    original_fields.push(HolderOriginalFieldPlan::Rendered(&plan.shared));
                },
            }
        }

        let has_skipped_fields = !skipped_fields.is_empty();
        let has_shape_policy_storage = rendered_fields
            .iter()
            .any(|field| field.storage.is_shape_policy);
        let has_any_conversion = rendered_fields.iter().any(|field| {
            field.conversion.needs_from_conversion || field.conversion.needs_into_conversion
        });
        let has_any_required = rendered_fields
            .iter()
            .any(|field| field.validation.needs_required);
        let has_shape_policy_required = rendered_fields
            .iter()
            .any(|field| field.validation.needs_shape_policy_required);

        Ok(Self {
            conversion,
            rendered_fields,
            skipped_fields,
            original_fields,
            has_skipped_fields,
            has_shape_policy_storage,
            has_any_conversion,
            has_any_required,
            has_shape_policy_required,
        })
    }

    pub fn conversion_mode(&self) -> HolderConversionMode {
        self.conversion.mode()
    }

    pub fn conversion_metadata_tokens(&self, context: &DeriveContext) -> TokenStream {
        self.conversion.metadata_tokens(context)
    }

    pub fn rendered_fields(&self) -> &[HolderFieldPlan<'a>] {
        &self.rendered_fields
    }

    pub fn skipped_fields(&self) -> &[SkippedHolderFieldPlan<'a>] {
        &self.skipped_fields
    }

    pub fn original_fields(&self) -> &[HolderOriginalFieldPlan<'a>] {
        &self.original_fields
    }

    pub fn has_skipped_fields(&self) -> bool {
        self.has_skipped_fields
    }

    pub fn has_shape_policy_storage(&self) -> bool {
        self.has_shape_policy_storage
    }

    pub fn has_any_conversion(&self) -> bool {
        self.has_any_conversion
    }

    pub fn has_any_required(&self) -> bool {
        self.has_any_required
    }

    pub fn has_shape_policy_required(&self) -> bool {
        self.has_shape_policy_required
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
                    <<#shape as #runtime_crate::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicy
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
struct HolderConversionPlan {
    mode: HolderConversionMode,
    fallibility: ConversionFallibility,
}

impl HolderConversionPlan {
    fn from_fields(fields: &[FieldPlan]) -> syn::Result<Self> {
        let mut predicates = Vec::new();
        let mut has_maybe_fallible_field = false;
        let has_skipped_fields = fields.iter().any(FieldPlan::is_skipped);
        let has_fallible_conversion = fields
            .iter()
            .filter_map(FieldPlan::shared)
            .any(HolderFieldIr::form_to_source_is_fallible);

        for field in fields.iter().filter_map(FieldPlan::shared) {
            if field.default_expr().is_some() {
                continue;
            }

            if let Some(shape) = field.storage().shape_policy() {
                has_maybe_fallible_field = true;
                predicates.push(ShapePolicyPredicate::RequiresValue {
                    shape: shape.clone(),
                });
            }
        }

        let fallibility = if has_skipped_fields || has_fallible_conversion {
            ConversionFallibility::Always
        } else if predicates.is_empty() {
            ConversionFallibility::Never
        } else {
            ConversionFallibility::DependsOnShape(predicates)
        };
        let mode = if has_skipped_fields {
            HolderConversionMode::SkippedFields
        } else if has_maybe_fallible_field || has_fallible_conversion {
            HolderConversionMode::FallibleRequired
        } else {
            HolderConversionMode::Infallible
        };

        Ok(Self { mode, fallibility })
    }

    fn mode(&self) -> HolderConversionMode {
        self.mode
    }

    fn can_fail_tokens(&self, context: &DeriveContext) -> TokenStream {
        self.fallibility.to_tokens(context)
    }

    fn metadata_tokens(&self, context: &DeriveContext) -> TokenStream {
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

#[derive(Clone, Copy, Debug)]
pub struct SkippedHolderFieldPlan<'a> {
    pub field: &'a SkippedFieldPlan,
}

impl<'a> SkippedHolderFieldPlan<'a> {
    fn new(field: &'a SkippedFieldPlan) -> Self {
        Self { field }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum HolderOriginalFieldPlan<'a> {
    Skipped(&'a SkippedFieldPlan),
    Rendered(&'a HolderFieldIr),
}

#[derive(Clone, Debug)]
pub struct HolderFieldPlan<'a> {
    pub field: &'a HolderFieldIr,
    pub storage: HolderStorageFieldPlan<'a>,
    pub default: HolderDefaultPlan<'a>,
    pub conversion: HolderConversionFieldPlan,
    pub validation: HolderValidationFieldPlan,
    pub present: PresentFieldPlan,
}

impl<'a> HolderFieldPlan<'a> {
    fn new(field: &'a HolderFieldIr) -> Self {
        Self {
            field,
            storage: HolderStorageFieldPlan::new(field),
            default: HolderDefaultPlan::new(field),
            conversion: HolderConversionFieldPlan::new(field),
            validation: HolderValidationFieldPlan::new(field),
            present: PresentFieldPlan::new(field),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HolderStorageFieldPlan<'a> {
    pub was_optional: bool,
    pub is_shape_policy: bool,
    pub shape_policy: Option<&'a syn::Path>,
}

impl<'a> HolderStorageFieldPlan<'a> {
    fn new(field: &'a HolderFieldIr) -> Self {
        let shape_policy = field.storage().shape_policy();
        Self {
            was_optional: field.was_optional,
            is_shape_policy: shape_policy.is_some(),
            shape_policy,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HolderDefaultPlan<'a> {
    pub has_default: bool,
    pub needs_shape_default_assertion: bool,
    pub shape_default_policy: Option<&'a syn::Path>,
}

impl<'a> HolderDefaultPlan<'a> {
    fn new(field: &'a HolderFieldIr) -> Self {
        let has_default = field.default_expr().is_some();
        let needs_shape_default_assertion =
            !field.was_optional && !has_default && field.storage().is_shape_policy();
        Self {
            has_default,
            needs_shape_default_assertion,
            shape_default_policy: needs_shape_default_assertion
                .then(|| field.storage().shape_policy())
                .flatten(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HolderConversionFieldPlan {
    pub needs_from_conversion: bool,
    pub needs_into_conversion: bool,
    pub into_conversion_is_fallible: bool,
}

impl HolderConversionFieldPlan {
    fn new(field: &HolderFieldIr) -> Self {
        Self {
            needs_from_conversion: field.source_to_form_expr().is_some(),
            needs_into_conversion: field.form_to_source_expr().is_some(),
            into_conversion_is_fallible: field.form_to_source_is_fallible(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HolderValidationFieldPlan {
    pub needs_required: bool,
    pub needs_shape_policy_required: bool,
    pub has_existing_validations: bool,
    pub is_newtype: bool,
    pub is_nested: bool,
}

impl HolderValidationFieldPlan {
    fn new(field: &HolderFieldIr) -> Self {
        let validation = field.validation();
        let needs_required = field.needs_required_validation();
        Self {
            needs_required,
            needs_shape_policy_required: needs_required && field.storage().is_shape_policy(),
            has_existing_validations: !validation.field_validators.is_empty()
                || !validation.element_validators.is_empty(),
            is_newtype: validation.is_newtype,
            is_nested: validation.is_nested,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PresentFieldPlan {
    pub borrows_source_value: bool,
}

impl PresentFieldPlan {
    fn new(field: &HolderFieldIr) -> Self {
        Self {
            borrows_source_value: !field.storage().is_shape_policy()
                && field.form_to_source_expr().is_none(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::derives::gpui_form::ir::{
        FieldAttrContext, FieldMetadata, HolderStoragePlan, KorumaValidationInfo,
        RenderedValueIntent, ValidationMetadata,
    };
    use proc_macro2::Span;

    fn holder_field(storage: HolderStoragePlan, was_optional: bool) -> HolderFieldIr {
        HolderFieldIr {
            field_name: syn::parse_quote!(name),
            original_type: syn::parse_quote!(String),
            source_value_type: syn::parse_quote!(String),
            form_type: syn::parse_quote!(String),
            was_optional,
            storage,
            validation: KorumaValidationInfo::default(),
            validation_metadata: ValidationMetadata::default(),
            metadata: FieldMetadata::default(),
            default_expr: None,
            default_span: None,
            value_mapping: RenderedValueIntent::Identity,
            context: FieldAttrContext::new(Span::call_site(), Span::call_site()),
        }
    }

    #[test]
    fn holder_plan_records_originally_optional_field_subplans() {
        let field = holder_field(HolderStoragePlan::OriginallyOptional, true);
        let fields = vec![FieldPlan::hidden(field)];

        let plan = ValueHolderPlan::new(&fields).expect("holder should plan");
        let field = &plan.rendered_fields()[0];

        assert!(!plan.has_skipped_fields());
        assert!(!plan.has_any_required());
        assert_eq!(plan.conversion_mode(), HolderConversionMode::Infallible);
        assert_eq!(
            field.storage,
            HolderStorageFieldPlan {
                was_optional: true,
                is_shape_policy: false,
                shape_policy: None
            }
        );
        assert!(field.present.borrows_source_value);
    }

    #[test]
    fn holder_plan_records_shape_policy_required_subplans() {
        let field = holder_field(
            HolderStoragePlan::ShapePolicy {
                shape: syn::parse_quote!(crate::RequiredShape),
            },
            false,
        );
        let fields = vec![FieldPlan::hidden(field)];

        let plan = ValueHolderPlan::new(&fields).expect("holder should plan");
        let field = &plan.rendered_fields()[0];

        assert!(plan.has_any_required());
        assert!(plan.has_shape_policy_required());
        assert_eq!(
            plan.conversion_mode(),
            HolderConversionMode::FallibleRequired
        );
        assert!(field.storage.is_shape_policy);
        assert!(field.storage.shape_policy.is_some());
        assert!(field.default.needs_shape_default_assertion);
        assert!(field.default.shape_default_policy.is_some());
        assert!(field.validation.needs_shape_policy_required);
        assert!(!field.present.borrows_source_value);
    }

    #[test]
    fn holder_plan_preserves_original_order_with_skipped_fields() {
        let editable = holder_field(HolderStoragePlan::Direct, false);
        let fields = vec![
            FieldPlan::skipped(syn::parse_quote!(id), syn::parse_quote!(u64)),
            FieldPlan::hidden(editable),
        ];

        let plan = ValueHolderPlan::new(&fields).expect("holder should plan");

        assert!(plan.has_skipped_fields());
        assert_eq!(plan.conversion_mode(), HolderConversionMode::SkippedFields);
        assert_eq!(plan.skipped_fields().len(), 1);
        assert_eq!(plan.rendered_fields().len(), 1);
        assert!(matches!(
            plan.original_fields(),
            [
                HolderOriginalFieldPlan::Skipped(_),
                HolderOriginalFieldPlan::Rendered(_)
            ]
        ));
    }
}
