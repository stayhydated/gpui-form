use gpui_form_core::components::ComponentsBehaviour;
use gpui_form_core::registry::GpuiFormShape;
use proc_macro2::TokenStream;
use quote::quote;

use super::implementations::{
    ComponentIdentities, ComponentShape, FieldCodeGenerator, checkbox::CheckboxCodeGenerator,
    date_picker::DatePickerCodeGenerator, dropdown::DropdownCodeGenerator,
    input::InputCodeGenerator, number_input::NumberInputCodeGenerator, switch::SwitchCodeGenerator,
};

macro_rules! field_generator {
    ($behaviour:expr) => {{
        match $behaviour {
            ComponentsBehaviour::Input => Box::new(InputCodeGenerator),
            ComponentsBehaviour::NumberInput => Box::new(NumberInputCodeGenerator),
            ComponentsBehaviour::Checkbox => Box::new(CheckboxCodeGenerator),
            ComponentsBehaviour::Switch => Box::new(SwitchCodeGenerator),
            ComponentsBehaviour::Dropdown(_) => Box::new(DropdownCodeGenerator),
            ComponentsBehaviour::DatePicker => Box::new(DatePickerCodeGenerator),
        }
    }};
}

pub struct ShapeIdentities<'a>(&'a GpuiFormShape);

impl<'a> ShapeIdentities<'a> {
    pub fn new(shape_data: &'a GpuiFormShape) -> Self {
        Self(shape_data)
    }
}

impl<'a> ComponentIdentities for ShapeIdentities<'a> {
    fn struct_name(&self) -> &'static str {
        self.0.struct_name
    }
}

pub struct FormShapeAdapter<'a> {
    pub shape_data: &'a GpuiFormShape,
    pub identities: ShapeIdentities<'a>,
}

impl<'a> FormShapeAdapter<'a> {
    pub fn new(shape_data: &'a GpuiFormShape) -> Self {
        Self {
            shape_data,
            identities: ShapeIdentities::new(shape_data),
        }
    }
}

impl<'a> ComponentShape for FormShapeAdapter<'a> {
    fn cx_new_calls(&self) -> Option<TokenStream> {
        let x: proc_macro2::TokenStream = self
            .shape_data
            .components
            .iter()
            .filter_map(|field| {
                let generator: Box<dyn FieldCodeGenerator> = field_generator!(field.behaviour);
                generator.generate_cx_new_call(field, &self.identities)
            })
            .collect();

        if x.is_empty() { None } else { Some(x) }
    }

    fn field_initializers(&self) -> Option<TokenStream> {
        let x: proc_macro2::TokenStream = self
            .shape_data
            .components
            .iter()
            .filter_map(|field| {
                let generator: Box<dyn FieldCodeGenerator> = field_generator!(field.behaviour);
                generator.generate_field_initializers(field, &self.identities)
            })
            .collect();

        if x.is_empty() { None } else { Some(x) }
    }

    fn child_elements(&self) -> TokenStream {
        self.shape_data
            .components
            .iter()
            .map(|field| {
                let generator: Box<dyn FieldCodeGenerator> = field_generator!(field.behaviour);
                generator.generate_render_child(field, &self.identities)
            })
            .collect()
    }

    fn focusable_cycle(&self) -> Option<proc_macro2::TokenStream> {
        let x: proc_macro2::TokenStream = self
            .shape_data
            .components
            .iter()
            .filter_map(|field| {
                let generator: Box<dyn FieldCodeGenerator> = field_generator!(field.behaviour);
                generator.generate_focusable_cycle(field, &self.identities)
            })
            .collect();

        if x.is_empty() { None } else { Some(x) }
    }

    fn subscription_calls(&self) -> Option<proc_macro2::TokenStream> {
        let calls: Vec<TokenStream> = self
            .shape_data
            .components
            .iter()
            .filter_map(|field| {
                let generator: Box<dyn FieldCodeGenerator> = field_generator!(field.behaviour);
                generator.generate_subscription(field, &self.identities)
            })
            .flat_map(|sub| sub.calls)
            .collect();

        if calls.is_empty() {
            None
        } else {
            Some(quote! {
                let _subscriptions = vec![#(#calls),*];
            })
        }
    }

    fn event_handlers(&self) -> Option<proc_macro2::TokenStream> {
        let handlers: Vec<TokenStream> = self
            .shape_data
            .components
            .iter()
            .filter_map(|field| {
                let generator: Box<dyn FieldCodeGenerator> = field_generator!(field.behaviour);
                generator.generate_subscription(field, &self.identities)
            })
            .flat_map(|sub| sub.handlers)
            .collect();

        if handlers.is_empty() {
            None
        } else {
            Some(quote! {
                #(#handlers)*
            })
        }
    }
}
