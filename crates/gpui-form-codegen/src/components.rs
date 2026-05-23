use darling::{Error as DarlingError, FromMeta, util::Flag};
use gpui_form_schema::components::ComponentKind;
use proc_macro2::TokenStream;
use quote::{ToTokens as _, quote};

use crate::implementations::ComponentLayout as _;

pub trait ComponentOption {}

pub struct FieldInformation<T: ComponentOption> {
    pub options: T,
    pub name: String,
    pub r#type: syn::Type,
}

impl<T: ComponentOption> FieldInformation<T> {
    pub fn new(options: T, name: String, r#type: syn::Type) -> Self {
        Self {
            options,
            name,
            r#type,
        }
    }
}

pub struct GeneratedFieldLayout {
    pub field_structure_tokens: TokenStream,
    pub field_base_declarations_tokens: TokenStream,
    pub wrap_in_option: bool,
}

fn default_custom_wraps_in_option() -> bool {
    true
}

#[derive(Clone, Debug)]
pub struct CustomOptions {
    /// Path to a type implementing `gpui_form_component::custom::CustomComponentShape`.
    pub shape: syn::Path,
    /// UI component type path (e.g. `TagsInput`).
    /// When provided, the prototyping code generator emits `Component::new(&entity)`.
    pub component: Option<syn::Path>,
    /// Whether the value holder should store this field as `Option<T>`.
    /// Defaults to `true`.
    pub wraps_in_option: bool,
    /// Whether prototyping code should wire this custom component through
    /// `CustomComponentValueAdapter`.
    pub value_binding: Option<bool>,
}

#[derive(Debug, Default, FromMeta)]
struct CustomOptionsMeta {
    #[darling(default)]
    shape: Option<syn::Path>,
    #[darling(default)]
    state: Option<syn::Path>,
    #[darling(default)]
    component: Option<syn::Path>,
    #[darling(default = "default_custom_wraps_in_option")]
    wraps_in_option: bool,
    #[darling(default)]
    value_binding: Flag,
}

impl CustomOptions {
    fn from_meta(meta: CustomOptionsMeta) -> darling::Result<Self> {
        let CustomOptionsMeta {
            shape,
            state,
            component,
            wraps_in_option,
            value_binding,
        } = meta;

        let shape = match (shape, state) {
            (Some(shape), None) | (None, Some(shape)) => shape,
            (Some(_), Some(_)) => {
                return Err(DarlingError::custom(
                    "custom component may specify only one of `shape` or `state`",
                ));
            },
            (None, None) => {
                return Err(DarlingError::custom(
                    "custom component requires `shape = ...` or `state = ...`",
                ));
            },
        };

        Ok(Self {
            shape,
            component,
            wraps_in_option,
            value_binding: value_binding.is_present().then_some(true),
        })
    }

    pub fn resolved_shape(&self, field_type: &syn::Type) -> syn::Path {
        substitute_infer_in_path(&self.shape, field_type)
    }

    pub fn with_field_type(mut self, field_type: &syn::Type) -> Self {
        self.shape = self.resolved_shape(field_type);
        self
    }
}

impl FromMeta for CustomOptions {
    fn from_word() -> darling::Result<Self> {
        Err(DarlingError::custom(
            "custom component requires `shape = ...` or `state = ...`",
        ))
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let meta = CustomOptionsMeta::from_list(items)?;
        Self::from_meta(meta)
    }
}

fn substitute_infer_in_type(ty: &syn::Type, replacement: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Infer(_) => replacement.clone(),
        syn::Type::Path(type_path) => {
            let mut type_path = type_path.clone();
            type_path.path = substitute_infer_in_path(&type_path.path, replacement);
            syn::Type::Path(type_path)
        },
        syn::Type::Tuple(tuple) => {
            let mut tuple = tuple.clone();
            tuple.elems = tuple
                .elems
                .iter()
                .map(|ty| substitute_infer_in_type(ty, replacement))
                .collect();
            syn::Type::Tuple(tuple)
        },
        syn::Type::Paren(paren) => {
            let mut paren = paren.clone();
            paren.elem = Box::new(substitute_infer_in_type(&paren.elem, replacement));
            syn::Type::Paren(paren)
        },
        syn::Type::Group(group) => {
            let mut group = group.clone();
            group.elem = Box::new(substitute_infer_in_type(&group.elem, replacement));
            syn::Type::Group(group)
        },
        syn::Type::Reference(reference) => {
            let mut reference = reference.clone();
            reference.elem = Box::new(substitute_infer_in_type(&reference.elem, replacement));
            syn::Type::Reference(reference)
        },
        _ => ty.clone(),
    }
}

fn substitute_infer_in_path(path: &syn::Path, replacement: &syn::Type) -> syn::Path {
    let mut path = path.clone();

    for segment in &mut path.segments {
        if let syn::PathArguments::AngleBracketed(args) = &mut segment.arguments {
            for arg in &mut args.args {
                match arg {
                    syn::GenericArgument::Type(ty) => {
                        *ty = substitute_infer_in_type(ty, replacement);
                    },
                    syn::GenericArgument::AssocType(assoc_type) => {
                        assoc_type.ty = substitute_infer_in_type(&assoc_type.ty, replacement);
                    },
                    _ => {},
                }
            }
        }
    }

    path
}

impl ComponentOption for CustomOptions {}

pub struct CustomComponent(pub FieldInformation<CustomOptions>);

impl CustomComponent {
    pub fn component_name() -> &'static str {
        ComponentKind::Custom.component_name()
    }
}

#[derive(Clone, Debug, FromMeta)]
#[darling(rename_all = "snake_case")]
pub enum Components {
    Custom(CustomOptions),
}

impl Components {
    pub const fn kind(&self) -> ComponentKind {
        match self {
            Self::Custom(_) => ComponentKind::Custom,
        }
    }

    pub fn wraps_in_option(&self) -> bool {
        match self {
            Self::Custom(options) => options.wraps_in_option,
        }
    }

    pub fn generate_field_layout(
        &self,
        field_name: String,
        field_type: syn::Type,
        _field_default: Option<syn::Expr>,
    ) -> GeneratedFieldLayout {
        let mut field_structure_tokens = TokenStream::new();
        let mut field_base_declarations_tokens = TokenStream::new();

        match self {
            Self::Custom(options) => {
                let options = options.clone().with_field_type(&field_type);
                let component =
                    CustomComponent(FieldInformation::new(options, field_name, field_type));
                component.field_tokens(
                    &mut field_structure_tokens,
                    &mut field_base_declarations_tokens,
                );
            },
        }

        GeneratedFieldLayout {
            field_structure_tokens,
            field_base_declarations_tokens,
            wrap_in_option: self.wraps_in_option(),
        }
    }

    pub fn behaviour_tokens(&self, _field_type: &syn::Type) -> TokenStream {
        match self {
            Self::Custom(_) => {
                quote! { ::gpui_form::schema::components::ComponentsBehaviour::Custom }
            },
        }
    }

    pub fn custom_component_tokens(&self, field_type: &syn::Type) -> Option<TokenStream> {
        let Self::Custom(options) = self;

        let shape = options.resolved_shape(field_type);
        if let Some(component) = options.component.as_ref() {
            let component_str = component.to_token_stream().to_string();
            Some(quote! { .with_custom_component(#component_str) })
        } else {
            Some(quote! {
                .with_custom_component_opt(
                    <#shape as ::gpui_form_component::custom::CustomComponentShape>::COMPONENT_PATH
                )
            })
        }
    }

    pub fn custom_shape_tokens(&self, field_type: &syn::Type) -> Option<TokenStream> {
        let Self::Custom(options) = self;

        let shape = options
            .resolved_shape(field_type)
            .to_token_stream()
            .to_string();
        Some(quote! { .with_custom_shape(#shape) })
    }

    pub fn custom_value_binding_tokens(&self, field_type: &syn::Type) -> Option<TokenStream> {
        let Self::Custom(options) = self;

        let shape = options.resolved_shape(field_type);

        Some(match options.value_binding {
            Some(true) => quote! { .with_custom_value_binding(true) },
            Some(false) => quote! { .with_custom_value_binding(false) },
            None => {
                quote! {
                    .with_custom_value_binding(
                        <#shape as ::gpui_form_component::custom::CustomComponentShape>::VALUE_BINDING
                    )
                }
            },
        })
    }
}
