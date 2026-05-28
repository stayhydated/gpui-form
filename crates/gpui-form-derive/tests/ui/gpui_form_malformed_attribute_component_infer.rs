use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form(crate::InputShape.component(_))]
    name: String,
}

fn main() {}
