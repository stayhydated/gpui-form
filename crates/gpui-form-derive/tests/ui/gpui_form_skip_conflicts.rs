use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct SkipWithComponent {
    #[gpui_form(skip, component(shape = crate::InputShape))]
    field: String,
}

#[derive(GpuiForm)]
struct ComponentWithSkip {
    #[gpui_form(component(shape = crate::InputShape), skip)]
    field: String,
}

#[derive(GpuiForm)]
struct SkipWithDefault {
    #[gpui_form(skip, default = "value")]
    field: String,
}

#[derive(GpuiForm)]
struct SkipWithConversion {
    #[gpui_form(skip, type = usize, source_to_form = |value| value.len(), form_to_source = |value| value.to_string())]
    field: String,
}

#[derive(GpuiForm)]
struct HiddenWithComponent {
    #[gpui_form(hidden, component(shape = crate::InputShape))]
    field: String,
}

#[derive(GpuiForm)]
struct ComponentWithHidden {
    #[gpui_form(component(shape = crate::InputShape), hidden)]
    field: String,
}

#[derive(GpuiForm)]
struct SkipWithHidden {
    #[gpui_form(skip, hidden)]
    field: String,
}

fn main() {}
