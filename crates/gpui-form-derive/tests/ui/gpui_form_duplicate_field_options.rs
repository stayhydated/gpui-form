use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct DuplicateType {
    #[gpui_form(type = String, type = std::string::String)]
    name: String,
}

#[derive(GpuiForm)]
struct DuplicateFrom {
    #[gpui_form(source_to_form = |value| value, source_to_form = std::convert::identity)]
    name: String,
}

#[derive(GpuiForm)]
struct DuplicateInto {
    #[gpui_form(form_to_source = |value| value, form_to_source = std::convert::identity)]
    name: String,
}

#[derive(GpuiForm)]
struct DuplicateDefault {
    #[gpui_form(default = String::new(), default = "")]
    name: String,
}

#[derive(GpuiForm)]
struct DuplicateSkip {
    #[gpui_form(skip, skip)]
    hidden: bool,
}

fn main() {}
