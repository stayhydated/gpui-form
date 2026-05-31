use gpui_form_codegen::components::{ComponentFieldIr, ShapeOptions};

pub fn generate_component_field(
    field_name: &syn::Ident,
    form_type: &syn::Type,
    component: &ShapeOptions,
    context: &crate::derives::gpui_form::structs::FieldAttrContext,
) -> syn::Result<ComponentFieldIr> {
    component
        .field_ir(field_name.to_string(), form_type.clone(), None)
        .map_err(|err| syn::Error::new(context.option_span, err.to_string()))
}
