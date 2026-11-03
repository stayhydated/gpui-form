//! Runtime helpers for gpui-form components.
//!
//! This crate provides:
//! - `file_picker` for native GPUI path selection with GPUI Kit styling
//! - `infinite_select` for cascading selects over nested enums

#[cfg(feature = "derive")]
pub use gpui_form_example_component_derive::InfiniteSelect;

/// Runtime helpers for the native file picker component.
pub mod file_picker;

/// Embedded i18n adapter used by runtime components and generated code.
pub mod i18n;

/// Runtime helpers for the InfiniteSelect component.
pub mod infinite_select;

/// Implementation details used by `#[derive(InfiniteSelect)]` expansions.
///
/// This module is public so generated code in downstream crates can resolve
/// strict Fluent labels without requiring a direct `gpui-es-fluent`
/// dependency. Its contents are not a stable application-facing API.
#[doc(hidden)]
pub mod __macro_support {
    pub use gpui_es_fluent::{localize_label, localize_message};
}
