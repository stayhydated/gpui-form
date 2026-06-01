mod derives;

use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;

use crate::derives::gpui_form::GpuiFormOptions;

#[proc_macro_derive(GpuiForm, attributes(gpui_form, koruma))]
#[proc_macro_error]
pub fn gpui_form_derive(input: TokenStream) -> TokenStream {
    derives::gpui_form::from(
        input,
        GpuiFormOptions {
            generate_shape: cfg!(feature = "inventory"),
        },
    )
}
