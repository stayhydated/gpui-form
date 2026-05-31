pub mod attrs;
pub mod cfg_attr;
pub mod components;
pub mod expansion;
pub mod holder_plan;
pub mod intent;
pub mod ir;
pub mod koruma;
pub mod planner;
pub mod structs;
pub mod tests;
pub mod utils;
pub mod validation;
pub mod value_holder;

use crate::derives::gpui_form::expansion::expand_gpui_form;
pub use structs::GpuiFormOptions;
use syn::DeriveInput;
use syn::parse_macro_input;

pub fn from(input: proc_macro::TokenStream, options: GpuiFormOptions) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    expand_gpui_form(derive_input, options).into()
}
