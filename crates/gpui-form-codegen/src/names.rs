use derive_more::From;
use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::Ident;

#[derive(From)]
pub struct ComponentEntityFieldName(pub Ident);

impl ComponentEntityFieldName {
    pub fn try_new(field_name: &str) -> syn::Result<Self> {
        Self::try_new_spanned(field_name, Span::call_site())
    }

    pub fn try_new_spanned(field_name: &str, span: Span) -> syn::Result<Self> {
        let raw = field_name.to_string();
        syn::parse_str::<Ident>(&raw).map(Self).map_err(|err| {
            syn::Error::new(
                span,
                format!("failed to generate component entity field identifier `{raw}`: {err}"),
            )
        })
    }

    #[allow(dead_code)]
    pub fn new(field_name: &str) -> Self {
        Self::try_new(field_name).unwrap_or_else(|_| {
            Ident::new(
                "__gpui_form_invalid_component_entity_field",
                Span::call_site(),
            )
            .into()
        })
    }
}

impl ToTokens for ComponentEntityFieldName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.0;
        tokens.extend(quote! { #ident });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn component_entity_field_names_preserve_valid_identifiers() {
        let name = ComponentEntityFieldName::try_new("profile_input").unwrap();
        assert_eq!(quote!(#name).to_string(), "profile_input");

        let ident: Ident = syn::parse_quote!(email_input);
        let converted = ComponentEntityFieldName::from(ident);
        assert_eq!(quote!(#converted).to_string(), "email_input");
    }

    #[test]
    fn component_entity_field_names_report_invalid_identifiers_at_the_given_span() {
        let span = Span::mixed_site();
        let error = match ComponentEntityFieldName::try_new_spanned("two words", span) {
            Ok(_) => panic!("invalid identifier unexpectedly parsed"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("failed to generate component entity field identifier `two words`")
        );
    }

    #[test]
    fn infallible_constructor_uses_a_stable_fallback_identifier() {
        let name = ComponentEntityFieldName::new("two words");
        assert_eq!(
            quote!(#name).to_string(),
            "__gpui_form_invalid_component_entity_field"
        );
    }
}
