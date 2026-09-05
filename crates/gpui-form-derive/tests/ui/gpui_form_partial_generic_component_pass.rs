use std::marker::PhantomData;

struct InputState<T>(PhantomData<T>);

impl<T> InputState<T> {
    fn new(_window: &mut gpui_kit::Window, _cx: &mut gpui_kit::Context<'_, Self>) -> Self {
        Self(PhantomData)
    }
}

component_shape_gpui::component_shape! {
    struct Input<T>
    where
        T: 'static,
    {
        state = InputState<T>;
        value = T;
    }
}

impl<T> gpui_form_runtime::shape::GpuiFormComponentShapePolicy for Input<T>
where
    T: 'static,
{
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}

#[derive(gpui_form::GpuiForm)]
#[gpui_form(no_inventory)]
struct PartialGenericComponent<T = String, U = usize>
where
    T: std::fmt::Debug + Clone + Default + std::str::FromStr + ToString + 'static,
    U: Default,
    gpui_form_runtime::shape::DirectValueStorage:
        gpui_form_runtime::shape::ValueStorage<T>,
    <gpui_form_runtime::shape::DirectValueStorage as gpui_form_runtime::shape::ValueStorage<
        T,
    >>::Storage: std::fmt::Debug + Clone + Default,
{
    #[gpui_form(component(Input::<_>))]
    component: T,
    #[gpui_form(hidden)]
    plain: U,
}

fn accepts_defaulted_holder(_: PartialGenericComponentFormValueHolder) {}

fn accepts_defaulted_fields(_: PartialGenericComponentFormFields) {}

fn accepts_defaulted_components(_: PartialGenericComponentFormComponents) {}

fn main() {}
