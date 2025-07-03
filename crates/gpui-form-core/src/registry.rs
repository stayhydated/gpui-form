use crate::components::ComponentsBehaviour;

inventory::collect!(GpuiFormShape);

#[derive(Debug)]
pub struct GpuiFormShape {
    pub struct_name: &'static str,
    pub components: &'static [FieldVariant],
}

impl GpuiFormShape {
    pub const fn new(struct_name: &'static str, components: &'static [FieldVariant]) -> Self {
        Self {
            struct_name,
            components,
        }
    }
}

#[derive(Debug)]
pub struct FieldVariant {
    pub field_name: &'static str,
    pub field_type: &'static str,
    pub optional: bool,
    pub behaviour: ComponentsBehaviour,
}

impl FieldVariant {
    pub const fn new(
        field_name: &'static str,
        field_type: &'static str,
        optional: bool,
        behaviour: ComponentsBehaviour,
    ) -> Self {
        Self {
            field_name,
            field_type,
            optional,
            behaviour,
        }
    }
    pub fn full_type(&self) -> syn::Type {
        let mut ty = syn::parse_str(self.field_type).unwrap();
        if self.optional {
            ty = syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::parse_str("Option").unwrap(),
            });
        }
        ty
    }
}

pub use inventory;
