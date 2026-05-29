use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form(component(crate::InputShape), component = _)]
    name: String,
}

fn main() {}
