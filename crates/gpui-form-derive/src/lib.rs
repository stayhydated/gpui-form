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

/// Derive macro for custom component state types used by `component(custom(...))`.
///
/// By default it calls `Self::new(window, cx)`. Override the constructor with:
/// `#[gpui_form_custom(new = path::to::constructor)]`.
///
/// `#[derive(CustomComponent)]` is an alias with the same expansion and a name
/// that fits reusable custom component metadata.
#[proc_macro_derive(CustomComponentState, attributes(gpui_form_custom))]
#[proc_macro_error]
pub fn derive_custom_component_state(input: TokenStream) -> TokenStream {
    derives::custom_component_state::from(input)
}

/// Derive macro for custom component shape metadata.
///
/// Supports the same `#[gpui_form_custom(...)]` options as
/// `CustomComponentState`: `new = ...`, `component = ...`, and
/// `value_binding`.
#[proc_macro_derive(CustomComponent, attributes(gpui_form_custom))]
#[proc_macro_error]
pub fn derive_custom_component(input: TokenStream) -> TokenStream {
    derives::custom_component_state::from(input)
}

/// Function-like macro for declaring a local custom component shape around
/// external component/state types.
///
/// This is the external-item counterpart to `#[derive(CustomComponent)]`: the
/// generated shape type is local to the caller, so it can implement
/// `gpui_form_component::custom::CustomComponentShape` even when the UI
/// component and state live in another crate.
#[proc_macro]
#[proc_macro_error]
pub fn custom_component(input: TokenStream) -> TokenStream {
    derives::custom_component::function(input)
}
