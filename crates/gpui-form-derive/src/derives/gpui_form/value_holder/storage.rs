use super::present::field_variant_ident;
use super::*;

pub(super) fn form_base_type(field: &HolderFieldIr) -> Type {
    field.form_type().clone()
}

pub(super) trait HolderStorageStrategy {
    fn shape_policy_type_tokens(
        &self,
        context: &DeriveContext,
        field_name: &syn::Ident,
    ) -> ValueHolderResult<TokenStream>;
    fn storage_type_tokens(
        &self,
        context: &DeriveContext,
        base_type: &Type,
        field_name: &syn::Ident,
    ) -> ValueHolderResult<TokenStream>;
    fn present_tokens(
        &self,
        context: &DeriveContext,
        value: TokenStream,
        base_type: &Type,
        field_name: &syn::Ident,
    ) -> ValueHolderResult<TokenStream>;
    fn default_storage_tokens(
        &self,
        context: &DeriveContext,
        base_type: &Type,
        field_name: &syn::Ident,
    ) -> ValueHolderResult<TokenStream>;
    fn try_into_source_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        access: TokenStream,
        error_type: &syn::Ident,
    ) -> ValueHolderResult<TokenStream>;
    fn source_to_storage_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        access: TokenStream,
    ) -> ValueHolderResult<TokenStream>;
    fn storage_to_source_infallible_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        access: TokenStream,
    ) -> ValueHolderResult<TokenStream>;
    fn present_field_entry_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        present_field_ident: &syn::Ident,
        _present_lifetime: &syn::Lifetime,
    ) -> ValueHolderResult<TokenStream>;
    fn required_validation_builder_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        validator_ident: &syn::Ident,
        target_ident: &syn::Ident,
    ) -> ValueHolderResult<Option<TokenStream>>;
}

impl HolderStorageStrategy for HolderStoragePlan {
    fn shape_policy_type_tokens(
        &self,
        context: &DeriveContext,
        field_name: &syn::Ident,
    ) -> ValueHolderResult<TokenStream> {
        let Some(shape) = self.shape_policy() else {
            return Err(syn::Error::new_spanned(
                field_name,
                "only shape-policy storage has a generated policy type",
            ));
        };
        let runtime_crate = context.paths.gpui_form_facade_runtime();
        Ok(quote! {
            <#shape as #runtime_crate::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicy
        })
    }

    fn storage_type_tokens(
        &self,
        context: &DeriveContext,
        base_type: &Type,
        field_name: &syn::Ident,
    ) -> ValueHolderResult<TokenStream> {
        match self {
            HolderStoragePlan::OriginallyOptional => Ok(quote! { Option<#base_type> }),
            HolderStoragePlan::Direct => Ok(quote! { #base_type }),
            HolderStoragePlan::ShapePolicy { .. } => {
                let policy = self.shape_policy_type_tokens(context, field_name)?;
                let runtime_crate = context.paths.gpui_form_facade_runtime();
                Ok(quote! {
                    <#policy as #runtime_crate::shape::ValueStorage<#base_type>>::Storage
                })
            },
        }
    }

    fn present_tokens(
        &self,
        context: &DeriveContext,
        value: TokenStream,
        base_type: &Type,
        field_name: &syn::Ident,
    ) -> ValueHolderResult<TokenStream> {
        match self {
            HolderStoragePlan::OriginallyOptional => Ok(quote! { Some(#value) }),
            HolderStoragePlan::Direct => Ok(value),
            HolderStoragePlan::ShapePolicy { .. } => {
                let policy = self.shape_policy_type_tokens(context, field_name)?;
                let runtime_crate = context.paths.gpui_form_facade_runtime();
                Ok(quote! {
                    <#policy as #runtime_crate::shape::ValueStorage<#base_type>>::present(#value)
                })
            },
        }
    }

    fn default_storage_tokens(
        &self,
        context: &DeriveContext,
        base_type: &Type,
        field_name: &syn::Ident,
    ) -> ValueHolderResult<TokenStream> {
        match self {
            HolderStoragePlan::OriginallyOptional => Ok(quote! { None }),
            HolderStoragePlan::Direct => Ok(quote_spanned! {field_name.span()=>
                <#base_type as ::core::default::Default>::default()
            }),
            HolderStoragePlan::ShapePolicy { .. } => {
                let policy = self.shape_policy_type_tokens(context, field_name)?;
                let runtime_crate = context.paths.gpui_form_facade_runtime();
                Ok(quote_spanned! {field_name.span()=>
                    <#policy
                        as #runtime_crate::shape::DefaultValueStorage<#base_type>>::default_storage()
                })
            },
        }
    }

    fn try_into_source_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        access: TokenStream,
        error_type: &syn::Ident,
    ) -> ValueHolderResult<TokenStream> {
        let field_name = field.field_name();
        match self {
            HolderStoragePlan::OriginallyOptional => {
                if needs_into_conversion(field) {
                    let converted =
                        apply_into_conversion_result(field, quote! { value }, error_type);
                    Ok(quote! {
                        #access.map(|value| #converted).transpose()?
                    })
                } else {
                    Ok(access)
                }
            },
            HolderStoragePlan::Direct => {
                let converted = apply_into_conversion_result(field, access, error_type);
                Ok(quote! {
                    #converted?
                })
            },
            HolderStoragePlan::ShapePolicy { .. } => {
                let base_type = form_base_type(field);
                let field_name_str = field_name.to_string();
                let policy = shape_policy_type_tokens(context, field)?;
                let runtime_crate = context.paths.gpui_form_facade_runtime();
                if let Some(default_expr) = field.default_expr() {
                    let default_span = field.default_expr_span();
                    let default_original = source_default_tokens(default_expr, default_span);
                    let converted =
                        apply_into_conversion_result(field, quote! { value }, error_type);
                    Ok(quote! {
                        {
                            let __gpui_form_value =
                                <#policy
                                    as #runtime_crate::shape::ValueStorage<#base_type>>::map_into_value(
                                        #access,
                                        ::core::option::Option::Some,
                                        || ::core::option::Option::None,
                                    );
                            if let Some(value) = __gpui_form_value {
                                #converted?
                            } else {
                                #default_original
                            }
                        }
                    })
                } else {
                    let converted =
                        apply_into_conversion_result(field, quote! { value }, error_type);
                    Ok(quote! {
                        {
                            let value =
                                <#policy
                                    as #runtime_crate::shape::ValueStorage<#base_type>>::try_map_into_value(
                                        #access,
                                        ::core::convert::identity,
                                        #error_type::missing_required(#field_name_str)
                                    )?;
                            #converted?
                        }
                    })
                }
            },
        }
    }

    fn source_to_storage_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        access: TokenStream,
    ) -> ValueHolderResult<TokenStream> {
        match self {
            HolderStoragePlan::OriginallyOptional => {
                if needs_from_conversion(field) {
                    let converted = apply_from_conversion(field, quote! { value });
                    Ok(quote! {
                        #access.map(|value| #converted)
                    })
                } else {
                    Ok(access)
                }
            },
            HolderStoragePlan::Direct => {
                let converted = apply_from_conversion(field, access);
                Ok(quote! {
                    #converted
                })
            },
            HolderStoragePlan::ShapePolicy { .. } => {
                let base_type = form_base_type(field);
                let policy = shape_policy_type_tokens(context, field)?;
                let runtime_crate = context.paths.gpui_form_facade_runtime();
                if let Some(default_expr) = field.default_expr() {
                    let default_span = field.default_expr_span();
                    let default_original = source_default_tokens(default_expr, default_span);
                    let original_type = field.original_type();
                    let converted = apply_from_conversion(field, quote! { value });
                    let converted_default =
                        apply_from_conversion(field, quote! { default_original });
                    Ok(quote! {
                        {
                            let value = #access;
                            let default_original: #original_type = #default_original;
                            let value = #converted;
                            let default_value = #converted_default;
                            <#policy
                                as #runtime_crate::shape::ValueStorage<#base_type>>::present_unless_default(
                                    value,
                                    default_value
                                )
                        }
                    })
                } else {
                    let converted = apply_from_conversion(field, access);
                    Ok(quote! {
                        <#policy
                            as #runtime_crate::shape::ValueStorage<#base_type>>::present(#converted)
                    })
                }
            },
        }
    }

    fn storage_to_source_infallible_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        access: TokenStream,
    ) -> ValueHolderResult<TokenStream> {
        match self {
            HolderStoragePlan::OriginallyOptional => {
                if needs_into_conversion(field) {
                    let converted = apply_into_conversion(field, quote! { value });
                    Ok(quote! {
                        #access.map(|value| #converted)
                    })
                } else {
                    Ok(access)
                }
            },
            HolderStoragePlan::Direct => {
                let converted = apply_into_conversion(field, access);
                Ok(quote! {
                    #converted
                })
            },
            HolderStoragePlan::ShapePolicy { .. } => {
                let base_type = form_base_type(field);
                let converted = apply_into_conversion(field, quote! { value });
                let policy = shape_policy_type_tokens(context, field)?;
                let runtime_crate = context.paths.gpui_form_facade_runtime();
                if let Some(default_expr) = field.default_expr() {
                    let default_span = field.default_expr_span();
                    let default_original = source_default_tokens(default_expr, default_span);
                    Ok(quote! {
                        <#policy
                            as #runtime_crate::shape::ValueStorage<#base_type>>::map_into_value(
                                #access,
                                |value| #converted,
                                || #default_original
                            )
                    })
                } else {
                    let field_name_str = field.field_name().to_string();
                    Ok(quote! {
                        <#policy
                            as #runtime_crate::shape::ValueStorage<#base_type>>::map_into_value(
                                #access,
                                |value| #converted,
                                || panic!("Missing required value for field '{}'", #field_name_str)
                            )
                    })
                }
            },
        }
    }

    fn present_field_entry_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        present_field_ident: &syn::Ident,
        _present_lifetime: &syn::Lifetime,
    ) -> ValueHolderResult<TokenStream> {
        let field_name = field.field_name();
        let variant_ident = field_variant_ident(field_name);

        match self {
            HolderStoragePlan::OriginallyOptional => {
                if needs_into_conversion(field) {
                    if field.form_to_source_is_fallible() {
                        let converted = apply_try_into_conversion_raw(field, quote! { value });
                        Ok(quote! {
                            if let Some(value) = self.#field_name.clone() {
                                if let Ok(value) = #converted {
                                    entries.push(#present_field_ident::#variant_ident(value));
                                }
                            }
                        })
                    } else {
                        let converted = apply_into_conversion(field, quote! { value });
                        Ok(quote! {
                            if let Some(value) = self.#field_name.clone() {
                                entries.push(#present_field_ident::#variant_ident(#converted));
                            }
                        })
                    }
                } else {
                    Ok(quote! {
                        if let Some(value) = self.#field_name.as_ref() {
                            entries.push(#present_field_ident::#variant_ident(value));
                        }
                    })
                }
            },
            HolderStoragePlan::Direct => {
                if needs_into_conversion(field) {
                    if field.form_to_source_is_fallible() {
                        let converted = apply_try_into_conversion_raw(field, quote! { value });
                        Ok(quote! {
                            let value = self.#field_name.clone();
                            if let Ok(value) = #converted {
                                entries.push(#present_field_ident::#variant_ident(value));
                            }
                        })
                    } else {
                        let converted = apply_into_conversion(field, quote! { value });
                        Ok(quote! {
                            let value = self.#field_name.clone();
                            entries.push(#present_field_ident::#variant_ident(#converted));
                        })
                    }
                } else {
                    Ok(quote! {
                        entries.push(#present_field_ident::#variant_ident(&self.#field_name));
                    })
                }
            },
            HolderStoragePlan::ShapePolicy { .. } => {
                let base_type = form_base_type(field);
                let policy = shape_policy_type_tokens(context, field)?;
                let runtime_crate = context.paths.gpui_form_facade_runtime();
                if field.form_to_source_is_fallible() {
                    let converted = apply_try_into_conversion_raw(field, quote! { value });
                    Ok(quote! {
                        if let Some(value) =
                            <#policy
                                as #runtime_crate::shape::ValueStorage<#base_type>>::map_present_cloned(
                                    &self.#field_name,
                                    ::core::convert::identity
                                )
                        {
                            if let Ok(value) = #converted {
                                entries.push(#present_field_ident::#variant_ident(value));
                            }
                        }
                    })
                } else {
                    let converted = apply_into_conversion(field, quote! { value });
                    Ok(quote! {
                        if let Some(value) =
                            <#policy
                                as #runtime_crate::shape::ValueStorage<#base_type>>::map_present_cloned(
                                    &self.#field_name,
                                    |value| #converted
                                )
                        {
                            entries.push(#present_field_ident::#variant_ident(value));
                        }
                    })
                }
            },
        }
    }

    fn required_validation_builder_tokens(
        &self,
        context: &DeriveContext,
        field: &HolderFieldIr,
        validator_ident: &syn::Ident,
        target_ident: &syn::Ident,
    ) -> ValueHolderResult<Option<TokenStream>> {
        match self {
            HolderStoragePlan::ShapePolicy { .. } => {
                let base_type = form_base_type(field);
                let policy = shape_policy_type_tokens(context, field)?;
                Ok(Some(quote! {
                    full(
                        #validator_ident::<
                            #target_ident<
                                #policy,
                                #base_type
                            >
                        >
                    )
                }))
            },
            HolderStoragePlan::OriginallyOptional | HolderStoragePlan::Direct => Ok(None),
        }
    }
}

pub(super) fn shape_policy_type_tokens(
    context: &DeriveContext,
    field: &HolderFieldIr,
) -> ValueHolderResult<TokenStream> {
    field
        .storage()
        .shape_policy_type_tokens(context, field.field_name())
}

pub(super) fn form_field_type_tokens(
    context: &DeriveContext,
    field: &HolderFieldIr,
) -> ValueHolderResult<TokenStream> {
    let base_type = form_base_type(field);
    field
        .storage()
        .storage_type_tokens(context, &base_type, field.field_name())
}

pub(super) fn apply_from_conversion(field: &HolderFieldIr, value: TokenStream) -> TokenStream {
    apply_source_to_form_conversion_tokens(
        field.source_to_form_expr(),
        field.source_to_form_span(),
        value,
    )
}

fn apply_into_conversion(field: &HolderFieldIr, value: TokenStream) -> TokenStream {
    apply_form_to_source_conversion_tokens(
        field.form_to_source_expr(),
        field.form_to_source_span(),
        value,
    )
}

fn apply_into_conversion_result(
    field: &HolderFieldIr,
    value: TokenStream,
    error_type: &syn::Ident,
) -> TokenStream {
    if let Some(conversion) = field.form_to_source_expr() {
        let span = field.form_to_source_span();
        if field.form_to_source_is_fallible() {
            let field_name = field.field_name().to_string();
            quote_spanned! {span=>
                (#conversion)(#value)
                    .map_err(|error| #error_type::invalid_value(#field_name, error))
            }
        } else {
            quote_spanned! {span=>
                ::core::result::Result::Ok((#conversion)(#value))
            }
        }
    } else {
        quote! {
            ::core::result::Result::Ok(#value)
        }
    }
}

fn apply_try_into_conversion_raw(field: &HolderFieldIr, value: TokenStream) -> TokenStream {
    let Some(conversion) = field.form_to_source_expr() else {
        return quote! { ::core::result::Result::Ok(#value) };
    };
    let span = field.form_to_source_span();
    if field.form_to_source_is_fallible() {
        quote_spanned! {span=> (#conversion)(#value) }
    } else {
        quote_spanned! {span=> ::core::result::Result::Ok((#conversion)(#value)) }
    }
}

fn needs_from_conversion(field: &HolderFieldIr) -> bool {
    field.source_to_form_expr().is_some()
}

fn needs_into_conversion(field: &HolderFieldIr) -> bool {
    field.form_to_source_expr().is_some()
}

pub(super) fn conversion_signature_assertions(
    fields: &[HolderFieldPlan<'_>],
    wrapped_ident: &syn::Ident,
    original_impl_generics: TokenStream,
    where_clause: TokenStream,
) -> ValueHolderResult<TokenStream> {
    let mut assertions = Vec::new();

    for shared in fields.iter().map(|plan| plan.field) {
        let field_name = shared.field_name();
        let source_type = &shared.source_value_type;
        let form_type = &shared.form_type;

        if let Some(expr) = shared.source_to_form_expr() {
            let span = shared.source_to_form_span();
            let assert_ident =
                format_ident!("__{}_assert_{}_source_to_form", wrapped_ident, field_name);
            assertions.push(quote_spanned! {span=>
                #[allow(dead_code, non_snake_case)]
                fn #assert_ident #original_impl_generics() #where_clause {
                    fn assert_source_to_form<F>(_: F)
                    where
                        F: ::core::ops::FnOnce(#source_type) -> #form_type,
                    {}

                    assert_source_to_form(#expr);
                }
            });
        }

        if let Some(expr) = shared.form_to_source_expr() {
            let span = shared.form_to_source_span();
            let assert_ident =
                format_ident!("__{}_assert_{}_form_to_source", wrapped_ident, field_name);
            if shared.form_to_source_is_fallible() {
                assertions.push(quote_spanned! {span=>
                    #[allow(dead_code, non_snake_case)]
                    fn #assert_ident #original_impl_generics() #where_clause {
                        fn assert_form_to_source<F, E>(_: F)
                        where
                            F: ::core::ops::FnOnce(#form_type) -> ::core::result::Result<#source_type, E>,
                            E: ::core::fmt::Debug,
                        {}

                        assert_form_to_source(#expr);
                    }
                });
            } else {
                assertions.push(quote_spanned! {span=>
                    #[allow(dead_code, non_snake_case)]
                    fn #assert_ident #original_impl_generics() #where_clause {
                        fn assert_form_to_source<F>(_: F)
                        where
                            F: ::core::ops::FnOnce(#form_type) -> #source_type,
                        {}

                        assert_form_to_source(#expr);
                    }
                });
            }
        }
    }

    Ok(quote! {
        #(#assertions)*
    })
}

pub(super) fn try_from_field_tokens(
    context: &DeriveContext,
    field: &HolderFieldIr,
    source: TokenStream,
    error_type: &syn::Ident,
) -> ValueHolderResult<TokenStream> {
    let field_name = field.field_name();
    let access = quote! { #source.#field_name };
    let value = field
        .storage()
        .try_into_source_tokens(context, field, access, error_type)?;
    Ok(quote! {
        #field_name: #value
    })
}

pub(super) fn generate_conversion_error_type(error_name: &syn::Ident) -> TokenStream {
    let kind_name = format_ident!("{error_name}Kind");
    quote! {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum #kind_name {
            MissingRequired,
            InvalidValue,
        }

        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct #error_name {
            pub field_name: &'static str,
            kind: #kind_name,
            detail: Option<String>,
        }

        impl #error_name {
            pub fn missing_required(field_name: &'static str) -> Self {
                Self {
                    field_name,
                    kind: #kind_name::MissingRequired,
                    detail: None,
                }
            }

            pub fn invalid_value(
                field_name: &'static str,
                error: impl ::core::fmt::Debug,
            ) -> Self {
                Self {
                    field_name,
                    kind: #kind_name::InvalidValue,
                    detail: Some(::std::format!("{error:?}")),
                }
            }

            pub fn kind(&self) -> #kind_name {
                self.kind
            }

            pub fn detail(&self) -> Option<&str> {
                self.detail.as_deref()
            }

            pub fn validator_name(&self) -> &'static str {
                match self.kind {
                    #kind_name::MissingRequired => "required",
                    #kind_name::InvalidValue => "conversion",
                }
            }
        }

        impl ::core::fmt::Display for #error_name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self.kind {
                    #kind_name::MissingRequired => {
                        write!(
                            f,
                            "Missing required value for field '{}'",
                            self.field_name
                        )
                    },
                    #kind_name::InvalidValue => {
                        if let Some(detail) = self.detail() {
                            write!(
                                f,
                                "Invalid value for field '{}': {}",
                                self.field_name,
                                detail
                            )
                        } else {
                            write!(
                                f,
                                "Invalid value for field '{}'",
                                self.field_name
                            )
                        }
                    },
                }
            }
        }

        impl ::core::error::Error for #error_name {}
    }
}
