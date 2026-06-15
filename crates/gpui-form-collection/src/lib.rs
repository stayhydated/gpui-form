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

#[cfg(test)]
mod tests {
    use super::{
        checkbox::Checkbox,
        color_picker::ColorPicker,
        date_picker::{DatePicker, DateRangePicker},
        input::Input,
        number_input::NumberInput,
        otp_input::OtpInput,
        slider::Slider,
        switch::Switch,
    };
    use chrono::NaiveDate;
    use component_shape::{ComponentShapeFor, DeclaredComponentShape};
    use gpui::{EventEmitter, Hsla};
    use gpui_component::slider::SliderValue;
    use gpui_form_runtime::shape::{
        GpuiComponentEventOf, GpuiComponentShape, GpuiComponentShapeFor, GpuiComponentStateOf,
        GpuiComponentValueBinding, GpuiFormComponentShapePolicy,
    };

    fn assert_declared_shape<Shape>()
    where
        Shape: DeclaredComponentShape + GpuiComponentShape + GpuiFormComponentShapePolicy,
    {
    }

    fn assert_shape_for<Shape, Value>()
    where
        Shape: ComponentShapeFor<Value> + GpuiComponentShapeFor<Value>,
    {
    }

    fn assert_value_binding<Shape, Value>()
    where
        Shape: GpuiComponentValueBinding<Value>,
        GpuiComponentStateOf<Shape>: EventEmitter<GpuiComponentEventOf<Shape, Value>>,
    {
    }

    #[test]
    fn built_in_components_publish_component_shape_contracts() {
        assert_declared_shape::<Checkbox>();
        assert_shape_for::<Checkbox, bool>();
        assert_value_binding::<Checkbox, bool>();

        assert_declared_shape::<Switch>();
        assert_shape_for::<Switch, bool>();
        assert_value_binding::<Switch, bool>();

        assert_declared_shape::<ColorPicker>();
        assert_shape_for::<ColorPicker, Hsla>();
        assert_value_binding::<ColorPicker, Hsla>();

        assert_declared_shape::<DatePicker>();
        assert_shape_for::<DatePicker, NaiveDate>();
        assert_value_binding::<DatePicker, NaiveDate>();

        assert_declared_shape::<DateRangePicker>();
        assert_shape_for::<DateRangePicker, (NaiveDate, NaiveDate)>();
        assert_value_binding::<DateRangePicker, (NaiveDate, NaiveDate)>();

        assert_declared_shape::<Input<String>>();
        assert_shape_for::<Input<String>, String>();
        assert_value_binding::<Input<String>, String>();
        assert_shape_for::<Input<u32>, u32>();

        assert_declared_shape::<NumberInput<u32>>();
        assert_shape_for::<NumberInput<u32>, u32>();
        assert_value_binding::<NumberInput<u32>, u32>();

        assert_declared_shape::<OtpInput<String>>();
        assert_shape_for::<OtpInput<String>, String>();
        assert_value_binding::<OtpInput<String>, String>();

        assert_declared_shape::<Slider>();
        assert_shape_for::<Slider, SliderValue>();
        assert_shape_for::<Slider, f32>();
        assert_value_binding::<Slider, SliderValue>();
        assert_value_binding::<Slider, f32>();
    }
}
