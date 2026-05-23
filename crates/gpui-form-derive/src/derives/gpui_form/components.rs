use crate::derives::gpui_form::structs::ComponentField;
use crate::derives::gpui_form::structs::ComponentFieldContent;
use crate::derives::gpui_form::utils::extract_option_inner_type;

fn extract_default_expr(field: &ComponentField) -> Option<syn::Expr> {
    field.default.as_ref().map(|expr| expr.0.clone())
}

pub fn generate_component_field(field: &ComponentField) -> ComponentFieldContent {
    let field_name = field.ident.as_ref().unwrap().to_string();
    let field_type = field.r#type.as_ref().map(|ty| &ty.0).unwrap_or(&field.ty);
    let field_type = extract_option_inner_type(field_type).1;

    let Some(component_def) = field.component.as_ref() else {
        return ComponentFieldContent {
            field_structure_tokens: proc_macro2::TokenStream::new(),
            field_base_declarations_tokens: proc_macro2::TokenStream::new(),
            requires_value: (field_name, false),
        };
    };

    let layout = component_def.generate_field_layout(
        field_name.clone(),
        field_type,
        extract_default_expr(field),
    );

    ComponentFieldContent {
        field_structure_tokens: layout.field_structure_tokens,
        field_base_declarations_tokens: layout.field_base_declarations_tokens,
        requires_value: (field_name, layout.requires_value),
    }
}
