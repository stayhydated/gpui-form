pub mod checkbox;
pub mod date_picker;
pub mod dropdown;
pub mod input;
pub mod number_input;
pub mod switch;

use crate::code_gen::ShapeIdentities;

use gpui_form_core::registry::FieldVariant;
use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;

#[derive(Default)]
pub struct GeneratedSubscription {
    pub calls: Vec<TokenStream>,
    pub handlers: Vec<TokenStream>,
}

impl GeneratedSubscription {
    pub fn is_empty(&self) -> bool {
        self.calls.is_empty() && self.handlers.is_empty()
    }
}

pub trait FieldCodeGenerator {
    fn generate_cx_new_call(
        &self,
        field: &FieldVariant,
        component: &ShapeIdentities,
    ) -> Option<TokenStream>;

    fn generate_field_initializers(
        &self,
        field: &FieldVariant,
        component: &ShapeIdentities,
    ) -> Option<TokenStream>;

    fn generate_render_child(
        &self,
        field: &FieldVariant,
        component: &ShapeIdentities,
    ) -> TokenStream;

    fn generate_focusable_cycle(
        &self,
        field: &FieldVariant,
        component: &ShapeIdentities,
    ) -> Option<TokenStream>;

    fn generate_subscription(
        &self,
        field: &FieldVariant,
        component: &ShapeIdentities,
    ) -> Option<GeneratedSubscription>;
}

pub trait ComponentShape {
    fn cx_new_calls(&self) -> Option<TokenStream>;

    fn field_initializers(&self) -> Option<TokenStream>;

    fn child_elements(&self) -> TokenStream;

    fn focusable_cycle(&self) -> Option<TokenStream>;

    fn subscription_calls(&self) -> Option<TokenStream>;

    fn event_handlers(&self) -> Option<TokenStream>;
}

pub trait ComponentIdentities {
    fn struct_name(&self) -> &'static str;
    fn struct_name_ident(&self) -> syn::Ident {
        syn::parse_str::<syn::Ident>(self.struct_name()).unwrap()
    }
    fn struct_form_ident(&self) -> syn::Ident {
        let str_repr = format!("{}Form", self.struct_name());
        syn::parse_str::<syn::Ident>(&str_repr).unwrap()
    }
    fn struct_form_components_ident(&self) -> syn::Ident {
        let str_repr = format!("{}FormComponents", self.struct_name());
        syn::parse_str::<syn::Ident>(&str_repr).unwrap()
    }
    fn struct_form_fields_ident(&self) -> syn::Ident {
        let str_repr = format!("{}FormFields", self.struct_name());
        syn::parse_str::<syn::Ident>(&str_repr).unwrap()
    }
    fn form_id_literal(&self) -> String {
        format!("{}-form", self.struct_name().to_snake_case())
    }
    fn ftl_label_ident(&self) -> syn::Ident {
        let str_repr = format!("{}LabelFtl", self.struct_name());
        syn::parse_str::<syn::Ident>(&str_repr).unwrap()
    }
    fn ftl_description_ident(&self) -> syn::Ident {
        let str_repr = format!("{}DescriptionFtl", self.struct_name());
        syn::parse_str::<syn::Ident>(&str_repr).unwrap()
    }
}

impl ComponentIdentities for FieldVariant {
    fn struct_name(&self) -> &'static str {
        self.field_type
    }
}
