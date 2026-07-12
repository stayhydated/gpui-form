use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form(skpi)]
    value: String,
}

fn main() {}
