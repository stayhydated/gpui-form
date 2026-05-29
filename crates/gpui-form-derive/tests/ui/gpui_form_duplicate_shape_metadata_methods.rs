use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct DuplicateComponent {
    #[gpui_form(
        component(DemoShape),
        component = crate::Widget,
        component = crate::OtherWidget
    )]
    name: String,
}

#[derive(GpuiForm)]
struct DuplicateFieldSuffix {
    #[gpui_form(component(DemoShape), field_suffix = "input", field_suffix = "field")]
    name: String,
}

fn main() {}
