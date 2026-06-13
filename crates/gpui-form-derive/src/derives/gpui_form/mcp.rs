use component_shape_codegen::{McpToolMetadataParts, mcp_tool_metadata_tokens};
use gpui_form_codegen::metadata::rust_type_tokens;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Path, Type};

use crate::derives::gpui_form::attrs::McpToolOptions;
use crate::derives::gpui_form::holder_plan::{HolderConversionMode, ValueHolderPlan};
use crate::derives::gpui_form::ir::{DeriveContext, FieldPlan, HolderFieldIr, HolderStoragePlan};

type McpResult<T> = syn::Result<T>;

fn field_value_presence_tokens(
    context: &DeriveContext,
    storage: &HolderStoragePlan,
) -> TokenStream {
    let facade_crate = &context.paths.gpui_form;
    match storage {
        HolderStoragePlan::OriginallyOptional => {
            quote! {
                #facade_crate::schema::registry::FieldValuePresence::Optional
            }
        },
        HolderStoragePlan::Direct => {
            quote! {
                #facade_crate::schema::registry::FieldValuePresence::DirectStorage
            }
        },
        HolderStoragePlan::ShapePolicy { shape } => {
            let runtime_crate = context.paths.gpui_form_facade_runtime();
            quote! {
                if <<#shape as #runtime_crate::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicy
                    as #runtime_crate::shape::ComponentValueStoragePolicy>::REQUIRES_VALUE
                {
                    #facade_crate::schema::registry::FieldValuePresence::RequiresValue
                } else {
                    #facade_crate::schema::registry::FieldValuePresence::DirectStorage
                }
            }
        },
    }
}

fn shape_present_tokens(context: &DeriveContext, shape: &Path, base_type: &Type) -> TokenStream {
    let runtime_crate = context.paths.gpui_form_facade_runtime();
    quote! {
        <<#shape as #runtime_crate::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicy
            as #runtime_crate::shape::ValueStorage<#base_type>>::present(
                __gpui_form_decoded
            )
    }
}

fn field_decode_tokens(context: &DeriveContext, field: &HolderFieldIr) -> TokenStream {
    let field_name = field.field_name();
    let field_name_str = field.field_name().to_string();
    let base_type = field.form_type();
    let required = !matches!(field.storage(), HolderStoragePlan::OriginallyOptional)
        && field.default_expr().is_none();

    match field.storage() {
        HolderStoragePlan::OriginallyOptional => {
            quote! {
                if let Some(__gpui_form_decoded) =
                    __gpui_form_arguments
                        .take_present_tool_value::<Option<#base_type>>(#field_name_str)?
                {
                    __gpui_form_holder.#field_name = __gpui_form_decoded;
                }
            }
        },
        HolderStoragePlan::Direct => {
            if required {
                quote! {
                    __gpui_form_holder.#field_name =
                        __gpui_form_arguments
                            .take_required_tool_value::<#base_type>(#field_name_str)?;
                }
            } else {
                quote! {
                    if let Some(__gpui_form_decoded) =
                        __gpui_form_arguments
                            .take_present_tool_value::<#base_type>(#field_name_str)?
                    {
                        __gpui_form_holder.#field_name = __gpui_form_decoded;
                    }
                }
            }
        },
        HolderStoragePlan::ShapePolicy { shape } => {
            let shape_present = shape_present_tokens(context, shape, base_type);
            if required {
                quote! {
                    let __gpui_form_decoded =
                        __gpui_form_arguments
                            .take_required_tool_value::<#base_type>(#field_name_str)?;
                    __gpui_form_holder.#field_name = #shape_present;
                }
            } else {
                quote! {
                    if let Some(__gpui_form_decoded) =
                        __gpui_form_arguments
                            .take_present_tool_value::<#base_type>(#field_name_str)?
                    {
                        __gpui_form_holder.#field_name = #shape_present;
                    }
                }
            }
        },
    }
}

fn field_decode_arm_tokens(context: &DeriveContext, field: &HolderFieldIr) -> TokenStream {
    let facade_crate = &context.paths.gpui_form;
    let field_name_str = field.field_name().to_string();
    let field_decode = field_decode_tokens(context, field);

    quote! {
        #field_name_str => {
            let mut __gpui_form_values = #facade_crate::mcp::serde_json::Map::new();
            __gpui_form_values.insert(field.to_string(), value.into_value());
            let mut __gpui_form_arguments =
                #facade_crate::mcp::McpArguments::new(__gpui_form_values);
            let __gpui_form_holder = holder;
            #field_decode
            __gpui_form_arguments.finish()?;
            Ok(())
        }
    }
}

fn field_descriptor_tokens(context: &DeriveContext, field: &FieldPlan) -> Option<TokenStream> {
    let shared = field.shared()?;
    let facade_crate = &context.paths.gpui_form;
    let field_name_str = shared.field_name().to_string();
    let form_type = shared.form_type();
    let value_type_tokens = rust_type_tokens(facade_crate, form_type);
    let value_presence_tokens = field_value_presence_tokens(context, shared.storage());
    let has_default = shared.default_expr().is_some();
    let component_shape = field.component().map(|component| component.shape());
    let field_descriptor_constructor = if let Some(shape) = component_shape {
        quote! {
            #facade_crate::mcp::McpField::component::<#form_type, #shape>(
                #field_name_str,
                #value_type_tokens,
                #value_presence_tokens
            )
        }
    } else {
        quote! {
            #facade_crate::mcp::McpField::typed::<#form_type>(
                #field_name_str,
                #value_type_tokens,
                #value_presence_tokens
            )
        }
    };

    Some(quote! {
        #field_descriptor_constructor
        .with_default(#has_default)
    })
}

fn tool_metadata_tokens(
    context: &DeriveContext,
    original_input: &DeriveInput,
    options: Option<&McpToolOptions>,
) -> McpResult<TokenStream> {
    let facade_crate = &context.paths.gpui_form;
    let mcp_crate: Path = syn::parse_quote!(#facade_crate::mcp);
    mcp_tool_metadata_tokens(
        &mcp_crate,
        &original_input.attrs,
        McpToolMetadataParts {
            name: options.and_then(|options| options.name.as_deref()),
            title: options.and_then(|options| options.title.as_deref()),
            description: options.and_then(|options| options.description.as_deref()),
        },
        original_input.ident.span(),
    )
}

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
    let (impl_generics, ty_generics, where_clause) = original_input.generics.split_for_impl();
    let tool_metadata = tool_metadata_tokens(context, original_input, mcp_tool_options)?;

    let field_descriptors: Vec<TokenStream> = field_plans
        .iter()
        .filter_map(|field| field_descriptor_tokens(context, field))
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

    let validation_tokens = if enable_koruma {
        quote! {
            holder.validate().map_err(|error| {
                #facade_crate::mcp::McpToolError::validation(format!("{error:?}"))
            })
        }
    } else {
        quote! {
            let _ = holder;
            Ok(())
        }
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
                        Response: #facade_crate::mcp::Serialize,
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
                        Response: #facade_crate::mcp::Serialize,
                        Error: ::core::fmt::Display,
                    {
                        #facade_crate::mcp::form::<Self>(server).model_async(handler)
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
                            #facade_crate::mcp::McpToolError::conversion(error.to_string())
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
                        Response: #facade_crate::mcp::Serialize,
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
                        Response: #facade_crate::mcp::Serialize,
                        Error: ::core::fmt::Display,
                    {
                        #facade_crate::mcp::form::<Self>(server).model_async(handler)
                    }
                }
            }
        },
        HolderConversionMode::SkippedFields => quote! {},
    };

    Ok(quote! {
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
                Response: #facade_crate::mcp::Serialize,
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
                Response: #facade_crate::mcp::Serialize,
                Error: ::core::fmt::Display,
            {
                #facade_crate::mcp::form::<
                    <Self as #facade_crate::mcp::McpSubmitArgument>::Form
                >(server).holder_async(handler)
            }
        }

        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #descriptor_fn_ident() -> #facade_crate::mcp::McpFormDescriptor {
            <#original_ident #ty_generics as #facade_crate::mcp::McpForm>::descriptor()
        }

        #facade_crate::mcp::registry::inventory::submit! {
            #facade_crate::mcp::registry::McpSubmitRegistration::new(#descriptor_fn_ident)
        }
    })
}
