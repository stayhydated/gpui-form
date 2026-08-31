mod context_submit;
mod descriptors;
mod metadata;
mod validation;

use context_submit::context_submit_registration_tokens;
use descriptors::{field_decode_arm_tokens, field_decode_tokens, field_descriptor_tokens};
use metadata::tool_metadata_tokens;
use validation::field_validation_issue_tokens;

use component_shape_codegen::{McpToolMetadataParts, mcp_tool_metadata_tokens};
use gpui_form_codegen::metadata::rust_type_tokens;
use koruma_derive_core::{ParsedValidatorUse, ValidatorTargetSelector, ValidatorTypeArg};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens as _, format_ident, quote};
use syn::{DeriveInput, Expr, Lit, LitStr, Path, Type, UnOp, spanned::Spanned as _};

use crate::derives::gpui_form::attrs::{
    McpContextSubmitOptions, McpIconThemeOption, McpToolOptions,
};
use crate::derives::gpui_form::holder_plan::{HolderConversionMode, ValueHolderPlan};
use crate::derives::gpui_form::ir::{DeriveContext, FieldPlan, HolderFieldIr, HolderStoragePlan};

type McpResult<T> = syn::Result<T>;

pub(super) fn generate_mcp_impl(
    context: &DeriveContext,
    original_input: &DeriveInput,
    field_plans: &[FieldPlan],
    holder_plan: &ValueHolderPlan<'_>,
    enable_koruma: bool,
    mcp_tool_options: Option<&McpToolOptions>,
) -> McpResult<TokenStream> {
    let facade_crate = &context.paths.gpui_form;
    let original_ident = &original_input.ident;
    let holder_ident = format_ident!("{}FormValueHolder", original_ident);
    let fields_const_ident = format_ident!("__{}GpuiFormMcpFields", original_ident);
    let descriptor_fn_ident = format_ident!("__{}_gpui_form_mcp_descriptor", original_ident);
    let editor_register_fn_ident =
        format_ident!("__{}_gpui_form_mcp_register_editor", original_ident);
    let editor_definitions_fn_ident =
        format_ident!("__{}_gpui_form_mcp_editor_tool_definitions", original_ident);
    let (impl_generics, ty_generics, where_clause) = original_input.generics.split_for_impl();
    let tool_metadata = tool_metadata_tokens(context, original_input, mcp_tool_options)?;
    let context_submit_registration = context_submit_registration_tokens(
        context,
        original_input,
        mcp_tool_options.and_then(|options| options.context_submit.as_ref()),
    );

    let mut field_descriptor_consts = Vec::new();
    let field_descriptors: Vec<TokenStream> = field_plans
        .iter()
        .enumerate()
        .filter_map(|(field_index, field)| {
            field_descriptor_tokens(context, original_ident, field_index, field).map(|tokens| {
                field_descriptor_consts.extend(tokens.constants);
                tokens.descriptor
            })
        })
        .collect();
    let field_decoders: Vec<TokenStream> = holder_plan
        .rendered_fields()
        .iter()
        .map(|field| field_decode_tokens(context, field.field))
        .collect();
    let rendered_fields: Vec<&HolderFieldIr> = holder_plan
        .rendered_fields()
        .iter()
        .map(|field| field.field)
        .collect();
    let field_decode_arms: Vec<TokenStream> = rendered_fields
        .iter()
        .map(|field| field_decode_arm_tokens(context, field))
        .collect();

    let validation_issue_extractors: Vec<TokenStream> = field_plans
        .iter()
        .enumerate()
        .filter_map(|(field_index, field)| {
            field.shared().map(|field| {
                field_validation_issue_tokens(context, original_ident, field_index, field)
            })
        })
        .collect();
    let validation_issue_tokens = if enable_koruma {
        quote! {
            match holder.validate() {
                Ok(()) => Ok(()),
                Err(__gpui_form_validation_error) => {
                    let mut __gpui_form_issues = ::std::vec::Vec::new();
                    #(#validation_issue_extractors)*
                    if __gpui_form_issues.is_empty() {
                        for __gpui_form_koruma_issue in
                            ::koruma::ValidationIssues::issues(&__gpui_form_validation_error)
                        {
                            let __gpui_form_scope = match __gpui_form_koruma_issue.scope() {
                                ::koruma::ValidationIssueScope::Form => {
                                    #facade_crate::mcp::McpValidationScope::Form
                                }
                                ::koruma::ValidationIssueScope::Field => {
                                    #facade_crate::mcp::McpValidationScope::Field
                                }
                                ::koruma::ValidationIssueScope::Element => {
                                    #facade_crate::mcp::McpValidationScope::Element
                                }
                            };
                            let mut __gpui_form_issue =
                                #facade_crate::mcp::McpValidationIssue::custom(
                                    __gpui_form_scope,
                                    __gpui_form_koruma_issue.message().to_string(),
                                );
                            if let Some(__gpui_form_field) =
                                __gpui_form_koruma_issue.field_name()
                            {
                                __gpui_form_issue =
                                    __gpui_form_issue.with_field(__gpui_form_field);
                            }
                            if let Some(__gpui_form_validator) =
                                __gpui_form_koruma_issue.validator()
                            {
                                __gpui_form_issue =
                                    __gpui_form_issue.with_validator(__gpui_form_validator);
                            }
                            if let Some(__gpui_form_label) =
                                __gpui_form_koruma_issue.label()
                            {
                                __gpui_form_issue =
                                    __gpui_form_issue.with_label(__gpui_form_label);
                            }
                            if let Some(__gpui_form_element_index) =
                                __gpui_form_koruma_issue.element_index()
                            {
                                __gpui_form_issue = __gpui_form_issue
                                    .with_element_index(__gpui_form_element_index);
                            }
                            __gpui_form_issues.push(__gpui_form_issue);
                        }
                    }
                    if __gpui_form_issues.is_empty() {
                        __gpui_form_issues.push(
                            #facade_crate::mcp::McpValidationIssue::form(
                                ::std::format!("{__gpui_form_validation_error:?}")
                            )
                        );
                    }
                    Err(__gpui_form_issues)
                }
            }
        }
    } else {
        quote! {
            let _ = holder;
            Ok(())
        }
    };
    let validation_tokens = quote! {
        <Self as #facade_crate::mcp::McpForm>::validate_holder_issues(holder)
            .map_err(#facade_crate::mcp::validation_issues_error)
    };

    let model_impl = match holder_plan.conversion_mode() {
        HolderConversionMode::Infallible => {
            quote! {
                impl #impl_generics #facade_crate::mcp::McpFormModel
                    for #original_ident #ty_generics #where_clause
                {
                    fn holder_into_model(
                        holder: Self::ValueHolder
                    ) -> Result<Self, #facade_crate::mcp::McpToolError> {
                        Ok(holder.into_original())
                    }
                }

                impl #impl_generics #facade_crate::mcp::McpSubmitArgument
                    for #original_ident #ty_generics #where_clause
                {
                    type Form = Self;

                    fn register_sync<Handler, Response, Error>(
                        server: &mut #facade_crate::mcp::McpServer,
                        handler: Handler,
                    ) -> Result<(), #facade_crate::mcp::McpToolError>
                    where
                        Handler: Fn(Self) -> Result<Response, Error> + Send + Sync + 'static,
                        Response: #facade_crate::mcp::Serialize + #facade_crate::mcp::McpJsonSchema,
                        Error: ::core::fmt::Display,
                    {
                        #facade_crate::mcp::form::<Self>(server).model(handler)
                    }

                    fn register_async<Handler, Fut, Response, Error>(
                        server: &mut #facade_crate::mcp::McpServer,
                        handler: Handler,
                    ) -> Result<(), #facade_crate::mcp::McpToolError>
                    where
                        Handler: Fn(Self) -> Fut + Send + Sync + 'static,
                        Fut: ::core::future::Future<Output = Result<Response, Error>> + Send + 'static,
                        Response: #facade_crate::mcp::Serialize + #facade_crate::mcp::McpJsonSchema,
                        Error: ::core::fmt::Display,
                    {
                        #facade_crate::mcp::form::<Self>(server).model_async(handler)
                    }

                    fn register_sync_with_editor<Handler, Response, Error>(
                        server: &mut #facade_crate::mcp::McpServer,
                        handler: Handler,
                    ) -> Result<(), #facade_crate::mcp::McpToolError>
                    where
                        Self::Form: #facade_crate::mcp::McpEditableForm,
                        <Self::Form as #facade_crate::mcp::McpForm>::ValueHolder:
                            Clone + Default + Send + 'static,
                        Handler: Fn(Self) -> Result<Response, Error> + Send + Sync + 'static,
                        Response: #facade_crate::mcp::Serialize + #facade_crate::mcp::McpJsonSchema,
                        Error: ::core::fmt::Display,
                    {
                        #facade_crate::mcp::form::<Self>(server).model_with_editor(handler)
                    }

                    fn register_async_with_editor<Handler, Fut, Response, Error>(
                        server: &mut #facade_crate::mcp::McpServer,
                        handler: Handler,
                    ) -> Result<(), #facade_crate::mcp::McpToolError>
                    where
                        Self::Form: #facade_crate::mcp::McpEditableForm,
                        <Self::Form as #facade_crate::mcp::McpForm>::ValueHolder:
                            Clone + Default + Send + 'static,
                        Handler: Fn(Self) -> Fut + Send + Sync + 'static,
                        Fut: ::core::future::Future<Output = Result<Response, Error>> + Send + 'static,
                        Response: #facade_crate::mcp::Serialize + #facade_crate::mcp::McpJsonSchema,
                        Error: ::core::fmt::Display,
                    {
                        #facade_crate::mcp::form::<Self>(server).model_async_with_editor(handler)
                    }
                }
            }
        },
        HolderConversionMode::FallibleRequired => {
            quote! {
                impl #impl_generics #facade_crate::mcp::McpFormModel
                    for #original_ident #ty_generics #where_clause
                {
                    fn holder_into_model(
                        holder: Self::ValueHolder
                    ) -> Result<Self, #facade_crate::mcp::McpToolError> {
                        holder.try_into_original().map_err(|error| {
                            #facade_crate::mcp::validation_issues_error(
                                vec![
                                    #facade_crate::mcp::McpValidationIssue::custom(
                                        #facade_crate::mcp::McpValidationScope::Field,
                                        error.to_string()
                                    )
                                    .with_field(error.field_name)
                                    .with_validator(error.validator_name())
                                ]
                            )
                        })
                    }
                }

                impl #impl_generics #facade_crate::mcp::McpSubmitArgument
                    for #original_ident #ty_generics #where_clause
                {
                    type Form = Self;

                    fn register_sync<Handler, Response, Error>(
                        server: &mut #facade_crate::mcp::McpServer,
                        handler: Handler,
                    ) -> Result<(), #facade_crate::mcp::McpToolError>
                    where
                        Handler: Fn(Self) -> Result<Response, Error> + Send + Sync + 'static,
                        Response: #facade_crate::mcp::Serialize + #facade_crate::mcp::McpJsonSchema,
                        Error: ::core::fmt::Display,
                    {
                        #facade_crate::mcp::form::<Self>(server).model(handler)
                    }

                    fn register_async<Handler, Fut, Response, Error>(
                        server: &mut #facade_crate::mcp::McpServer,
                        handler: Handler,
                    ) -> Result<(), #facade_crate::mcp::McpToolError>
                    where
                        Handler: Fn(Self) -> Fut + Send + Sync + 'static,
                        Fut: ::core::future::Future<Output = Result<Response, Error>> + Send + 'static,
                        Response: #facade_crate::mcp::Serialize + #facade_crate::mcp::McpJsonSchema,
                        Error: ::core::fmt::Display,
                    {
                        #facade_crate::mcp::form::<Self>(server).model_async(handler)
                    }

                    fn register_sync_with_editor<Handler, Response, Error>(
                        server: &mut #facade_crate::mcp::McpServer,
                        handler: Handler,
                    ) -> Result<(), #facade_crate::mcp::McpToolError>
                    where
                        Self::Form: #facade_crate::mcp::McpEditableForm,
                        <Self::Form as #facade_crate::mcp::McpForm>::ValueHolder:
                            Clone + Default + Send + 'static,
                        Handler: Fn(Self) -> Result<Response, Error> + Send + Sync + 'static,
                        Response: #facade_crate::mcp::Serialize + #facade_crate::mcp::McpJsonSchema,
                        Error: ::core::fmt::Display,
                    {
                        #facade_crate::mcp::form::<Self>(server).model_with_editor(handler)
                    }

                    fn register_async_with_editor<Handler, Fut, Response, Error>(
                        server: &mut #facade_crate::mcp::McpServer,
                        handler: Handler,
                    ) -> Result<(), #facade_crate::mcp::McpToolError>
                    where
                        Self::Form: #facade_crate::mcp::McpEditableForm,
                        <Self::Form as #facade_crate::mcp::McpForm>::ValueHolder:
                            Clone + Default + Send + 'static,
                        Handler: Fn(Self) -> Fut + Send + Sync + 'static,
                        Fut: ::core::future::Future<Output = Result<Response, Error>> + Send + 'static,
                        Response: #facade_crate::mcp::Serialize + #facade_crate::mcp::McpJsonSchema,
                        Error: ::core::fmt::Display,
                    {
                        #facade_crate::mcp::form::<Self>(server).model_async_with_editor(handler)
                    }
                }
            }
        },
        HolderConversionMode::SkippedFields => quote! {},
    };

    Ok(quote! {
        #(#field_descriptor_consts)*

        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        pub const #fields_const_ident: &[#facade_crate::mcp::McpField] = &[
            #(#field_descriptors),*
        ];

        impl #impl_generics #facade_crate::mcp::McpForm
            for #original_ident #ty_generics #where_clause
        {
            type ValueHolder = #holder_ident #ty_generics;

            fn descriptor() -> #facade_crate::mcp::McpFormDescriptor {
                #facade_crate::mcp::McpFormDescriptor::new(
                    stringify!(#original_ident),
                    #facade_crate::schema::registry::RustPath::from_macro_tokens_unchecked(
                        module_path!()
                    ),
                    #fields_const_ident,
                    #enable_koruma,
                    #tool_metadata
                )
            }

            fn decode_arguments(
                call: #facade_crate::mcp::McpToolCall
            ) -> Result<Self::ValueHolder, #facade_crate::mcp::McpToolError> {
                let mut __gpui_form_holder = #holder_ident::default();
                let mut __gpui_form_arguments = call.into_arguments();
                #(#field_decoders)*
                __gpui_form_arguments.finish()?;
                Ok(__gpui_form_holder)
            }

            fn validate_holder(
                holder: &Self::ValueHolder
            ) -> Result<(), #facade_crate::mcp::McpToolError> {
                #validation_tokens
            }

            fn validate_holder_issues(
                holder: &Self::ValueHolder
            ) -> Result<(), ::std::vec::Vec<#facade_crate::mcp::McpValidationIssue>> {
                #validation_issue_tokens
            }
        }

        impl #impl_generics #facade_crate::mcp::McpFormValidation
            for #original_ident #ty_generics #where_clause
        {
            fn validate_holder_issues(
                holder: &Self::ValueHolder
            ) -> Result<(), ::std::vec::Vec<#facade_crate::mcp::McpValidationIssue>> {
                <Self as #facade_crate::mcp::McpForm>::validate_holder_issues(holder)
            }
        }

        #model_impl

        impl #impl_generics #facade_crate::mcp::McpEditableForm
            for #original_ident #ty_generics #where_clause
        {
            fn decode_field(
                holder: &mut Self::ValueHolder,
                field: &str,
                value: #facade_crate::mcp::McpAny,
            ) -> Result<(), #facade_crate::mcp::McpToolError> {
                match field {
                    #(#field_decode_arms),*,
                    _ => Err(#facade_crate::mcp::McpToolError::UnknownField {
                        field: field.to_string(),
                    }),
                }
            }
        }

        impl #impl_generics #facade_crate::mcp::McpSubmitArgument
            for #holder_ident #ty_generics #where_clause
        {
            type Form = #original_ident #ty_generics;

            fn register_sync<Handler, Response, Error>(
                server: &mut #facade_crate::mcp::McpServer,
                handler: Handler,
            ) -> Result<(), #facade_crate::mcp::McpToolError>
            where
                Handler: Fn(Self) -> Result<Response, Error> + Send + Sync + 'static,
                Response: #facade_crate::mcp::Serialize + #facade_crate::mcp::McpJsonSchema,
                Error: ::core::fmt::Display,
            {
                #facade_crate::mcp::form::<
                    <Self as #facade_crate::mcp::McpSubmitArgument>::Form
                >(server).holder(handler)
            }

            fn register_async<Handler, Fut, Response, Error>(
                server: &mut #facade_crate::mcp::McpServer,
                handler: Handler,
            ) -> Result<(), #facade_crate::mcp::McpToolError>
            where
                Handler: Fn(Self) -> Fut + Send + Sync + 'static,
                Fut: ::core::future::Future<Output = Result<Response, Error>> + Send + 'static,
                Response: #facade_crate::mcp::Serialize + #facade_crate::mcp::McpJsonSchema,
                Error: ::core::fmt::Display,
            {
                #facade_crate::mcp::form::<
                    <Self as #facade_crate::mcp::McpSubmitArgument>::Form
                >(server).holder_async(handler)
            }

            fn register_sync_with_editor<Handler, Response, Error>(
                server: &mut #facade_crate::mcp::McpServer,
                handler: Handler,
            ) -> Result<(), #facade_crate::mcp::McpToolError>
            where
                Self::Form: #facade_crate::mcp::McpEditableForm,
                <Self::Form as #facade_crate::mcp::McpForm>::ValueHolder:
                    Clone + Default + Send + 'static,
                Handler: Fn(Self) -> Result<Response, Error> + Send + Sync + 'static,
                Response: #facade_crate::mcp::Serialize + #facade_crate::mcp::McpJsonSchema,
                Error: ::core::fmt::Display,
            {
                #facade_crate::mcp::form::<
                    <Self as #facade_crate::mcp::McpSubmitArgument>::Form
                >(server).holder_with_editor(handler)
            }

            fn register_async_with_editor<Handler, Fut, Response, Error>(
                server: &mut #facade_crate::mcp::McpServer,
                handler: Handler,
            ) -> Result<(), #facade_crate::mcp::McpToolError>
            where
                Self::Form: #facade_crate::mcp::McpEditableForm,
                <Self::Form as #facade_crate::mcp::McpForm>::ValueHolder:
                    Clone + Default + Send + 'static,
                Handler: Fn(Self) -> Fut + Send + Sync + 'static,
                Fut: ::core::future::Future<Output = Result<Response, Error>> + Send + 'static,
                Response: #facade_crate::mcp::Serialize + #facade_crate::mcp::McpJsonSchema,
                Error: ::core::fmt::Display,
            {
                #facade_crate::mcp::form::<
                    <Self as #facade_crate::mcp::McpSubmitArgument>::Form
                >(server).holder_async_with_editor(handler)
            }
        }

        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #descriptor_fn_ident() -> #facade_crate::mcp::McpFormDescriptor {
            <#original_ident #ty_generics as #facade_crate::mcp::McpForm>::descriptor()
        }

        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #editor_register_fn_ident(
            server: &mut #facade_crate::mcp::McpServer
        ) -> Result<(), #facade_crate::mcp::McpToolError> {
            #facade_crate::mcp::register_editor::<#original_ident #ty_generics>(server)
        }

        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #editor_definitions_fn_ident(
        ) -> Result<Vec<#facade_crate::mcp::ToolDefinition>, #facade_crate::mcp::McpToolError> {
            #facade_crate::mcp::editor_tool_definitions::<#original_ident #ty_generics>()
        }

        #facade_crate::mcp::registry::inventory::submit! {
            #facade_crate::mcp::registry::McpSubmitRegistration::new(#descriptor_fn_ident)
        }

        #facade_crate::mcp::registry::inventory::submit! {
            #facade_crate::mcp::registry::McpEditorRegistration::new(
                #descriptor_fn_ident,
                #editor_register_fn_ident,
                #editor_definitions_fn_ident,
            )
        }

        #context_submit_registration
    })
}
