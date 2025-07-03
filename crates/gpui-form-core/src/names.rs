use derive_more::From;
use quote::{ToTokens, format_ident, quote};

#[derive(From)]
pub struct ComponentFieldName(pub String);

impl ComponentFieldName {
    pub fn new(component_name: &str, field_name: &str) -> Self {
        format!("{}_{}", field_name, component_name).into()
    }
}

impl ToTokens for ComponentFieldName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = format_ident!("{}", &self.0);
        tokens.extend(quote! { #ident });
    }
}

#[macro_export]
macro_rules! component_field_name {
    ($name:expr) => {
        $crate::names::ComponentFieldName::new(Self::component_name(), $name)
    };
}

#[derive(From)]
pub struct ComponentValueFieldName(pub String);

impl From<&ComponentFieldName> for ComponentValueFieldName {
    fn from(field_name: &ComponentFieldName) -> Self {
        format!("{}_{}", field_name.0, "value").into()
    }
}

impl From<ComponentFieldName> for ComponentValueFieldName {
    fn from(field_name: ComponentFieldName) -> Self {
        format!("{}_{}", field_name.0, "value").into()
    }
}

impl ToTokens for ComponentValueFieldName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = format_ident!("{}", &self.0);
        tokens.extend(quote! { #ident });
    }
}
