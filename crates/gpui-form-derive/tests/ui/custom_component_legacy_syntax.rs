use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form(component(custom(shape = crate::shape::DemoShape)))]
    field: String,
}

fn main() {}
