use crate::derives::gpui_form::structs::ComponentFieldContent;
use crate::derives::gpui_form::structs::ComponentRenderedField;
use crate::derives::gpui_form::utils::extract_option_inner_type;

pub fn generate_component_field(
    field_name: &syn::Ident,
    field_type: &syn::Type,
    rendered: &ComponentRenderedField<'_>,
) -> ComponentFieldContent {
    let field_type = rendered.r#type.map(|ty| &ty.value.0).unwrap_or(field_type);
    let field_type = extract_option_inner_type(field_type).1;
    let default_expr = rendered.default.map(|expr| expr.value.0.clone());

    let layout =
        rendered
            .component
            .generate_field_layout(field_name.to_string(), field_type, default_expr);

    ComponentFieldContent {
        field_structure_tokens: layout.field_structure_tokens,
        field_base_declarations_tokens: layout.field_base_declarations_tokens,
        required_value: layout.required_value,
    }
}
