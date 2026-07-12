mod infinite_select;

use proc_macro::TokenStream;

/// Derive macro for infinite select enums used by `gpui-form-component`.
///
/// This macro emits an implementation of
/// `gpui_form_component::infinite_select::InfiniteSelectValue`
/// for nested enums composed of unit, single-field tuple, or single-field struct
/// variants.
#[proc_macro_derive(InfiniteSelect, attributes(tuple_enum, fluent_kv))]
pub fn derive_infinite_select(input: TokenStream) -> TokenStream {
    infinite_select::from(input)
}
