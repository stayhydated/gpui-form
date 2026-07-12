use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form(unknown = "value")]
    name: String,
}

fn main() {}
