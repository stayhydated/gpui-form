use gpui_form_derive::GpuiForm;

fn to_form(value: u64) -> String {
    value.to_string()
}

fn to_source(value: String) -> u64 {
    value.parse().unwrap_or_default()
}

#[derive(GpuiForm)]
#[gpui_form(no_inventory)]
struct OptionalOverride {
    #[gpui_form(hidden(value(type = String, from_source = to_form, into_source = to_source)))]
    id: Option<u64>,
}

fn main() {
    let source = OptionalOverride { id: Some(7) };
    let holder = OptionalOverrideFormValueHolder::from(source);
    let _roundtrip: OptionalOverride = holder.into_original();
}
