#[derive(gpui_form::GpuiForm)]
#[gpui_form(no_inventory)]
struct PlainGeneric<T = String>
where
    T: Default,
{
    #[gpui_form(hidden)]
    value: T,
}

fn accepts_defaulted_holder(_: PlainGenericFormValueHolder) {}

fn accepts_defaulted_fields(_: PlainGenericFormFields) {}

fn accepts_defaulted_components(_: PlainGenericFormComponents) {}

fn main() {}
