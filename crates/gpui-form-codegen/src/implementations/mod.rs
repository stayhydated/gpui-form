use proc_macro2::TokenStream;

pub mod shape;

pub trait ComponentLayout {
    fn field_tokens(
        &self,
        field_structure_tokens: &mut TokenStream,
        field_base_declarations_tokens: &mut TokenStream,
    );
}
