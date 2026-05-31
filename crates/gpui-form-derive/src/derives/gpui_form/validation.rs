use darling::{Error as DarlingError, FromMeta};
use koruma_derive_core::{FieldInfo as KorumaFieldInfo, ParseFieldResult};
use std::collections::HashMap;
use syn::DeriveInput;

use crate::derives::gpui_form::ir::{ValidationMetadata, ValidationRule};

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

pub fn parse_koruma_fields(
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

pub fn validation_metadata_from_koruma(info: &KorumaFieldInfo) -> ValidationMetadata {
    let mut metadata = ValidationMetadata::default();
    for validator in &info.validation.field_validators {
        if validator.name() == "RequiredValidation" {
            metadata.push(ValidationRule::Required);
        } else {
            metadata.push(ValidationRule::Validator(validator.name().to_string()));
        }
    }
    if info.is_newtype() {
        metadata.push(ValidationRule::Newtype);
    }
    if info.is_nested() {
        metadata.push(ValidationRule::Nested);
    }

    metadata
}
