mod select_item;

use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;

#[proc_macro_derive(SelectItem, attributes(select_item))]
#[proc_macro_error]
pub fn derive_select_item(input: TokenStream) -> TokenStream {
    select_item::from(input)
}
