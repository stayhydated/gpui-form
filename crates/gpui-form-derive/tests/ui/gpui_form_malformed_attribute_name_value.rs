use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct Demo {
    #[gpui_form = "gpui_form_collection::input::Input::<_>"]
    name: String,
}

fn main() {}
