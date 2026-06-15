use component_shape_codegen::{McpToolMetadataParts, mcp_tool_metadata_tokens};
use gpui_form_codegen::metadata::rust_type_tokens;
use koruma_derive_core::{ParsedValidatorUse, ValidatorTargetSelector, ValidatorTypeArg};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, Expr, Lit, LitStr, Path, Type, UnOp, spanned::Spanned as _};

use crate::derives::gpui_form::attrs::{
    McpContextSubmitOptions, McpIconThemeOption, McpTaskSupportOption, McpToolOptions,
};
use crate::derives::gpui_form::holder_plan::{HolderConversionMode, ValueHolderPlan};
use crate::derives::gpui_form::ir::{DeriveContext, FieldPlan, HolderFieldIr, HolderStoragePlan};

type McpResult<T> = syn::Result<T>;

struct FieldDescriptorTokens {
    constants: Vec<TokenStream>,
    descriptor: TokenStream,
}

struct ValidationRuleTokenPlan {
    param_const: Option<TokenStream>,
    rule: TokenStream,
}

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

fn field_descriptor_tokens(
    context: &DeriveContext,
    form_ident: &syn::Ident,
    field_index: usize,
    field: &FieldPlan,
) -> Option<FieldDescriptorTokens> {
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
    let label_tokens = shared.metadata.label.as_ref().map(|label| {
        let label = LitStr::new(label, shared.field_name().span());
        quote! { .with_label(#label) }
    });
    let description_tokens = shared.metadata.description.as_ref().map(|description| {
        let description = LitStr::new(description, shared.field_name().span());
        quote! { .with_description(#description) }
    });
    let examples_tokens = (!shared.metadata.examples.is_empty()).then(|| {
        let examples = shared
            .metadata
            .examples
            .iter()
            .map(|example| LitStr::new(example, shared.field_name().span()))
            .collect::<Vec<_>>();
        quote! { .with_examples(&[#(#examples),*]) }
    });
    let (validation_rule_consts, validation_rules_tokens) =
        validation_rules_tokens(context, form_ident, field_index, shared);

    let descriptor = quote! {
        #field_descriptor_constructor
        .with_default(#has_default)
        #label_tokens
        #description_tokens
        #examples_tokens
        #validation_rules_tokens
    };

    Some(FieldDescriptorTokens {
        constants: validation_rule_consts,
        descriptor,
    })
}

fn validation_rules_tokens(
    context: &DeriveContext,
    form_ident: &syn::Ident,
    field_index: usize,
    field: &HolderFieldIr,
) -> (Vec<TokenStream>, Option<TokenStream>) {
    let facade_crate = &context.paths.gpui_form;
    let rules_const_ident = field_validation_rules_const_ident(form_ident, field_index);
    let mut consts = Vec::new();
    let mut rules = Vec::new();
    let mut rule_index = 0_usize;

    for validator in &field.validation.field_validators {
        let plan = validation_rule_token_plan(
            context,
            quote! { #facade_crate::mcp::McpValidationScope::Field },
            validator,
            form_ident,
            field_index,
            rule_index,
        );
        if let Some(param_const) = plan.param_const {
            consts.push(param_const);
        }
        rules.push(plan.rule);
        rule_index += 1;
    }

    for validator in &field.validation.element_validators {
        let plan = validation_rule_token_plan(
            context,
            quote! { #facade_crate::mcp::McpValidationScope::Element },
            validator,
            form_ident,
            field_index,
            rule_index,
        );
        if let Some(param_const) = plan.param_const {
            consts.push(param_const);
        }
        rules.push(plan.rule);
        rule_index += 1;
    }

    if rules.is_empty() {
        return (consts, None);
    }

    consts.push(quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        const #rules_const_ident: &[#facade_crate::mcp::McpValidationRule] = &[
            #(#rules),*
        ];
    });

    (
        consts,
        Some(quote! {
            .with_validation_rules(#rules_const_ident)
        }),
    )
}

fn validation_rule_token_plan(
    context: &DeriveContext,
    scope_tokens: TokenStream,
    validator_use: &ParsedValidatorUse,
    form_ident: &syn::Ident,
    field_index: usize,
    rule_index: usize,
) -> ValidationRuleTokenPlan {
    let facade_crate = &context.paths.gpui_form;
    let validator = validator_use.validator();
    let validator_name = LitStr::new(&validator.name().to_string(), validator.name().span());
    let validator_path = LitStr::new(&validator.path_name(), validator.path().span());
    let label_tokens = validator_use
        .label()
        .map(|label| {
            let label = LitStr::new(&label.to_string(), validator_use.source_span());
            quote! { Some(#label) }
        })
        .unwrap_or_else(|| quote! { None });
    let target_tokens = match validator_use.target() {
        ValidatorTargetSelector::Default => {
            quote! { #facade_crate::mcp::McpValidationTarget::Default }
        },
        ValidatorTargetSelector::Full { .. } => {
            quote! { #facade_crate::mcp::McpValidationTarget::Full }
        },
        ValidatorTargetSelector::Unwrapped { .. } => {
            quote! { #facade_crate::mcp::McpValidationTarget::Unwrapped }
        },
    };
    let type_arg_tokens = match validator.type_arg() {
        ValidatorTypeArg::None => quote! { #facade_crate::mcp::McpValidationTypeArgMode::None },
        ValidatorTypeArg::Infer => quote! { #facade_crate::mcp::McpValidationTypeArgMode::Infer },
        ValidatorTypeArg::Explicit(_) => {
            quote! { #facade_crate::mcp::McpValidationTypeArgMode::Explicit }
        },
    };
    let params = validator
        .setter_calls()
        .iter()
        .flat_map(|call| {
            let method_name = call.method().to_string();
            let arg_count = call.args().len();
            call.args()
                .iter()
                .enumerate()
                .map(move |(index, arg)| {
                    let param_name = if arg_count == 1 {
                        method_name.clone()
                    } else {
                        format!("{method_name}[{index}]")
                    };
                    let param_name = LitStr::new(&param_name, call.method().span());
                    let expr = arg.as_expr();
                    if let Some(literal) = literal_expr_string(expr) {
                        let literal = LitStr::new(&literal, expr.span());
                        quote! {
                            #facade_crate::mcp::McpValidationParam::literal(
                                #param_name,
                                #literal
                            )
                        }
                    } else {
                        let expr = LitStr::new(&expr.to_token_stream().to_string(), expr.span());
                        quote! {
                            #facade_crate::mcp::McpValidationParam::expr(
                                #param_name,
                                #expr
                            )
                        }
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let params_ident = field_validation_params_const_ident(form_ident, field_index, rule_index);
    let (param_const, params_tokens) = if params.is_empty() {
        (
            None,
            quote! { #facade_crate::mcp::MCP_VALIDATION_PARAMS_NONE },
        )
    } else {
        (
            Some(quote! {
                #[doc(hidden)]
                #[allow(non_upper_case_globals)]
                const #params_ident: &[#facade_crate::mcp::McpValidationParam] = &[
                    #(#params),*
                ];
            }),
            quote! { #params_ident },
        )
    };

    let rule = quote! {
        #facade_crate::mcp::McpValidationRule::new(
            #scope_tokens,
            #validator_name,
            #validator_path,
            #label_tokens,
            #type_arg_tokens,
            #params_tokens,
        )
        .with_target(#target_tokens)
    };

    ValidationRuleTokenPlan { param_const, rule }
}

fn field_validation_rules_const_ident(form_ident: &syn::Ident, field_index: usize) -> syn::Ident {
    format_ident!("__{}GpuiFormMcpValidationRules{field_index}", form_ident)
}

fn field_validation_params_const_ident(
    form_ident: &syn::Ident,
    field_index: usize,
    rule_index: usize,
) -> syn::Ident {
    format_ident!(
        "__{}GpuiFormMcpValidationParams{field_index}_{rule_index}",
        form_ident
    )
}

fn literal_expr_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(lit) => literal_string(&lit.lit),
        Expr::Unary(unary)
            if matches!(unary.op, UnOp::Neg(_))
                && matches!(
                    unary.expr.as_ref(),
                    Expr::Lit(expr_lit)
                        if matches!(expr_lit.lit, Lit::Int(_) | Lit::Float(_))
                ) =>
        {
            let Expr::Lit(lit) = unary.expr.as_ref() else {
                return None;
            };
            literal_string(&lit.lit).map(|literal| format!("-{literal}"))
        },
        _ => None,
    }
}

fn literal_string(lit: &Lit) -> Option<String> {
    match lit {
        Lit::Int(lit) => Some(lit.base10_digits().to_string()),
        Lit::Float(lit) => Some(lit.base10_digits().to_string()),
        Lit::Bool(lit) => Some(lit.value.to_string()),
        Lit::Str(lit) => Some(lit.value()),
        _ => None,
    }
}

fn field_validation_issue_tokens(
    context: &DeriveContext,
    form_ident: &syn::Ident,
    field_index: usize,
    field: &HolderFieldIr,
) -> TokenStream {
    if field.validation.field_validators.is_empty()
        && field.validation.element_validators.is_empty()
    {
        return quote! {};
    }

    let facade_crate = &context.paths.gpui_form;
    let field_ident = field.field_name();
    let field_name = field.field_name().to_string();
    let rules_const_ident = field_validation_rules_const_ident(form_ident, field_index);
    let field_issues = field
        .validation
        .field_validators
        .iter()
        .enumerate()
        .map(|(rule_index, validator)| {
            let getter = validator_getter_ident(validator);
            quote! {
                if let Some(__gpui_form_validator) = __gpui_form_field_error.#getter() {
                    __gpui_form_issues.push(
                        #facade_crate::mcp::McpValidationIssue::for_rule(
                            #field_name,
                            #rules_const_ident[#rule_index],
                            ::std::format!("{__gpui_form_validator:?}")
                        )
                    );
                }
            }
        })
        .collect::<Vec<_>>();
    let element_issues = field
        .validation
        .element_validators
        .iter()
        .enumerate()
        .map(|(element_rule_index, validator)| {
            let rule_index = field.validation.field_validators.len() + element_rule_index;
            let getter = validator_getter_ident(validator);
            quote! {
                if let Some(__gpui_form_validator) = __gpui_form_element_error.#getter() {
                    __gpui_form_issues.push(
                        #facade_crate::mcp::McpValidationIssue::for_rule(
                            #field_name,
                            #rules_const_ident[#rule_index],
                            ::std::format!("{__gpui_form_validator:?}")
                        )
                        .with_element_index(*__gpui_form_index)
                    );
                }
            }
        })
        .collect::<Vec<_>>();
    let element_loop = if element_issues.is_empty() {
        quote! {}
    } else {
        quote! {
            for (__gpui_form_index, __gpui_form_element_error) in
                __gpui_form_field_error.element_errors()
            {
                #(#element_issues)*
            }
        }
    };

    quote! {
        let __gpui_form_field_error = __gpui_form_validation_error.#field_ident();
        #(#field_issues)*
        #element_loop
    }
}

fn validator_getter_ident(validator_use: &ParsedValidatorUse) -> syn::Ident {
    if let Some(label) = validator_use.label() {
        return syn::Ident::new(
            &label.to_string(),
            validator_use
                .label_span()
                .unwrap_or_else(|| validator_use.source_span()),
        );
    }

    syn::Ident::new(
        &upper_camel_to_snake(&validator_use.validator().name().to_string()),
        validator_use.source_span(),
    )
}

fn upper_camel_to_snake(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut previous_was_lower_or_digit = false;
    for ch in input.chars() {
        if ch.is_ascii_uppercase() {
            if previous_was_lower_or_digit {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            previous_was_lower_or_digit = false;
        } else {
            output.push(ch);
            previous_was_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
        }
    }
    output
}

fn tool_metadata_tokens(
    context: &DeriveContext,
    original_input: &DeriveInput,
    options: Option<&McpToolOptions>,
) -> McpResult<TokenStream> {
    let facade_crate = &context.paths.gpui_form;
    let mcp_crate: Path = syn::parse_quote!(#facade_crate::mcp);
    let span = original_input.ident.span();
    let mut tokens = mcp_tool_metadata_tokens(
        &mcp_crate,
        &original_input.attrs,
        McpToolMetadataParts {
            name: options.and_then(|options| options.name.as_deref()),
            title: options.and_then(|options| options.title.as_deref()),
            description: options.and_then(|options| options.description.as_deref()),
            read_only: options.and_then(|options| options.read_only),
            destructive: options.and_then(|options| options.destructive),
            idempotent: options.and_then(|options| options.idempotent),
            open_world: options.and_then(|options| options.open_world),
        },
        span,
    )?;

    if let Some(options) = options {
        if !options.icons.is_empty() {
            let icons = options
                .icons
                .iter()
                .map(|icon| icon_tokens(&mcp_crate, icon, span))
                .collect::<Vec<_>>();
            tokens = quote! {{
                const MCP_TOOL_ICONS: &[#mcp_crate::McpToolIcon] = &[#(#icons),*];
                (#tokens).with_icons(MCP_TOOL_ICONS)
            }};
        }
        if let Some(task_support) = options.task_support {
            let task_support = task_support_tokens(&mcp_crate, task_support);
            tokens = quote! { (#tokens).with_task_support(#task_support) };
        }
    }

    Ok(tokens)
}

fn icon_tokens(
    mcp_crate: &Path,
    icon: &crate::derives::gpui_form::attrs::McpIconOptions,
    span: Span,
) -> TokenStream {
    let src = LitStr::new(&icon.src, span);
    let mut tokens = quote! { #mcp_crate::McpToolIcon::new(#src) };
    if let Some(mime_type) = icon.mime_type.as_deref() {
        let mime_type = LitStr::new(mime_type, span);
        tokens = quote! { #tokens.with_mime_type(#mime_type) };
    }
    if !icon.sizes.is_empty() {
        let sizes = icon
            .sizes
            .iter()
            .map(|size| LitStr::new(size, span))
            .collect::<Vec<_>>();
        tokens = quote! { #tokens.with_sizes(&[#(#sizes),*]) };
    }
    if let Some(theme) = icon.theme {
        let theme = match theme {
            McpIconThemeOption::Light => quote! { #mcp_crate::McpIconTheme::Light },
            McpIconThemeOption::Dark => quote! { #mcp_crate::McpIconTheme::Dark },
        };
        tokens = quote! { #tokens.with_theme(#theme) };
    }
    tokens
}

fn task_support_tokens(mcp_crate: &Path, task_support: McpTaskSupportOption) -> TokenStream {
    match task_support {
        McpTaskSupportOption::Forbidden => quote! { #mcp_crate::McpToolTaskSupport::Forbidden },
        McpTaskSupportOption::Optional => quote! { #mcp_crate::McpToolTaskSupport::Optional },
        McpTaskSupportOption::Required => quote! { #mcp_crate::McpToolTaskSupport::Required },
    }
}

fn context_submit_registration_tokens(
    context: &DeriveContext,
    original_input: &DeriveInput,
    context_submit: Option<&McpContextSubmitOptions>,
) -> TokenStream {
    let Some(context_submit) = context_submit else {
        return quote! {};
    };

    let facade_crate = &context.paths.gpui_form;
    let original_ident = &original_input.ident;
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
    let context_submit_impl = context_submit
        .submit
        .as_ref()
        .map(|submit| {
            let submit_body = if let Some(map_response) = context_submit.map_response.as_ref() {
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
