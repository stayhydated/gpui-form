use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct TopLevelType {
    #[gpui_form(type = String)]
    name: String,
}

#[derive(GpuiForm)]
struct TopLevelSourceToForm {
    #[gpui_form(source_to_form = std::convert::identity)]
    name: String,
}

#[derive(GpuiForm)]
struct TopLevelFormToSource {
    #[gpui_form(form_to_source = std::convert::identity)]
    name: String,
}

#[derive(GpuiForm)]
struct TopLevelDefault {
    #[gpui_form(default = String::new())]
    name: String,
}

fn main() {}
