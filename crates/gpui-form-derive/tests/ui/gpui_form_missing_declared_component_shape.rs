use gpui_form_derive::GpuiForm;
use gpui_form_runtime::shape::{
    ComponentShapeFor, ComponentShapeMetadata, GpuiComponentShape, GpuiComponentShapeFor,
    GpuiFormComponentShapePolicy, NoGpuiRenderComponent, RequiredValueStorage,
};

struct State;

impl State {
    fn new(_window: &mut gpui_kit::Window, _cx: &mut gpui_kit::Context<'_, Self>) -> Self {
        Self
    }
}

struct ManualShape;

impl ComponentShapeMetadata for ManualShape {}

impl GpuiComponentShape for ManualShape {
    type State = State;
    type RenderComponent = NoGpuiRenderComponent;

    fn new(window: &mut gpui_kit::Window, cx: &mut gpui_kit::Context<'_, Self::State>) -> Self::State {
        State::new(window, cx)
    }
}

impl GpuiFormComponentShapePolicy for ManualShape {
    type ValueStoragePolicy = RequiredValueStorage;
}

impl<T> ComponentShapeFor<T> for ManualShape {}

impl<T> GpuiComponentShapeFor<T> for ManualShape {}

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form(component(ManualShape))]
    value: String,
}

fn main() {}
