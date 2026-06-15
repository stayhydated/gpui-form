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

#[cfg(all(test, feature = "component-shape"))]
mod tests {
    use std::path::PathBuf;

    use component_shape::DeclaredComponentShape;
    use gpui::EventEmitter;
    use gpui_form_runtime::shape::{
        GpuiComponentEventOf, GpuiComponentShape, GpuiComponentShapeFor, GpuiComponentStateOf,
        GpuiComponentValueBinding, GpuiFormComponentShapePolicy,
    };

    use crate::{
        date_picker::{DatePicker, DateRangePicker},
        file_picker::FilePicker,
    };

    fn assert_declared_shape<Shape>()
    where
        Shape: DeclaredComponentShape + GpuiComponentShape + GpuiFormComponentShapePolicy,
    {
    }

    fn assert_shape_for<Shape, Value>()
    where
        Shape: GpuiComponentShapeFor<Value>,
    {
    }

    fn assert_value_binding<Shape, Value>()
    where
        Shape: GpuiComponentValueBinding<Value>,
        GpuiComponentStateOf<Shape>: EventEmitter<GpuiComponentEventOf<Shape, Value>>,
    {
    }

    #[test]
    fn component_shape_feature_publishes_runtime_component_contracts() {
        assert_declared_shape::<DatePicker>();
        assert_shape_for::<DatePicker, chrono::NaiveDate>();
        assert_value_binding::<DatePicker, chrono::NaiveDate>();

        assert_declared_shape::<DateRangePicker>();
        assert_shape_for::<DateRangePicker, (chrono::NaiveDate, chrono::NaiveDate)>();
        assert_value_binding::<DateRangePicker, (chrono::NaiveDate, chrono::NaiveDate)>();

        assert_declared_shape::<FilePicker>();
        assert_shape_for::<FilePicker, Vec<PathBuf>>();
        assert_value_binding::<FilePicker, Vec<PathBuf>>();
    }
}
