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

struct FacadeShapeState;

impl FacadeShapeState {
    fn new(_window: &mut gpui_kit::Window, _cx: &mut gpui_kit::Context<'_, Self>) -> Self {
        Self
    }
}

component_shape_gpui::component_shape! {
    struct FacadeBooleanShape {
        state = FacadeShapeState;
        value = bool;
    }
}

impl gpui_form::runtime::shape::GpuiFormComponentShapePolicy for FacadeBooleanShape {
    type ValueStoragePolicy = gpui_form::runtime::shape::DirectValueStorage;
}

component_shape_gpui::component_shape! {
    struct FacadeDateShape {
        state = FacadeShapeState;
        value = chrono::NaiveDate;
    }
}

impl gpui_form::runtime::shape::GpuiFormComponentShapePolicy for FacadeDateShape {
    type ValueStoragePolicy = gpui_form::runtime::shape::DirectValueStorage;
}

#[test]
fn facade_runtime_accepts_declared_boolean_shape() {
    assert_form_shape::<FacadeBooleanShape, bool>();

    assert_eq!(
        <FacadeBooleanShape as ComponentShapeMetadata>::MCP_INPUT.input_shape(),
        McpInputShape::Scalar(McpPrimitiveKind::Boolean)
    );
}

#[test]
fn facade_runtime_accepts_declared_date_shape() {
    assert_form_shape::<FacadeDateShape, chrono::NaiveDate>();

    assert_eq!(
        <FacadeDateShape as ComponentShapeMetadata>::MCP_INPUT.input_shape(),
        McpInputShape::Scalar(McpPrimitiveKind::Date)
    );
}
