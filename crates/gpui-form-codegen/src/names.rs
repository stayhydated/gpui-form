use derive_more::From;
use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::Ident;

#[derive(From)]
pub struct ComponentFieldName(pub Ident);

impl ComponentFieldName {
    pub fn try_new(component_name: &str, field_name: &str) -> syn::Result<Self> {
        let raw = format!("{}_{}", field_name, component_name);
        syn::parse_str::<Ident>(&raw).map(Self).map_err(|err| {
            syn::Error::new(
                Span::call_site(),
                format!("failed to generate component field identifier `{raw}`: {err}"),
            )
        })
    }

    #[allow(dead_code)]
    pub fn new(component_name: &str, field_name: &str) -> Self {
        Self::try_new(component_name, field_name).unwrap_or_else(|_| {
            Ident::new("__gpui_form_invalid_component_field", Span::call_site()).into()
        })
    }
}

impl ToTokens for ComponentFieldName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.0;
        tokens.extend(quote! { #ident });
    }
}

#[macro_export]
macro_rules! component_field_name {
    ($name:expr) => {
        $crate::names::ComponentFieldName::new(Self::component_name(), $name)
    };
}
