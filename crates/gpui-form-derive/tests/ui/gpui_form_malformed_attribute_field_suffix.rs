use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct InvalidFieldSuffix {
    #[gpui_form(shape = crate::Input, field_suffix = "bad-suffix")]
    value: String,
}

fn main() {}
