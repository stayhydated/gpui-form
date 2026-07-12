use es_fluent::EsFluentLabel;
use gpui_form::GpuiForm;

#[derive(Clone, Debug, Default, EsFluentLabel, GpuiForm)]
#[gpui_form(empty)]
pub struct Empty;
