use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct MissingIntent {
    value: String,
}

#[derive(GpuiForm)]
struct DefaultWithoutIntent {
    #[gpui_form(default = String::new())]
    value: String,
}

fn main() {}
