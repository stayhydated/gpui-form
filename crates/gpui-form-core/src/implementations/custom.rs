use crate::components::*;
use proc_macro2::TokenStream;

impl super::ComponentLayout for CustomComponent {
    fn field_tokens(
        &self,
        _field_structure_tokens: &mut TokenStream,
        _field_base_declarations_tokens: &mut TokenStream,
    ) {
        let FieldInformation::<CustomOptions> {
            options: _,
            name: _,
            r#type: _,
        } = &self.0;
    }
}
