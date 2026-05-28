#[derive(gpui_form::GpuiForm)]
struct GenericForm<T>
where
    T: std::fmt::Debug + Clone + Default + std::str::FromStr + ToString + 'static,
    gpui_form_runtime::shape::AllowMissingValue:
        gpui_form_runtime::shape::ValueHolderStorage<T>,
    <gpui_form_runtime::shape::AllowMissingValue as gpui_form_runtime::shape::ValueHolderStorage<
        T,
    >>::Storage: std::fmt::Debug + Clone + Default,
{
    #[gpui_form(gpui_form_collection::input::Input::<_>)]
    value: T,
}

fn main() {}
