use std::collections::HashMap;

use darling::{FromDeriveInput, FromField, ast};
use gpui_form_core::components::*;
use gpui_form_core::implementations::ComponentLayout as _;
use itertools::Itertools as _;
use proc_macro2::TokenStream;
use quote::{ToTokens as _, format_ident, quote};
use syn::{DeriveInput, GenericArgument, Ident, PathArguments, Type, parse_macro_input};

#[derive(Debug, FromField)]
#[darling(attributes(gpui_form))]
struct ComponentField {
    pub ident: Option<Ident>,
    pub ty: Type,
    #[darling(default)]
    pub component: Option<Components>,
    #[darling(default)]
    skip: bool,
}

impl ComponentField {
    pub fn skip(&self) -> bool {
        self.skip && self.component.is_none()
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(gpui_form), supports(struct_named))]
struct ComponentStruct {
    pub ident: Ident,
    pub data: ast::Data<(), ComponentField>,
}

fn get_components_behaviour_tokens(component: &Components) -> TokenStream {
    match component {
        Components::Input => {
            quote! { ::gpui_form::core::components::ComponentsBehaviour::Input }
        },
        Components::NumberInput => {
            quote! { ::gpui_form::core::components::ComponentsBehaviour::NumberInput }
        },
        Components::Checkbox => {
            quote! { ::gpui_form::core::components::ComponentsBehaviour::Checkbox }
        },
        Components::Switch => {
            quote! { ::gpui_form::core::components::ComponentsBehaviour::Switch }
        },
        Components::Dropdown(options) => {
            let searchable = options.behaviour.searchable;
            let partial = options.behaviour.partial;
            quote! {
                ::gpui_form::core::components::ComponentsBehaviour::Dropdown(
                    ::gpui_form::core::components::BehaviourDropdownOptions {
                        searchable: #searchable,
                        partial: #partial,
                    }
                )
            }
        },
        Components::DatePicker => {
            quote! { ::gpui_form::core::components::ComponentsBehaviour::DatePicker }
        },
        Components::Custom(custom_options) => {
            let component_ident = &custom_options.behaviour.name;
            quote! { #component_ident }
        },
    }
}

struct ComponentFieldContent {
    field_structure_tokens: TokenStream,
    field_base_declarations_tokens: TokenStream,
    should_be_unwrapped: (String, bool),
}

fn generate_component_field(field: &ComponentField) -> ComponentFieldContent {
    let field_name = field.ident.as_ref().unwrap().to_string();
    let field_type = &field.ty;

    let mut field_structure_tokens = proc_macro2::TokenStream::new();
    let mut field_base_declarations_tokens = proc_macro2::TokenStream::new();
    let mut should_be_unwrapped = (field_name.clone(), false);

    let component_def = if field.component.is_some() {
        field.component.as_ref().unwrap()
    } else {
        return ComponentFieldContent {
            field_structure_tokens,
            field_base_declarations_tokens,
            should_be_unwrapped,
        };
    };

    match component_def {
        Components::Input => {
            let component = InputComponent(FieldInformation::new(
                InputOptions,
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
            should_be_unwrapped.1 = true;
        },
        Components::NumberInput => {
            let component = NumberInputComponent(FieldInformation::new(
                NumberInputOptions,
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
            should_be_unwrapped.1 = true;
        },
        Components::Checkbox => {
            let component = CheckboxComponent(FieldInformation::new(
                CheckboxOptions,
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
            should_be_unwrapped.1 = true;
        },
        Components::Switch => {
            let component = SwitchComponent(FieldInformation::new(
                SwitchOptions,
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
            should_be_unwrapped.1 = true;
        },
        Components::Dropdown(options) => {
            let component = DropdownComponent(FieldInformation::new(
                options.clone(),
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
            should_be_unwrapped.1 = true;
        },
        Components::DatePicker => {
            let component = DatePickerComponent(FieldInformation::new(
                DatePickerOptions,
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
            should_be_unwrapped.1 = false;
        },
        Components::Custom(options) => {
            let component = CustomComponent(FieldInformation::new(
                options.clone(),
                field_name.clone(),
                extract_type_ident(field_type),
            ));
            component.field_tokens(
                &mut field_structure_tokens,
                &mut field_base_declarations_tokens,
            );
            should_be_unwrapped.1 = options.behaviour.should_be_unwrapped;
        },
    }

    ComponentFieldContent {
        field_structure_tokens,
        field_base_declarations_tokens,
        should_be_unwrapped,
    }
}

fn extract_type_ident(ty: &Type) -> Ident {
    match ty {
        Type::Path(type_path) => {
            let last_segment = type_path.path.segments.last().unwrap_or_else(|| {
                panic!(
                    "Expected at least one segment in type path: {:?}",
                    type_path.to_token_stream()
                )
            });

            if last_segment.ident == "Option"
                && let PathArguments::AngleBracketed(args) = &last_segment.arguments
                && let Some(GenericArgument::Type(inner_type)) = args.args.first()
            {
                return extract_type_ident(inner_type);
            }
            last_segment.ident.clone()
        },
        _ => panic!(
            "Unsupported type for component field: not a Type::Path. Got: {:?}",
            ty.to_token_stream()
        ),
    }
}

pub struct GpuiFormOptions {
    pub generate_shape: bool,
}

pub fn from(input: proc_macro::TokenStream, options: GpuiFormOptions) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    let parsed = match ComponentStruct::from_derive_input(&derive_input) {
        Ok(parsed) => parsed,
        Err(e) => return e.write_errors().into(),
    };

    let struct_name = &parsed.ident;
    let components_holder_name = format_ident!("{}FormFields", struct_name);
    let components_base_declarations_name = format_ident!("{}FormComponents", struct_name);

    let fields_iter = match &parsed.data {
        ast::Data::Struct(s) => &s.fields,
        _ => unreachable!("GpuiForm derive only supports named structs"),
    };

    let component_field_pairs: Vec<ComponentFieldContent> = fields_iter
        .iter()
        .filter(|field| !field.skip())
        .map(generate_component_field)
        .collect();

    let (field_structure_tokens, field_base_declarations_tokens, should_be_unwrapped): (
        Vec<TokenStream>,
        Vec<TokenStream>,
        HashMap<String, bool>,
    ) = component_field_pairs
        .into_iter()
        .map(|content| {
            (
                content.field_structure_tokens,
                content.field_base_declarations_tokens,
                content.should_be_unwrapped,
            )
        })
        .multiunzip();

    let field_variant_construction_code: Vec<TokenStream> = fields_iter
        .iter()
        .filter_map(|field| {
            if field.skip() || field.component.is_none() {
                None
            } else {
                let field_name_str = field
                    .ident
                    .as_ref()
                    .expect("Field should have an ident if not skipped and has component")
                    .to_string();
                let (is_optional, base_type) = 'option_check: {
                    if let syn::Type::Path(type_path) = &field.ty
                        && let Some(segment) = type_path.path.segments.last()
                        && segment.ident == "Option"
                        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
                    {
                        break 'option_check (true, inner_ty);
                    }
                    (false, &field.ty)
                };

                let field_type_str = base_type.to_token_stream().to_string();
                let component_def = field.component.as_ref().unwrap();
                let behaviour_tokens = get_components_behaviour_tokens(component_def);
                Some(quote! {
                    ::gpui_form::core::registry::FieldVariant::new(
                        #field_name_str,
                        #field_type_str,
                        #is_optional,
                        #behaviour_tokens
                    )
                })
            }
        })
        .collect();

    let shape_impl = if options.generate_shape {
        quote! {
            ::gpui_form::core::registry::inventory::submit! {
                ::gpui_form::core::registry::GpuiFormShape::new(
                    stringify!(#struct_name),
                    &[
                        #(#field_variant_construction_code),*
                    ]
                )
            }
        }
    } else {
        quote! {}
    };

    let model_options = unwrapped_core::Opts::builder()
        .suffix(format_ident!("FormValueHolder"))
        .build();

    let macro_options =
        unwrapped_core::ProcUsageOpts::new(should_be_unwrapped, Some(format_ident!("gpui_form")));

    let model_struct = unwrapped_core::unwrapped(&derive_input, Some(model_options), macro_options);

    let expanded = quote! {
        #model_struct
        pub struct #components_holder_name {
            #(#field_structure_tokens)*
        }

        #shape_impl

        pub struct #components_base_declarations_name;

        impl #components_base_declarations_name {
          #(#field_base_declarations_tokens)*
        }
    };

    expanded.into()
}
