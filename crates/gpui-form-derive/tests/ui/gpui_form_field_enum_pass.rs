pub struct State;

impl State {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }
}

component_shape_gpui::component_shape! {
    pub struct TextShape {
        state = State;
        value = String;
    }
}

impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for TextShape {
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}

#[derive(gpui_form::GpuiForm)]
#[gpui_form(no_inventory)]
pub struct UserProfile {
    #[gpui_form(component(TextShape))]
    display_name: String,
    #[gpui_form(hidden)]
    internal_note: String,
    #[gpui_form(skip)]
    database_id: u64,
}

const DISPLAY_NAME: &str = UserProfileFormField::DisplayName.name();

fn accepts_form_field(field: impl gpui_form::core::FormField) -> &'static str {
    field.name()
}

fn main() {
    let _ = DISPLAY_NAME;
    let _ = accepts_form_field(UserProfileFormField::DisplayName);
}
