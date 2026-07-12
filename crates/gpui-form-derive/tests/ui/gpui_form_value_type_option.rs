use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct OptionTypeOverride {
    #[gpui_form(hidden(value(
        type = Option<String>,
        from_source = std::convert::identity,
        into_source = std::convert::identity
    )))]
    name: Option<String>,
}

fn main() {}
