mod derives;
#[cfg(feature = "mcp")]
mod mcp_submit;

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
            generate_mcp: cfg!(feature = "mcp"),
        },
    )
}

#[cfg(feature = "mcp")]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn mcp_submit(attr: TokenStream, item: TokenStream) -> TokenStream {
    match mcp_submit::expand(attr.into(), item.into()) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}
