use super::*;

pub(super) fn context_submit_registration_tokens(
    context: &DeriveContext,
    original_input: &DeriveInput,
    context_submit: Option<&McpContextSubmitOptions>,
) -> TokenStream {
    let Some(context_submit) = context_submit else {
        return quote! {};
    };

    let facade_crate = &context.paths.gpui_form;
    let original_ident = &original_input.ident;
    let context_type_id_fn_ident =
        format_ident!("__{}_gpui_form_mcp_context_submit_type_id", original_ident);
    let context_register_fn_ident =
        format_ident!("__{}_gpui_form_mcp_register_context_submit", original_ident);
    let context_definitions_fn_ident = format_ident!(
        "__{}_gpui_form_mcp_context_submit_tool_definitions",
        original_ident
    );
    let descriptor_fn_ident = format_ident!("__{}_gpui_form_mcp_descriptor", original_ident);
    let (impl_generics, ty_generics, where_clause) = original_input.generics.split_for_impl();
    let (context_type, response_type, context_submit_impl) = match context_submit {
        McpContextSubmitOptions::Explicit(context_submit) => {
            let context_type = &context_submit.context;
            let response_type = context_submit
                .response
                .as_ref()
                .map(|response_type| quote! { #response_type })
                .unwrap_or_else(|| quote! { #facade_crate::mcp::McpObject });
            let error_type = context_submit
                .error
                .as_ref()
                .map(|error_type| quote! { #error_type })
                .unwrap_or_else(|| quote! { String });
            let context_type = quote! { #context_type };
            let context_submit_impl = context_submit
                .submit
                .as_ref()
                .map(|submit| {
                    let submit_body =
                        if let Some(map_response) = context_submit.map_response.as_ref() {
                            quote! {
                                async move {
                                    let __gpui_form_response = (#submit)(self, context).await?;
                                    (#map_response)(__gpui_form_response)
                                }
                            }
                        } else {
                            quote! {
                                (#submit)(self, context)
                            }
                        };

                    quote! {
                        impl #impl_generics #facade_crate::mcp::McpContextSubmit<#context_type>
                            for #original_ident #ty_generics
                            #where_clause
                        {
                            type Response = #response_type;
                            type Error = #error_type;

                            fn submit_with_context(
                                self,
                                context: #context_type,
                            ) -> impl ::std::future::Future<
                                Output = Result<Self::Response, Self::Error>
                            > + Send + 'static {
                                #submit_body
                            }
                        }
                    }
                })
                .unwrap_or_else(|| quote! {});

            (context_type, response_type, context_submit_impl)
        },
        McpContextSubmitOptions::Submitter(context_submit) => {
            let submitter = &context_submit.submitter;
            let context_type = quote! { <#original_ident #ty_generics as #submitter>::Context };
            let submitter_response_type =
                quote! { <#original_ident #ty_generics as #submitter>::Response };
            let response_type = context_submit
                .response
                .as_ref()
                .map(|response_type| quote! { #response_type })
                .unwrap_or_else(|| {
                    if context_submit.map_response.is_some() {
                        quote! { #facade_crate::mcp::McpObject }
                    } else {
                        quote! { #submitter_response_type }
                    }
                });
            let error_type = quote! { <#original_ident #ty_generics as #submitter>::Error };
            let mut submitter_generics = original_input.generics.clone();
            {
                let where_clause = submitter_generics.make_where_clause();
                where_clause.predicates.push(syn::parse_quote! {
                    #original_ident #ty_generics: #submitter
                });
                where_clause.predicates.push(syn::parse_quote! {
                    #context_type: Clone + Send + Sync + 'static
                });
                where_clause.predicates.push(syn::parse_quote! {
                    #response_type: #facade_crate::mcp::Serialize + #facade_crate::mcp::McpJsonSchema
                });
                where_clause.predicates.push(syn::parse_quote! {
                    #error_type: ::core::fmt::Display
                });
            }
            let (submitter_impl_generics, _, submitter_where_clause) =
                submitter_generics.split_for_impl();
            let submit_body = if let Some(map_response) = context_submit.map_response.as_ref() {
                quote! {
                    async move {
                        let __gpui_form_response =
                            <#original_ident #ty_generics as #submitter>::submit_with_context(
                                self,
                                context,
                            )
                            .await?;
                        (#map_response)(__gpui_form_response)
                    }
                }
            } else {
                quote! {
                    <#original_ident #ty_generics as #submitter>::submit_with_context(
                        self,
                        context,
                    )
                }
            };
            let context_submit_impl = quote! {
                impl #submitter_impl_generics #facade_crate::mcp::McpContextSubmit<#context_type>
                    for #original_ident #ty_generics
                    #submitter_where_clause
                {
                    type Response = #response_type;
                    type Error = #error_type;

                    fn submit_with_context(
                        self,
                        context: #context_type,
                    ) -> impl ::std::future::Future<
                        Output = Result<Self::Response, Self::Error>
                    > + Send + 'static {
                        #submit_body
                    }
                }
            };

            (context_type, response_type, context_submit_impl)
        },
    };

    quote! {
        #context_submit_impl

        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #context_type_id_fn_ident() -> ::std::any::TypeId {
            ::std::any::TypeId::of::<#context_type>()
        }

        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #context_register_fn_ident(
            server: &mut #facade_crate::mcp::McpServer,
            context: #facade_crate::mcp::McpSubmitContext,
            options: #facade_crate::mcp::McpFormEditorOptions,
        ) -> Result<(), #facade_crate::mcp::McpToolError> {
            fn __gpui_form_mcp_context_submit_assert<T, Context, Response>()
            where
                T: #facade_crate::mcp::McpContextSubmit<Context, Response = Response>,
                Context: Clone + Send + Sync + 'static,
                Response: #facade_crate::mcp::Serialize + #facade_crate::mcp::McpJsonSchema,
            {}
            __gpui_form_mcp_context_submit_assert::<
                #original_ident #ty_generics,
                #context_type,
                #response_type,
            >();

            let context = context.downcast::<#context_type>().map_err(|_| {
                #facade_crate::mcp::McpToolError::validation(format!(
                    "context submitter for `{}` requires context `{}`",
                    stringify!(#original_ident),
                    ::std::any::type_name::<#context_type>(),
                ))
            })?;

            #facade_crate::mcp::form::<#original_ident #ty_generics>(server)
                .model_async_with_editor_options(options, move |request| {
                    let context = (*context).clone();
                    <
                        #original_ident #ty_generics
                        as #facade_crate::mcp::McpContextSubmit<#context_type>
                    >::submit_with_context(request, context)
                })
        }

        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #context_definitions_fn_ident(
        ) -> Result<Vec<#facade_crate::mcp::ToolDefinition>, #facade_crate::mcp::McpToolError> {
            fn __gpui_form_mcp_context_submit_assert<T, Context, Response>()
            where
                T: #facade_crate::mcp::McpContextSubmit<Context, Response = Response>,
                Context: Clone + Send + Sync + 'static,
                Response: #facade_crate::mcp::Serialize + #facade_crate::mcp::McpJsonSchema,
            {}
            __gpui_form_mcp_context_submit_assert::<
                #original_ident #ty_generics,
                #context_type,
                #response_type,
            >();
            <
                #original_ident #ty_generics as #facade_crate::mcp::McpSubmitArgument
            >::tool_definitions_with_editor::<#response_type>()
        }

        #facade_crate::mcp::registry::inventory::submit! {
            #facade_crate::mcp::registry::McpContextSubmitRegistration::new(
                #descriptor_fn_ident,
                #context_type_id_fn_ident,
                #context_register_fn_ident,
                #context_definitions_fn_ident,
            )
        }
    }
}
