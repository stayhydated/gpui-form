//! Curated component shapes for `gpui-form`.

extern crate self as gpui_form_collection;

macro_rules! impl_form_component_shape {
    (impl<$($generics:ident),+> $shape:ty where [$($where_clause:tt)*]; $storage:ty) => {
        impl<$($generics),+> gpui_form_runtime::shape::GpuiFormComponentShapePolicy for $shape
        where
            $($where_clause)*
        {
            type ValueStoragePolicy = $storage;
        }
    };
    ($shape:ty, $storage:ty) => {
        impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for $shape {
            type ValueStoragePolicy = $storage;
        }
    };
}

pub mod checkbox;
pub mod color_picker;
pub mod combobox;
pub mod date_picker;
pub mod input;
pub mod number_input;
pub mod otp_input;
pub mod select;
pub mod slider;
pub mod switch;
