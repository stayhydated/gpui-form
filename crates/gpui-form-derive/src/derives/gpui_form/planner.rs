use syn::DeriveInput;

use crate::derives::gpui_form::components::generate_component_field;
use crate::derives::gpui_form::intent::{ComponentFieldIntent, ComponentStruct};
use crate::derives::gpui_form::ir::{
    ComponentFieldPlan, FieldPlan, HolderFieldIr, HolderStoragePlan, KorumaValidationInfo,
    RenderedValueIntent,
};
use crate::derives::gpui_form::utils::extract_option_inner_type;
use crate::derives::gpui_form::validation::{
    KorumaOptions, parse_koruma_fields, validation_metadata_from_koruma,
};

pub struct FormPlan {
    pub fields: Vec<FieldPlan>,
    pub enable_koruma_fluent: bool,
    pub effective_enable_koruma: bool,
}

impl FormPlan {
    #[allow(dead_code)]
    pub fn non_skipped_fields(&self) -> impl Iterator<Item = &HolderFieldIr> {
        self.fields.iter().filter_map(FieldPlan::shared)
    }

    pub fn component_fields(&self) -> impl Iterator<Item = &ComponentFieldPlan> {
        self.fields.iter().filter_map(FieldPlan::component_plan)
    }
}

pub fn plan_form(
    parsed: &ComponentStruct,
    derive_input: &DeriveInput,
    koruma_options: Option<&KorumaOptions>,
) -> syn::Result<FormPlan> {
    let fields_iter = match &parsed.data {
        darling::ast::Data::Struct(s) => &s.fields,
        _ => {
            return Err(syn::Error::new_spanned(
                derive_input,
                "`#[derive(GpuiForm)]` only supports named structs",
            ));
        },
    };

    let parsed_koruma_fields = parse_koruma_fields(derive_input)?;
    let enable_koruma = koruma_options.is_some() || !parsed_koruma_fields.is_empty();
    let enable_koruma_fluent = koruma_options.map(|k| k.fluent).unwrap_or(false);

    let mut fields = Vec::new();
    for field in fields_iter {
        let field_name = field.context.field_ident.clone();
        let intent = field.intent();

        if matches!(intent, ComponentFieldIntent::Skipped(_)) {
            fields.push(FieldPlan::skipped(
                field_name,
                field.context.field_ty.clone(),
            ));
            continue;
        }

        let field_name_str = field_name.to_string();
        let (was_optional, source_value_type) = extract_option_inner_type(&field.context.field_ty);
        let rendered = field.rendered().ok_or_else(|| {
            syn::Error::new_spanned(
                &field_name,
                "field attribute must provide rendered options unless the field is `skip`d",
            )
        })?;
        let value_mapping = rendered.value.clone();
        let form_type = match &value_mapping {
            RenderedValueIntent::Identity => source_value_type.clone(),
            RenderedValueIntent::Converted(converted) => {
                extract_option_inner_type(&converted.form_type.value.0).1
            },
        };

        let (component, storage) = match field.component() {
            Some(component) => {
                let component = generate_component_field(
                    &field_name,
                    &form_type,
                    component.component,
                    component.rendered.context,
                )?;
                let storage = HolderStoragePlan::from_component_shape_storage_policy(
                    was_optional,
                    component.storage_policy.clone(),
                );
                (Some(component), storage)
            },
            None => (
                None,
                HolderStoragePlan::from_non_component_field(was_optional),
            ),
        };

        let koruma_info = parsed_koruma_fields.get(&field_name_str);
        let validation = koruma_info
            .map(KorumaValidationInfo::from)
            .unwrap_or_default();
        let validation_metadata = koruma_info
            .map(validation_metadata_from_koruma)
            .unwrap_or_default();

        let shared = HolderFieldIr {
            field_name,
            original_type: field.context.field_ty.clone(),
            source_value_type,
            form_type,
            was_optional,
            storage,
            validation,
            validation_metadata,
            default_expr: rendered.default.map(|expr| expr.value.0.clone()),
            default_span: rendered.default.map(|expr| expr.span),
            value_mapping,
            context: rendered.context.clone(),
        };

        if let Some(component) = component {
            fields.push(FieldPlan::component_field(shared, component));
        } else {
            fields.push(FieldPlan::hidden(shared));
        }
    }

    let has_fields_needing_required = fields
        .iter()
        .filter_map(FieldPlan::shared)
        .any(|field| field.needs_required_validation() && !field.validation().is_newtype);
    let has_any_koruma_validations = fields.iter().filter_map(FieldPlan::shared).any(|field| {
        let validation = field.validation();
        !validation.field_validators.is_empty()
            || !validation.element_validators.is_empty()
            || validation.is_nested
    });
    let effective_enable_koruma =
        enable_koruma || (has_fields_needing_required && has_any_koruma_validations);
    Ok(FormPlan {
        fields,
        enable_koruma_fluent,
        effective_enable_koruma,
    })
}
