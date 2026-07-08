//! Facade crate for deriving and using generated `gpui-form` forms.

#[cfg(feature = "derive")]
pub use gpui_form_derive::GpuiForm;
#[cfg(all(feature = "derive", feature = "mcp"))]
pub use gpui_form_derive::mcp_submit;

pub use gpui_form_core as core;
#[cfg(feature = "mcp")]
pub use gpui_form_mcp as mcp;
#[cfg(feature = "runtime")]
pub use gpui_form_runtime as runtime;
pub use gpui_form_schema as schema;

pub use bon;
