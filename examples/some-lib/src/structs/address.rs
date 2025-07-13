use es_fluent::EsFluent;
use gpui::{IntoElement as _, SharedString};
use gpui_component::dropdown::DropdownItem;
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default)]
pub struct Country {
    name: SharedString,
    code: SharedString,
}

impl Country {
    pub fn new(name: impl Into<SharedString>, code: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            code: code.into(),
        }
    }
}

impl DropdownItem for Country {
    type Value = Country;

    fn title(&self) -> SharedString {
        self.name.clone()
    }

    fn display_title(&self) -> Option<gpui::AnyElement> {
        Some(format!("{} ({})", self.name, self.code).into_any_element())
    }

    fn value(&self) -> &Self::Value {
        &self
    }
}

#[derive(Clone, Debug, Default, EsFluent, GpuiForm)]
#[fluent(display = "std")]
#[fluent(this, keys = ["Description", "Label"])]
pub struct Address {
    #[gpui_form(component(input))]
    pub street: Option<String>,
    #[gpui_form(component(dropdown(partial)))]
    pub dynamic_country: Country,
    #[gpui_form(skip)]
    pub skip_me: bool,
}
