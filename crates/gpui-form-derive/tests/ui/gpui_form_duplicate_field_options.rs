use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct DuplicateType {
    #[gpui_form(type = String, type = std::string::String)]
    name: String,
}

#[derive(GpuiForm)]
struct DuplicateFrom {
    #[gpui_form(from = |value| value, from = std::convert::identity)]
    name: String,
}

#[derive(GpuiForm)]
struct DuplicateInto {
    #[gpui_form(into = |value| value, into = std::convert::identity)]
    name: String,
}

#[derive(GpuiForm)]
struct DuplicateDefault {
    #[gpui_form(default = String::new(), default = "")]
    name: String,
}

#[derive(GpuiForm)]
struct DuplicateSkip {
    #[gpui_form(skip, skip = true)]
    hidden: bool,
}

fn main() {}
