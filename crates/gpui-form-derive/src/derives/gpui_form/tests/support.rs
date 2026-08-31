pub(super) use super::super::*;
pub(super) use crate::derives::gpui_form::cfg_attr::flatten_cfg_attr_in_derive_input;
pub(super) use crate::derives::gpui_form::koruma;
pub(super) use koruma_derive_core::{ParsedDataField, ValidatorAttr};
pub(super) use quote::quote;
pub(super) use syn::DeriveInput;

pub(super) fn compact_tokens(tokens: &str) -> String {
    tokens.chars().filter(|c| !c.is_whitespace()).collect()
}

pub(super) fn parse_field_after_cfg_attr_flattening(
    derive_input: DeriveInput,
    index: usize,
) -> syn::Result<ParsedDataField> {
    let derive_input = flatten_cfg_attr_in_derive_input(derive_input);
    let syn::Data::Struct(data_struct) = &derive_input.data else {
        panic!("Expected struct data");
    };
    let field = data_struct.fields.iter().nth(index).unwrap();

    koruma_derive_core::parse_field(field, index)
}
