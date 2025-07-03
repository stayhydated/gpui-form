use proc_macro2::TokenStream;

pub mod checkbox;
pub mod custom;
pub mod date_picker;
pub mod dropdown;
pub mod input;
pub mod number_input;
pub mod switch;

pub trait ComponentLayout {
    fn field_tokens(
        &self,
        field_structure_tokens: &mut TokenStream,
        field_base_declarations_tokens: &mut TokenStream,
    );
}

mod __crate_paths;
