use super::storage::HolderStorageStrategy as _;
use super::*;

pub(crate) fn field_variant_ident(field_name: &syn::Ident) -> syn::Ident {
    let source = field_name.to_string();
    let source = source.strip_prefix("r#").unwrap_or(&source);
    let mut ident = String::new();
    let mut uppercase_next = true;

    for ch in source.chars() {
        if ch.is_ascii_alphanumeric() {
            if uppercase_next {
                ident.push(ch.to_ascii_uppercase());
                uppercase_next = false;
            } else {
                ident.push(ch);
            }
        } else {
            uppercase_next = true;
        }
    }

    if ident.is_empty() {
        ident.push_str("Field");
    }
    if ident.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        ident.insert_str(0, "Field");
    }

    format_ident!("{ident}")
}

fn present_field_variant_type_tokens<'a>(
    field: &'a HolderFieldIr,
    present_lifetime: &'a syn::Lifetime,
    present: &PresentFieldPlan,
) -> TokenStream {
    let source_type = &field.source_value_type;

    if present.borrows_source_value {
        quote! { &#present_lifetime #source_type }
    } else {
        quote! { #source_type }
    }
}

pub(super) fn generate_present_field_variant(
    field: &HolderFieldIr,
    present_lifetime: &syn::Lifetime,
    present: &PresentFieldPlan,
) -> TokenStream {
    let variant_ident = field_variant_ident(field.field_name());
    let variant_type = present_field_variant_type_tokens(field, present_lifetime, present);

    quote! {
        #variant_ident(#variant_type)
    }
}

pub(super) fn generate_present_field_entry(
    context: &DeriveContext,
    field: &HolderFieldIr,
    present_field_ident: &syn::Ident,
    present_lifetime: &syn::Lifetime,
) -> ValueHolderResult<TokenStream> {
    field.storage().present_field_entry_tokens(
        context,
        field,
        present_field_ident,
        present_lifetime,
    )
}

pub(super) fn value_storage_policy_validator_tokens(
    context: &DeriveContext,
    validator_ident: &syn::Ident,
    builder_ident: &syn::Ident,
    target_ident: &syn::Ident,
    target_trait_ident: &syn::Ident,
    enable_koruma_fluent: bool,
) -> TokenStream {
    let runtime_crate = context.paths.gpui_form_facade_runtime();
    let fluent_impl = if enable_koruma_fluent {
        quote! {
            impl<Target> ::es_fluent::FluentMessage for #validator_ident<Target> {
                fn to_fluent_string_with(
                    &self,
                    _localize: &mut ::es_fluent::FluentMessageLookup<'_>,
                ) -> String {
                    "This field is required.".to_string()
                }
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #[doc(hidden)]
        pub struct #target_ident<Policy, Value> {
            _marker: ::core::marker::PhantomData<fn() -> (Policy, Value)>,
        }

        #[doc(hidden)]
        pub trait #target_trait_ident {
            type Storage;

            fn is_present(value: &Self::Storage) -> bool;
        }

        impl<Policy, Value> #target_trait_ident for #target_ident<Policy, Value>
        where
            Policy: #runtime_crate::shape::ValueStorage<Value>,
        {
            type Storage =
                <Policy as #runtime_crate::shape::ValueStorage<Value>>::Storage;

            fn is_present(value: &Self::Storage) -> bool {
                <Policy as #runtime_crate::shape::ValueStorage<Value>>::is_present(value)
            }
        }

        #[doc(hidden)]
        pub struct #validator_ident<Target> {
            _marker: ::core::marker::PhantomData<fn() -> Target>,
        }

        impl<Target> ::core::clone::Clone for #validator_ident<Target> {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<Target> ::core::marker::Copy for #validator_ident<Target> {}

        impl<Target> ::core::fmt::Debug for #validator_ident<Target> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(stringify!(#validator_ident)).finish()
            }
        }

        impl<Target> #validator_ident<Target> {
            pub fn __koruma_builder() -> #builder_ident<Target> {
                #builder_ident {
                    _marker: ::core::marker::PhantomData,
                }
            }

            pub fn builder() -> #builder_ident<Target> {
                #builder_ident {
                    _marker: ::core::marker::PhantomData,
                }
            }
        }

        impl<Target> #validator_ident<Target>
        where
            Target: #target_trait_ident,
        {
            pub fn validate(&self, value: &<Target as #target_trait_ident>::Storage) -> bool {
                <Target as #target_trait_ident>::is_present(value)
            }
        }

        #[doc(hidden)]
        pub struct #builder_ident<Target> {
            _marker: ::core::marker::PhantomData<fn() -> Target>,
        }

        impl<Target> ::core::clone::Clone for #builder_ident<Target> {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<Target> ::core::marker::Copy for #builder_ident<Target> {}

        impl<Target> ::core::fmt::Debug for #builder_ident<Target> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(stringify!(#builder_ident)).finish()
            }
        }

        impl<Target> #builder_ident<Target> {
            pub fn build(self) -> #validator_ident<Target> {
                #validator_ident {
                    _marker: ::core::marker::PhantomData,
                }
            }
        }

        impl<Target> ::koruma::__private::CaptureValueRef<
            <Target as #target_trait_ident>::Storage
        >
            for #builder_ident<Target>
        where
            Target: #target_trait_ident,
        {
            type Output = Self;

            fn capture_value_ref(
                self,
                _value: &<Target as #target_trait_ident>::Storage,
            ) -> Self::Output {
                self
            }
        }

        impl<Target> ::koruma::__private::BuildValidator
            for #builder_ident<Target>
        {
            type Validator = #validator_ident<Target>;

            fn build_validator(self) -> Self::Validator {
                self.build()
            }
        }

        impl<Target> ::koruma::Validate<<Target as #target_trait_ident>::Storage>
            for #validator_ident<Target>
        where
            Target: #target_trait_ident,
        {
            fn validate(
                &self,
                value: &<Target as #target_trait_ident>::Storage,
            ) -> bool {
                self.validate(value)
            }
        }

        #fluent_impl
    }
}

pub(super) fn required_validation_builder_tokens(
    context: &DeriveContext,
    field: &HolderFieldIr,
    validator_ident: &syn::Ident,
    target_ident: &syn::Ident,
) -> ValueHolderResult<Option<TokenStream>> {
    if !field.needs_required_validation() {
        return Ok(None);
    }

    field.storage().required_validation_builder_tokens(
        context,
        field,
        validator_ident,
        target_ident,
    )
}
