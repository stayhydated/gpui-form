use es_fluent::EsFluent;
use garde::Validate;
use gpui_form::{DropdownItem, GpuiForm};
use rust_decimal::Decimal;
use strum::EnumIter;

#[derive(Clone, Debug, Default, DropdownItem, EnumIter, EsFluent, PartialEq)]
#[fluent(display = "std")]
pub enum PreferedLanguage {
    #[default]
    English,
    French,
    Chinese,
}

#[derive(Clone, Debug, Default, DropdownItem, EnumIter, EsFluent, PartialEq)]
#[fluent(display = "std")]
pub enum EnumCountry {
    #[default]
    UnitedStates,
    France,
    China,
}

#[derive(Clone, Debug, Default, EsFluent, GpuiForm, Validate)]
#[fluent(display = "std")]
#[fluent(keys = ["Description", "Label"])]
pub struct User {
    #[gpui_form(component(input))]
    #[garde(length(min = 3, max = 50))]
    pub username: Option<String>,

    #[gpui_form(component(input))]
    #[garde(email)]
    pub email: String,

    #[gpui_form(component(number_input))]
    #[garde(range(min = 0, max = 150))]
    pub age: Option<u32>,

    #[gpui_form(component(number_input))]
    #[garde(range(min = Decimal::ZERO))]
    pub balance: Decimal,

    #[gpui_form(component(checkbox))]
    #[garde(skip)]
    pub subscribe_newsletter: bool,

    #[gpui_form(component(switch))]
    #[garde(skip)]
    pub enable_notifications: bool,

    #[gpui_form(component(dropdown(default)))]
    #[garde(skip)]
    pub preferred: PreferedLanguage,

    #[gpui_form(component(dropdown(searchable, index = EnumCountry::France)))]
    #[garde(skip)]
    pub country: Option<EnumCountry>,

    #[gpui_form(component(date_picker))]
    #[garde(skip)]
    pub birth_date: Option<chrono::NaiveDate>,

    #[gpui_form(skip)]
    #[garde(skip)]
    pub skip_me: bool,
}
