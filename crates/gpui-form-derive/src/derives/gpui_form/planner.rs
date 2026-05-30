use gpui_form_codegen::components::RequiredValue;
use koruma_derive_core::{FieldInfo as KorumaFieldInfo, ParseFieldResult};
use std::collections::HashMap;
use syn::DeriveInput;

use crate::derives::gpui_form::components::generate_component_field;
use crate::derives::gpui_form::structs::{
    ComponentFieldIntent, ComponentStruct, FieldPlan, KorumaOptions, SharedFieldPlan, StoragePlan,
    ValidationMetadata, ValidationRule,
};
use crate::derives::gpui_form::utils::extract_option_inner_type;

pub struct FormPlan {
    pub fields: Vec<FieldPlan>,
    pub has_skipped_fields: bool,
    pub enable_koruma_fluent: bool,
    pub effective_enable_koruma: bool,
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
        let field_name = field.ident.clone();
        let intent = field.intent();

        if matches!(intent, ComponentFieldIntent::Skipped(_)) {
            fields.push(FieldPlan::skipped(field_name, field.ty.clone()));
            continue;
        }

        let field_name_str = field_name.to_string();
        let (was_optional, source_value_type) = extract_option_inner_type(&field.ty);
        let rendered = field.rendered().ok_or_else(|| {
            syn::Error::new_spanned(
                &field_name,
                "field attribute must provide rendered options unless the field is `skip`d",
            )
        })?;
        let override_type = rendered
            .r#type
            .as_ref()
            .map(|rendered_type| rendered_type.value.0.clone());
        let form_type = override_type
            .as_ref()
            .map(|ty| extract_option_inner_type(ty).1)
            .unwrap_or_else(|| source_value_type.clone());

        let (component, component_layout, required_value) = match field.component() {
            Some(component) => {
                let layout = generate_component_field(&field_name, &source_value_type, &component);
                (
                    Some(component.component.clone()),
                    Some(layout.clone()),
                    layout.required_value,
                )
            },
            None => (None, None, RequiredValue::explicit(false)),
        };
        let storage = StoragePlan::from_required_value(&field_name, was_optional, required_value)?;

        let koruma_info = parsed_koruma_fields.get(&field_name_str);
        let validation = koruma_info
            .map(|info| info.validation.clone())
            .unwrap_or_default();
        let mut validation_metadata = ValidationMetadata::default();
        if let Some(info) = koruma_info {
            for validator in &info.validation.field_validators {
                if validator.name() == "RequiredValidation" {
                    validation_metadata.push(ValidationRule::Required);
                } else {
                    validation_metadata
                        .push(ValidationRule::Validator(validator.name().to_string()));
                }
            }
            if info.is_newtype() {
                validation_metadata.push(ValidationRule::Newtype);
            }
            if info.is_nested() {
                validation_metadata.push(ValidationRule::Nested);
            }
        }

        let shared = SharedFieldPlan {
            field_name,
            original_type: field.ty.clone(),
            source_value_type,
            form_type,
            was_optional,
            storage,
            validation,
            validation_metadata,
            default_expr: rendered.default.map(|expr| expr.value.0.clone()),
            override_type,
            into_expr: rendered
                .form_to_source
                .map(|form_to_source| form_to_source.value.clone()),
            from_expr: rendered
                .source_to_form
                .map(|source_to_form| source_to_form.value.clone()),
        };

        if let (Some(component), Some(component_layout)) = (component, component_layout) {
            fields.push(FieldPlan::component_field(
                shared,
                component,
                component_layout,
            ));
        } else {
            fields.push(FieldPlan::hidden(shared));
        }
    }

    let has_fields_needing_required = fields.iter().any(|field| {
        field.needs_required_validation()
            && !field
                .validation()
                .map(|validation| validation.is_newtype)
                .unwrap_or(false)
    });
    let has_any_koruma_validations = fields.iter().any(|field| {
        let Some(validation) = field.validation() else {
            return false;
        };
        !validation.field_validators.is_empty()
            || !validation.element_validators.is_empty()
            || validation.is_nested
    });
    let effective_enable_koruma =
        enable_koruma || (has_fields_needing_required && has_any_koruma_validations);
    let has_skipped_fields = fields.iter().any(FieldPlan::is_skipped);

    Ok(FormPlan {
        fields,
        has_skipped_fields,
        enable_koruma_fluent,
        effective_enable_koruma,
    })
}

fn parse_koruma_fields(
    derive_input: &DeriveInput,
) -> syn::Result<HashMap<String, KorumaFieldInfo>> {
    let syn::Data::Struct(data_struct) = &derive_input.data else {
        return Ok(HashMap::new());
    };

    let mut fields = HashMap::new();
    let mut errors: Option<syn::Error> = None;
    for field in &data_struct.fields {
        let Some(ident) = field.ident.as_ref() else {
            continue;
        };
        match koruma_derive_core::parse_field(field, 0) {
            ParseFieldResult::Valid(info) => {
                fields.insert(ident.to_string(), *info);
            },
            ParseFieldResult::Skip => {},
            ParseFieldResult::Error(error) => {
                if let Some(errors) = &mut errors {
                    errors.combine(error);
                } else {
                    errors = Some(error);
                }
            },
        }
    }

    if let Some(errors) = errors {
        Err(errors)
    } else {
        Ok(fields)
    }
}
