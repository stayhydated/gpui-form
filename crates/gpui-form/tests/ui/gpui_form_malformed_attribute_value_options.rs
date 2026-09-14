use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct MissingFromSource {
    #[gpui_form(component(crate::InputShape, value(type = String, into_source = std::convert::identity)))]
    name: String,
}

#[derive(GpuiForm)]
struct MissingIntoSource {
    #[gpui_form(component(crate::InputShape, value(type = String, from_source = std::convert::identity)))]
    name: String,
}

#[derive(GpuiForm)]
struct FromSourceWithoutType {
    #[gpui_form(component(crate::InputShape, value(from_source = std::convert::identity)))]
    name: String,
}

#[derive(GpuiForm)]
struct IntoSourceWithoutType {
    #[gpui_form(component(crate::InputShape, value(into_source = std::convert::identity)))]
    name: String,
}

fn main() {}
