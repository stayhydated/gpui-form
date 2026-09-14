use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form = "crate::shapes::Input::<_>"]
    name: String,
}

fn main() {}
