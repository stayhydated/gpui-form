use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct MissingIntent {
    value: String,
}

#[derive(GpuiForm)]
struct EmptyAttribute {
    #[gpui_form()]
    value: String,
}

fn main() {}
