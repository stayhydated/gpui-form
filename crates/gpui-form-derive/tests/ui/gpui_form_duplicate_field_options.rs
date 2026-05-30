use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct DuplicateType {
    #[gpui_form(component(crate::InputShape, value(type = String, type = std::string::String)))]
    name: String,
}

#[derive(GpuiForm)]
struct DuplicateFrom {
    #[gpui_form(component(
        crate::InputShape,
        value(
            type = String,
            from_source = |value| value,
            from_source = std::convert::identity,
            into_source = std::convert::identity,
        )
    ))]
    name: String,
}

#[derive(GpuiForm)]
struct DuplicateInto {
    #[gpui_form(component(
        crate::InputShape,
        value(
            type = String,
            from_source = std::convert::identity,
            into_source = |value| value,
            into_source = std::convert::identity,
        )
    ))]
    name: String,
}

#[derive(GpuiForm)]
struct DuplicateDefault {
    #[gpui_form(component(crate::InputShape, default = String::new(), default = ""))]
    name: String,
}

#[derive(GpuiForm)]
struct DuplicateSkip {
    #[gpui_form(skip, skip)]
    hidden: bool,
}

fn main() {}
