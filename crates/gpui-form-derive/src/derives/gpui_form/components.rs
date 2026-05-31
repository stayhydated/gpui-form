use gpui_form_codegen::components::{ComponentFieldIr, ShapeOptions};

use crate::derives::gpui_form::ir::FieldAttrContext;

pub fn generate_component_field(
    field_name: &syn::Ident,
    form_type: &syn::Type,
    component: &ShapeOptions,
    _context: &FieldAttrContext,
) -> syn::Result<ComponentFieldIr> {
    component.field_ir(field_name.to_string(), form_type.clone(), None)
}
