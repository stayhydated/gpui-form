use crate::derives::gpui_form::structs::ComponentField;
use crate::derives::gpui_form::structs::ComponentFieldContent;
use crate::derives::gpui_form::utils::extract_option_inner_type;

fn extract_default_expr(field: &ComponentField) -> Option<syn::Expr> {
    field
        .rendered()
        .and_then(|rendered| rendered.default.map(|expr| expr.value.0.clone()))
}

pub fn generate_component_field(field: &ComponentField) -> ComponentFieldContent {
    let Some(rendered) = field.component() else {
        unreachable!("generate_component_field should only be called for component fields");
    };
    let field_type = rendered.r#type.map(|ty| &ty.value.0).unwrap_or(&field.ty);
    let field_type = extract_option_inner_type(field_type).1;

    let layout = rendered.component.generate_field_layout(
        field.ident.to_string(),
        field_type,
        extract_default_expr(field),
    );

    ComponentFieldContent {
        field_structure_tokens: layout.field_structure_tokens,
        field_base_declarations_tokens: layout.field_base_declarations_tokens,
        required_value: layout.required_value,
    }
}
