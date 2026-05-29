#[derive(gpui_form::GpuiForm)]
struct PartialGenericComponent<T = String, U = usize>
where
    T: std::fmt::Debug + Clone + Default + std::str::FromStr + ToString + 'static,
    U: Default,
    gpui_form_runtime::shape::AllowMissingValue:
        gpui_form_runtime::shape::ValueHolderStorage<T>,
    <gpui_form_runtime::shape::AllowMissingValue as gpui_form_runtime::shape::ValueHolderStorage<
        T,
    >>::Storage: std::fmt::Debug + Clone + Default,
{
    #[gpui_form(gpui_form_collection::input::Input::<_>)]
    component: T,
    plain: U,
}

fn accepts_defaulted_holder(_: PartialGenericComponentFormValueHolder) {}

fn accepts_defaulted_fields(_: PartialGenericComponentFormFields) {}

fn accepts_defaulted_components(_: PartialGenericComponentFormComponents) {}

fn main() {}
