use super::validation::validation_rules_tokens;
use super::*;

pub(super) struct FieldDescriptorTokens {
    pub(super) constants: Vec<TokenStream>,
    pub(super) descriptor: TokenStream,
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

pub(super) fn field_decode_tokens(context: &DeriveContext, field: &HolderFieldIr) -> TokenStream {
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

pub(super) fn field_decode_arm_tokens(
    context: &DeriveContext,
    field: &HolderFieldIr,
) -> TokenStream {
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

pub(super) fn field_descriptor_tokens(
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
