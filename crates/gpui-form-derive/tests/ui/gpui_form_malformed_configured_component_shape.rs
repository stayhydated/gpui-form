use gpui_form::GpuiForm;

#[derive(GpuiForm)]
#[gpui_form(no_inventory)]
struct InvalidConfiguredComponentShapeForm {
    #[gpui_form(component(Select::<_>::searchable(true)))]
    country: String,
}

fn main() {}
