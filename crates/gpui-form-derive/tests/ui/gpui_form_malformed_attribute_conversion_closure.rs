use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
#[gpui_form(no_inventory)]
struct BadFromSourceClosure {
    #[gpui_form(hidden(value(
        type = String,
        from_source = |value: String| value,
        into_source = |value: String| value.parse().unwrap(),
    )))]
    code: u64,
}

fn main() {}
