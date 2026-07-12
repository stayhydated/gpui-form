#![cfg(feature = "runtime")]

use gpui_form::runtime::shape::{
    ComponentShapeMetadata, DeclaredGpuiComponentShape, GpuiComponentShapeFor,
    GpuiFormComponentShapePolicy, McpInputShape, McpPrimitiveKind,
};

fn assert_form_shape<Shape, Value>()
where
    Shape: ComponentShapeMetadata
        + DeclaredGpuiComponentShape
        + GpuiComponentShapeFor<Value>
        + GpuiFormComponentShapePolicy,
{
}

#[test]
fn facade_runtime_accepts_curated_collection_shapes() {
    assert_form_shape::<gpui_form_collection::checkbox::Checkbox, bool>();

    assert_eq!(
        <gpui_form_collection::checkbox::Checkbox as ComponentShapeMetadata>::MCP_INPUT
            .input_shape(),
        McpInputShape::Scalar(McpPrimitiveKind::Boolean)
    );
}

#[test]
fn component_package_feature_publishes_shape_metadata() {
    assert_form_shape::<gpui_form_component::date_picker::DatePicker, chrono::NaiveDate>();

    assert_eq!(
        <gpui_form_component::date_picker::DatePicker as ComponentShapeMetadata>::MCP_INPUT
            .input_shape(),
        McpInputShape::Scalar(McpPrimitiveKind::Date)
    );
}
