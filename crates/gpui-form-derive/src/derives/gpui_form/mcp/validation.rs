use super::*;

struct ValidationRuleTokenPlan {
    param_const: Option<TokenStream>,
    rule: TokenStream,
}

pub(super) fn validation_rules_tokens(
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

pub(super) fn field_validation_issue_tokens(
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
