#[derive(gpui_form::GpuiForm)]
struct GenericForm<T>
where
    T: std::fmt::Debug + Clone + Default + std::str::FromStr + ToString + 'static,
    gpui_form_runtime::shape::DirectValueStorage:
        gpui_form_runtime::shape::ValueStorage<T>,
    <gpui_form_runtime::shape::DirectValueStorage as gpui_form_runtime::shape::ValueStorage<
        T,
    >>::Storage: std::fmt::Debug + Clone + Default,
{
    #[gpui_form(shape = gpui_form_collection::input::Input::<_>)]
    value: T,
}

fn main() {}
