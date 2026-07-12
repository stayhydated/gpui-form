//! Proc macros for `gpui-form-collection`.

mod select_item;

use proc_macro::TokenStream;

/// Derives collection select-item metadata for enum-like option types.
#[proc_macro_derive(SelectItem, attributes(select_item))]
pub fn derive_select_item(input: TokenStream) -> TokenStream {
    select_item::from(input)
}
