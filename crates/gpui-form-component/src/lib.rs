//! Runtime helpers for gpui-form components.
//!
//! This crate provides:
//! - `date_picker` for localized runtime date and date-range pickers
//! - `file_picker` for native GPUI path selection with gpui-component styling
//! - `infinite_select` for cascading selects over nested enums

#[cfg(feature = "derive")]
pub use gpui_form_component_derive::InfiniteSelect;

mod calendar;

/// Runtime helpers for the localized date picker component.
pub mod date_picker;

/// Runtime helpers for the native file picker component.
pub mod file_picker;

/// Embedded i18n adapter used by runtime components and generated code.
pub mod i18n;

/// Runtime helpers for the InfiniteSelect component.
pub mod infinite_select;
