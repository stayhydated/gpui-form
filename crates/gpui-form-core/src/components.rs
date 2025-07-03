use darling::FromMeta;
use gpui_form_internal_macros::{ComponentDefinitions, ComponentOption};
use quote::quote;
use strum::{Display, EnumDiscriminants, EnumString, IntoStaticStr};

fn default_true() -> bool {
    true
}

pub trait ComponentOption {}

pub trait ComponentDefinition {
    fn component_name() -> &'static str;
}

pub struct FieldInformation<T: ComponentOption> {
    pub options: T,
    pub name: String,
    pub r#type: syn::Ident,
}

impl<T: ComponentOption> FieldInformation<T> {
    pub fn new(options: T, name: String, r#type: syn::Ident) -> Self {
        Self {
            options,
            name,
            r#type,
        }
    }
}

#[derive(Clone, ComponentOption, Debug, Default, Eq, FromMeta, PartialEq)]
pub struct BehaviourDropdownOptions {
    #[darling(default)]
    pub partial: bool,
    #[darling(default)]
    pub searchable: bool,
}

#[derive(Clone, ComponentOption, Debug, Eq, FromMeta, PartialEq)]
pub struct BehaviourCustomOptions {
    #[darling(default = "default_true", rename = "uw")]
    pub should_be_unwrapped: bool,
    #[darling(default)]
    pub partial: bool,
    pub name: syn::Ident,
}

#[derive(Clone, ComponentOption, Debug, FromMeta)]
pub struct CustomOptions {
    #[darling(flatten)]
    pub behaviour: BehaviourCustomOptions,
}

#[derive(Clone, ComponentOption, Debug, FromMeta)]
pub struct DropdownOptions {
    #[darling(flatten)]
    pub behaviour: BehaviourDropdownOptions,
    #[darling(default, rename = "index")]
    named_index: Option<syn::Path>,
    #[darling(default, rename = "default")]
    index_default: bool,
}

impl DropdownOptions {
    pub fn named_index(&self) -> Option<&syn::Path> {
        if self.named_index.is_some() && self.index_default {
            panic!("Cannot specify both named_index and index_default");
        }
        self.named_index.as_ref()
    }

    pub fn index_default(&self) -> bool {
        if self.named_index.is_some() && self.index_default {
            panic!("Cannot specify both named_index and index_default");
        }
        self.index_default
    }
}

#[derive(Clone, ComponentOption, Debug, FromMeta)]
pub struct InputOptions;
#[derive(Clone, ComponentOption, Debug, FromMeta)]
pub struct NumberInputOptions;
#[derive(Clone, ComponentOption, Debug, FromMeta)]
pub struct CheckboxOptions;
#[derive(Clone, ComponentOption, Debug, FromMeta)]
pub struct SwitchOptions;
#[derive(Clone, ComponentOption, Debug, FromMeta)]
pub struct DatePickerOptions;

#[derive(Clone, ComponentDefinitions, Debug, EnumDiscriminants, FromMeta)]
#[strum_discriminants(derive(EnumString, Display, IntoStaticStr))]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
#[darling(rename_all = "snake_case")]
pub enum Components {
    Input,
    NumberInput,
    Checkbox,
    Switch,
    Dropdown(DropdownOptions),
    DatePicker,
    Custom(CustomOptions),
}

#[derive(Clone, Debug, Display, EnumString, Eq, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum ComponentsBehaviour {
    Input,
    NumberInput,
    Checkbox,
    Switch,
    Dropdown(BehaviourDropdownOptions),
    DatePicker,
}

impl ComponentsBehaviour {
    pub fn as_component_ident(&self) -> proc_macro2::TokenStream {
        match self {
            ComponentsBehaviour::Input => quote! { TextInput },
            ComponentsBehaviour::NumberInput => quote! { NumberInput },
            ComponentsBehaviour::Checkbox => quote! { Checkbox },
            ComponentsBehaviour::Switch => quote! { Switch },
            ComponentsBehaviour::Dropdown(_) => quote! { Dropdown },
            ComponentsBehaviour::DatePicker => quote! { DatePicker },
        }
    }

    pub fn is_value_only_field(&self) -> bool {
        matches!(
            self,
            ComponentsBehaviour::Checkbox | ComponentsBehaviour::Switch
        )
    }

    pub fn needs_value_field(&self) -> bool {
        matches!(self, ComponentsBehaviour::NumberInput)
    }

    pub fn partial(&self) -> bool {
        match self {
            ComponentsBehaviour::Dropdown(options) => options.partial,
            _ => false,
        }
    }

    pub fn subscribable(&self) -> bool {
        matches!(
            self,
            ComponentsBehaviour::Input
                | ComponentsBehaviour::NumberInput
                | ComponentsBehaviour::Dropdown(_)
        )
    }

    pub fn focusable(&self) -> bool {
        matches!(
            self,
            ComponentsBehaviour::Input
                | ComponentsBehaviour::NumberInput
                | ComponentsBehaviour::Dropdown(_)
        )
    }
}
