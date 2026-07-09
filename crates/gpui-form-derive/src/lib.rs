//! Proc macros for deriving `gpui-form` form state and MCP submit helpers.

mod derives;
#[cfg(feature = "mcp")]
mod mcp_submit;

use proc_macro::TokenStream;

use crate::derives::gpui_form::GpuiFormOptions;

/// Derives generated form state, metadata, value holders, and helper APIs.
#[proc_macro_derive(GpuiForm, attributes(gpui_form, koruma))]
pub fn gpui_form_derive(input: TokenStream) -> TokenStream {
    derives::gpui_form::from(
        input,
        GpuiFormOptions {
            generate_shape: cfg!(feature = "inventory"),
            generate_mcp: cfg!(feature = "mcp"),
        },
    )
}

/// Registers an async function as an MCP submit handler for generated forms.
#[cfg(feature = "mcp")]
#[proc_macro_attribute]
pub fn mcp_submit(attr: TokenStream, item: TokenStream) -> TokenStream {
    match mcp_submit::expand(attr.into(), item.into()) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}
