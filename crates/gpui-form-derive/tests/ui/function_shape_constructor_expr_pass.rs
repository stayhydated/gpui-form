use gpui_form_derive::component_shape;

pub struct EmailInput;

pub struct ExternalState;

impl EmailInput {
    fn new(_entity: &gpui::Entity<ExternalState>) -> Self {
        Self
    }
}

impl ExternalState {
    fn new(_window: &mut gpui::Window, _cx: &mut gpui::Context<'_, Self>) -> Self {
        Self
    }

    fn with_label(self, _label: &'static str) -> Self {
        self
    }
}

component_shape! {
    pub struct DefaultExternalShape {
        type State = ExternalState;
        component = EmailInput;
    }
}

component_shape! {
    pub struct ConfiguredExternalShape {
        type State = ExternalState;
        new = ExternalState::new(window, cx).with_label("email");
        component = EmailInput;
    }
}

component_shape! {
    pub struct SuffixedExternalShape {
        type State = ExternalState;
        component = EmailInput;
        field_suffix = "email";
    }
}

fn main() {}
