use std::marker::PhantomData;

struct InputState<T>(PhantomData<T>);

impl<T> InputState<T> {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self(PhantomData)
    }
}

gpui_form_derive::component_shape! {
    struct Input<T>
    where
        T: 'static,
    {
        type State = InputState<T>;
        value = T;
        value_storage = direct;
    }
}

#[derive(gpui_form::GpuiForm)]
#[gpui_form(no_inventory)]
struct GenericForm<T>
where
    T: std::fmt::Debug + Clone + Default + std::str::FromStr + ToString + 'static,
    gpui_form_runtime::shape::DirectValueStorage:
        gpui_form_runtime::shape::ValueStorage<T>,
    <gpui_form_runtime::shape::DirectValueStorage as gpui_form_runtime::shape::ValueStorage<
        T,
    >>::Storage: std::fmt::Debug + Clone + Default,
{
    #[gpui_form(component(Input::<_>))]
    value: T,
}

fn main() {}
