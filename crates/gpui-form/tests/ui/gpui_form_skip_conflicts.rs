use gpui_form_derive::GpuiForm;

#[derive(GpuiForm)]
struct SkipWithComponent {
    #[gpui_form(skip, component(crate::InputShape))]
    field: String,
}

#[derive(GpuiForm)]
struct ComponentWithSkip {
    #[gpui_form(component(crate::InputShape), skip)]
    field: String,
}

#[derive(GpuiForm)]
struct HiddenWithComponent {
    #[gpui_form(hidden, component(crate::InputShape))]
    field: String,
}

#[derive(GpuiForm)]
struct ComponentWithHidden {
    #[gpui_form(component(crate::InputShape), hidden)]
    field: String,
}

#[derive(GpuiForm)]
struct SkipWithHidden {
    #[gpui_form(skip, hidden)]
    field: String,
}

fn main() {}
