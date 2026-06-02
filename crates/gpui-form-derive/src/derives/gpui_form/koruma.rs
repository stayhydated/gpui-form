use koruma_derive_core::{
    ParsedValidatorUse, ValidatorAttr, ValidatorTargetSelector, ValidatorTypeArg,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Convert a ValidatorAttr to a TokenStream for generating koruma attribute.
/// This produces normalized direct-validator tokens like
/// `ValidatorPath::<Type>::arg1(val1).arg2(val2)`.
pub fn validator_attr_to_tokens(validator: &ValidatorAttr) -> TokenStream {
    let path = validator.path();

    let type_params = if matches!(validator.type_arg(), ValidatorTypeArg::Infer) {
        quote! { ::<_> }
    } else if let Some(explicit_ty) = validator.explicit_type() {
        quote! { ::<#explicit_ty> }
    } else {
        quote! {}
    };

    let setter_calls = validator.setter_calls();
    let mut setter_calls = setter_calls.iter();
    let Some(first_method) = setter_calls.next() else {
        return quote! { #path #type_params };
    };

    let method_name = first_method.method();
    let args = first_method.args();
    let builder_calls = setter_calls.map(|method| {
        let method_name = method.method();
        let args = method.args();
        quote! { .#method_name(#(#args),*) }
    });

    quote! { #path #type_params ::#method_name(#(#args),*) #(#builder_calls)* }
}

pub fn validator_use_to_tokens(validator_use: &ParsedValidatorUse) -> TokenStream {
    let validator = validator_attr_to_tokens(validator_use.validator());
    let validator = match validator_use.target() {
        ValidatorTargetSelector::Default => validator,
        ValidatorTargetSelector::Full { .. } => quote! { full(#validator) },
        ValidatorTargetSelector::Unwrapped { .. } => quote! { unwrapped(#validator) },
    };

    if let Some(label) = validator_use.label() {
        let label_ident = Ident::new(
            &label.to_string(),
            validator_use
                .label_span()
                .unwrap_or_else(|| validator_use.source_span()),
        );
        quote! { #label_ident = #validator }
    } else {
        validator
    }
}
