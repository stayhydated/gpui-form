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

/// Derive macro for component shape metadata.
///
/// By default it calls `Self::new(window, cx)`. Override the constructor with:
/// `#[gpui_form_shape(new = path::to::constructor)]`, a closure, or a direct
/// constructor expression such as `Self::new(window, cx).with_options()`.
///
/// Supports `new = ...`, `component = ...`, `value_binding`,
/// `field_suffix = ...`, and `shape_crate = ...`.
#[proc_macro_derive(ComponentShape, attributes(gpui_form_shape))]
#[proc_macro_error]
pub fn derive_component_shape(input: TokenStream) -> TokenStream {
    derives::component_shape_state::from(input)
}

/// Function-like macro for declaring a local component shape around
/// external component/state types.
///
/// This is the external-item counterpart to `#[derive(ComponentShape)]`: the
/// generated shape type is local to the caller, so it can implement
/// `gpui_form_component::shape::ComponentShape` even when the UI
/// component and state live in another crate. If `new` is omitted, the
/// generated implementation calls `<State>::new(window, cx)`.
#[proc_macro]
#[proc_macro_error]
pub fn component_shape(input: TokenStream) -> TokenStream {
    derives::component_shape::function(input)
}
