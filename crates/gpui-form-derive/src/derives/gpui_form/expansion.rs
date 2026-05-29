use darling::FromDeriveInput as _;
use gpui_form_codegen::{CratePaths, components::RequiredValue};
use itertools::Itertools as _;
use koruma_derive_core::{FieldInfo as KorumaFieldInfo, ParseFieldResult};
use proc_macro2::TokenStream;
use quote::{ToTokens as _, format_ident, quote};
use std::collections::HashMap;
use syn::DeriveInput;
use syn::GenericParam;

use crate::derives::gpui_form::cfg_attr::flatten_cfg_attr_in_derive_input;
use crate::derives::gpui_form::components::generate_component_field;
use crate::derives::gpui_form::structs::{ComponentStruct, FieldOptionality, GpuiFormOptions};
use crate::derives::gpui_form::utils::extract_option_inner_type;
use crate::derives::gpui_form::value_holder::{
    generate_value_holder, holder_conversion_can_fail_metadata_tokens, parse_field_default,
};

fn option_expr_string_tokens(expr: &Option<syn::Expr>) -> TokenStream {
    match expr {
        Some(expr) => {
            let expr_str = expr.to_token_stream().to_string();
            let facade_crate = CratePaths::resolve().gpui_form;
            quote! {
                Some(#facade_crate::schema::registry::RustExpr::new_unchecked(#expr_str))
            }
        },
        None => quote! { None },
    }
}

fn phantom_type_tokens(generics: &syn::Generics) -> TokenStream {
    let params: Vec<TokenStream> = generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(param) => {
                let ident = &param.ident;
                Some(quote! { #ident })
            },
            GenericParam::Lifetime(param) => {
                let lifetime = &param.lifetime;
                Some(quote! { &#lifetime () })
            },
            GenericParam::Const(_) => None,
        })
        .collect();

    if params.is_empty() {
        quote! { () }
    } else {
        quote! { (#(#params),*) }
    }
}

fn marker_field_tokens(generics: &syn::Generics) -> TokenStream {
    if generics
        .params
        .iter()
        .any(|param| matches!(param, GenericParam::Type(_) | GenericParam::Lifetime(_)))
    {
        let phantom_type = phantom_type_tokens(generics);
        quote! {
            #[doc(hidden)]
            pub __gpui_form_marker: ::core::marker::PhantomData<fn() -> #phantom_type>,
        }
    } else {
        quote! {}
    }
}

pub fn expand_gpui_form(
    derive_input: DeriveInput,
    options: GpuiFormOptions,
) -> proc_macro2::TokenStream {
    let facade_crate = CratePaths::resolve().gpui_form;
    let original_input = derive_input.clone();
    let derive_input = flatten_cfg_attr_in_derive_input(derive_input);

    let parsed = match ComponentStruct::from_derive_input(&derive_input) {
        Ok(parsed) => parsed,
        Err(e) => return e.write_errors(),
    };

    let struct_name = &parsed.ident;
    let components_holder_name = format_ident!("{}FormFields", struct_name);
    let components_base_declarations_name = format_ident!("{}FormComponents", struct_name);
    let (impl_generics, ty_generics, where_clause) = original_input.generics.split_for_impl();
    let declaration_generics = &original_input.generics;
    let form_fields_marker = marker_field_tokens(&original_input.generics);

    let koruma_options = parsed.koruma.as_ref().map(|k| k.0.clone());

    if parsed.empty.is_some() {
        let enable_koruma = koruma_options.is_some();
        let enable_koruma_fluent = koruma_options.as_ref().map(|k| k.fluent).unwrap_or(false);
        let empty_fields: Vec<FieldOptionality> = Vec::new();
        let (value_holder_tokens, _) = generate_value_holder(
            &original_input,
            &empty_fields,
            enable_koruma,
            enable_koruma_fluent,
        );
        let shape_impl = if options.generate_shape && original_input.generics.params.is_empty() {
            quote! {
                #facade_crate::schema::registry::inventory::submit! {
                    #facade_crate::schema::registry::GpuiFormShape::new(
                        stringify!(#struct_name),
                        &[],
                        file!(),
                        #enable_koruma
                    ).with_skipped_fields(false)
                    .with_holder_conversion_can_fail(false)
                }
            }
        } else {
            quote! {}
        };
        let empty_struct_declarations = if original_input.generics.params.is_empty() {
            quote! {
                pub struct #components_holder_name;
                pub struct #components_base_declarations_name;
            }
        } else {
            quote! {
                pub struct #components_holder_name #declaration_generics #where_clause {
                    #form_fields_marker
                }

                pub struct #components_base_declarations_name #declaration_generics #where_clause {
                    #form_fields_marker
                }
            }
        };

        return quote! {
            #value_holder_tokens

            #shape_impl

            #empty_struct_declarations
        };
    }

    let fields_iter = match &parsed.data {
        darling::ast::Data::Struct(s) => &s.fields,
        _ => unreachable!("GpuiForm derive only supports named structs"),
    };

    let has_skipped_fields = fields_iter.iter().any(|field| field.skip());

    let parsed_koruma_fields: HashMap<String, KorumaFieldInfo> = match &derive_input.data {
        syn::Data::Struct(data_struct) => {
            let mut fields = HashMap::new();
            for field in &data_struct.fields {
                let Some(ident) = field.ident.as_ref() else {
                    continue;
                };
                match koruma_derive_core::parse_field(field, 0) {
                    ParseFieldResult::Valid(info) => {
                        fields.insert(ident.to_string(), *info);
                    },
                    ParseFieldResult::Skip => {},
                    ParseFieldResult::Error(error) => return error.to_compile_error(),
                }
            }
            fields
        },
        _ => HashMap::new(),
    };

    let enable_koruma = koruma_options.is_some() || !parsed_koruma_fields.is_empty();
    let enable_koruma_fluent = koruma_options.as_ref().map(|k| k.fluent).unwrap_or(false);

    let koruma_validations: HashMap<String, Vec<String>> = parsed_koruma_fields
        .iter()
        .map(|(name, info)| {
            let mut validator_names: Vec<String> = info
                .validation
                .field_validators
                .iter()
                .map(|v| v.name().to_string())
                .collect();

            if info.is_newtype() {
                validator_names.push("NewtypeValidation".to_string());
            }

            if info.is_nested() {
                validator_names.push("NestedValidation".to_string());
            }

            (name.clone(), validator_names)
        })
        .collect();

    if fields_iter.is_empty() {
        return syn::Error::new_spanned(
            &derive_input,
            format!(
                "Struct `{}` has no fields. Add `#[gpui_form(empty)]` attribute to explicitly mark it as an empty form.",
                struct_name
            ),
        )
        .to_compile_error();
    }

    if let Some(error) = fields_iter.iter().find_map(|field| {
        let rendered = field.rendered()?;
        let component_def = rendered.component?;
        let (_, original_inner_type) = extract_option_inner_type(&field.ty);
        let base_type = rendered
            .r#type
            .map(|ty| extract_option_inner_type(&ty.0).1)
            .unwrap_or(original_inner_type);

        component_def.validate_field_type(&base_type).err()
    }) {
        return error.to_compile_error();
    }

    let component_field_pairs: Vec<crate::derives::gpui_form::structs::ComponentFieldContent> =
        fields_iter
            .iter()
            .filter(|field| !field.skip())
            .map(generate_component_field)
            .collect();

    let (field_structure_tokens, field_base_declarations_tokens, required_value_map): (
        Vec<TokenStream>,
        Vec<TokenStream>,
        HashMap<String, RequiredValue>,
    ) = component_field_pairs
        .into_iter()
        .map(|content| {
            (
                content.field_structure_tokens,
                content.field_base_declarations_tokens,
                content.required_value,
            )
        })
        .multiunzip();

    let mut field_optionality = Vec::new();
    for field in fields_iter {
        let field_name = field.ident.clone().unwrap();
        let field_name_str = field_name.to_string();
        let (was_optional, inner_type) = extract_option_inner_type(&field.ty);
        let rendered = field.rendered();
        let required_value = if rendered
            .as_ref()
            .is_some_and(|rendered| rendered.component.is_some())
            && !was_optional
        {
            required_value_map
                .get(&field_name_str)
                .cloned()
                .unwrap_or_else(|| RequiredValue::explicit(false))
        } else {
            RequiredValue::explicit(false)
        };
        let koruma_info = parsed_koruma_fields.get(&field_name_str);
        let validation = koruma_info
            .map(|info| info.validation.clone())
            .unwrap_or_default();
        let default_expr = parse_field_default(field);
        field_optionality.push(FieldOptionality {
            field_name,
            original_type: field.ty.clone(),
            inner_type,
            was_optional,
            required_value,
            validation,
            default_expr,
            override_type: rendered
                .as_ref()
                .and_then(|rendered| rendered.r#type.map(|ty| ty.0.clone())),
            into_expr: rendered
                .as_ref()
                .and_then(|rendered| rendered.into.cloned()),
            from_expr: rendered
                .as_ref()
                .and_then(|rendered| rendered.from.cloned()),
            skip: field.skip(),
        });
    }

    let has_fields_needing_required = field_optionality
        .iter()
        .any(|f| f.needs_required_validation() && !f.validation.is_newtype);

    let has_any_koruma_validations = field_optionality.iter().any(|f| {
        !f.skip
            && (!f.validation.field_validators.is_empty()
                || !f.validation.element_validators.is_empty()
                || f.validation.is_nested)
    });

    let effective_enable_koruma =
        enable_koruma || (has_fields_needing_required && has_any_koruma_validations);
    let (value_holder_tokens, fields_requiring_required) = generate_value_holder(
        &original_input,
        &field_optionality,
        effective_enable_koruma,
        enable_koruma_fluent,
    );
    let conversion_can_fail = holder_conversion_can_fail_metadata_tokens(&field_optionality);

    let field_variant_construction_code: Vec<TokenStream> = fields_iter
        .iter()
        .filter_map(|field| {
            if field.skip() {
                None
            } else if let Some(rendered) = field.rendered()
                && let Some(component_def) = rendered.component
            {
                let field_name_str = field
                    .ident
                    .as_ref()
                    .expect("Field should have an ident if not skipped and has component")
                    .to_string();
                let (was_optional, original_inner_type) = extract_option_inner_type(&field.ty);
                let base_type = field
                    .rendered()
                    .and_then(|rendered| rendered.r#type)
                    .map(|ty| extract_option_inner_type(&ty.0).1)
                    .unwrap_or_else(|| original_inner_type.clone());
                let component_required_value = required_value_map
                    .get(&field_name_str)
                    .cloned()
                    .unwrap_or_else(|| RequiredValue::explicit(false));
                let required_value = if was_optional {
                    RequiredValue::explicit(false)
                } else {
                    component_required_value
                };
                let requires_value_tokens = required_value.metadata_tokens();

                let is_optional = was_optional;
                let field_type_str = base_type.to_token_stream().to_string();
                let source_value_type_str = original_inner_type.to_token_stream().to_string();
                let mut validation_rules = koruma_validations
                    .get(&field_name_str)
                    .cloned()
                    .unwrap_or_default();

                let needs_inferred_required_rule = fields_requiring_required
                    .contains(&field_name_str)
                    && !validation_rules.contains(&"RequiredValidation".to_string());

                if needs_inferred_required_rule
                    && !matches!(required_value, RequiredValue::Shape(_))
                {
                    validation_rules.insert(0, "RequiredValidation".to_string());
                }

                let validation_literals: Vec<_> = validation_rules
                    .iter()
                    .map(|v| syn::LitStr::new(v, proc_macro2::Span::call_site()))
                    .collect();
                let validation_rules_tokens = if needs_inferred_required_rule
                    && let RequiredValue::Shape(shape) = &required_value
                {
                    let runtime_crate = CratePaths::resolve().gpui_form_facade_runtime();
                    quote! {
                        {
                            const __GPUI_FORM_FIELD_VALIDATIONS: &[&str] =
                                if <<#shape as #runtime_crate::shape::ComponentShape>::RequiredValuePolicy
                                    as #runtime_crate::shape::ComponentRequiredValuePolicy>::REQUIRES_VALUE
                                {
                                    &[
                                        "RequiredValidation",
                                        #( #validation_literals ),*
                                    ]
                                } else {
                                    &[
                                        #( #validation_literals ),*
                                    ]
                                };
                            __GPUI_FORM_FIELD_VALIDATIONS
                        }
                    }
                } else {
                    quote! {
                        &[
                            #( #validation_literals ),*
                        ]
                    }
                };

                let default_expr_tokens = rendered.default.map(|expr| {
                    let expr_str = expr.0.to_token_stream().to_string();
                    quote! {
                        .with_default(#facade_crate::schema::registry::RustExpr::new_unchecked(#expr_str))
                    }
                });

                let shape_path_tokens = component_def.shape_path_tokens(&base_type);
                let component_type_tokens = component_def.component_type_tokens(&base_type);
                let value_binding_tokens = component_def.value_binding_tokens(&base_type);
                let prototyping_tokens = component_def.prototyping_tokens(&base_type, &field_name_str);
                let from_expr_tokens = option_expr_string_tokens(&rendered.from.cloned());
                let into_expr_tokens = option_expr_string_tokens(&rendered.into.cloned());

                Some(quote! {
                    #facade_crate::schema::registry::FieldVariant::new(
                        #field_name_str,
                        #facade_crate::schema::registry::RustType::new_unchecked(#field_type_str),
                        #is_optional
                    )
                    .with_source_value_type(
                        #facade_crate::schema::registry::RustType::new_unchecked(#source_value_type_str)
                    )
                    .with_requires_value(#requires_value_tokens)
                    .with_conversions(#from_expr_tokens, #into_expr_tokens)
                    .with_validations(#validation_rules_tokens)
                    #default_expr_tokens
                    #shape_path_tokens
                    #component_type_tokens
                    #value_binding_tokens
                    #prototyping_tokens
                })
            } else {
                None
            }
        })
        .collect();

    let component_type_check_tokens: Vec<TokenStream> = fields_iter
        .iter()
        .filter_map(|field| {
            if field.skip() {
                return None;
            }

            let rendered = field.rendered()?;
            let component_def = rendered.component?;
            let (_, original_inner_type) = extract_option_inner_type(&field.ty);
            let base_type = field
                .rendered()
                .and_then(|rendered| rendered.r#type)
                .map(|ty| extract_option_inner_type(&ty.0).1)
                .unwrap_or(original_inner_type);

            Some(component_def.type_check_tokens(&base_type))
        })
        .collect();

    let component_type_checks = if component_type_check_tokens.is_empty() {
        quote! {}
    } else {
        quote! {
            impl #impl_generics #components_holder_name #ty_generics #where_clause {
                #[allow(dead_code)]
                fn __gpui_form_component_type_checks() {
                    #(#component_type_check_tokens)*
                }
            }
        }
    };

    let shape_impl = if options.generate_shape && original_input.generics.params.is_empty() {
        quote! {
            #facade_crate::schema::registry::inventory::submit! {
                #facade_crate::schema::registry::GpuiFormShape::new(
                    stringify!(#struct_name),
                    &[
                        #(#field_variant_construction_code),*
                    ],
                    file!(),
                    #effective_enable_koruma
                ).with_skipped_fields(#has_skipped_fields)
                .with_holder_conversion_can_fail(#conversion_can_fail)
            }
        }
    } else {
        quote! {}
    };

    let components_base_declarations = if original_input.generics.params.is_empty() {
        quote! {
            pub struct #components_base_declarations_name;

            impl #components_base_declarations_name {
              #(#field_base_declarations_tokens)*
            }
        }
    } else {
        let phantom_type = phantom_type_tokens(&original_input.generics);
        quote! {
            pub struct #components_base_declarations_name #declaration_generics #where_clause {
                #[doc(hidden)]
                pub __gpui_form_marker: ::core::marker::PhantomData<fn() -> #phantom_type>,
            }

            impl #impl_generics #components_base_declarations_name #ty_generics #where_clause {
              #(#field_base_declarations_tokens)*
            }
        }
    };

    let expanded = quote! {
        #value_holder_tokens

        pub struct #components_holder_name #declaration_generics #where_clause {
            #(#field_structure_tokens)*
            #form_fields_marker
        }

        #component_type_checks

        #shape_impl

        #components_base_declarations
    };

    expanded
}
