use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct SkipWithComponent {
    #[gpui_form(skip, crate::InputShape)]
    field: String,
}

#[derive(GpuiForm)]
struct ComponentWithSkip {
    #[gpui_form(crate::InputShape, skip)]
    field: String,
}

#[derive(GpuiForm)]
struct SkipWithDefault {
    #[gpui_form(skip, default = "value")]
    field: String,
}

#[derive(GpuiForm)]
struct SkipWithConversion {
    #[gpui_form(skip, type = usize, from = |value| value.len(), into = |value| value.to_string())]
    field: String,
}

fn main() {}
