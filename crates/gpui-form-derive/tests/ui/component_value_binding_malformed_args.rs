use gpui_form_derive::component_value_binding;

struct State;

#[component_value_binding]
impl gpui_form_runtime::shape::ComponentValueBinding<> for State {}

#[component_value_binding]
impl gpui_form_runtime::shape::ComponentValueBinding<String, usize> for State {}

#[component_value_binding]
impl gpui_form_runtime::shape::ComponentValueBinding<'static> for State {}

#[component_value_binding]
impl gpui_form_runtime::shape::ComponentValueBinding<1> for State {}

#[component_value_binding]
impl gpui_form_runtime::shape::ComponentValueBinding<T = String> for State {}

fn main() {}
