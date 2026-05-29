use darling::Error as DarlingError;
use gpui_form_schema::registry::validate_component_suffix;
use proc_macro2::{Group, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned as _;
use syn::{LitStr, Path, Type};

use crate::CratePaths;
use crate::implementations::ComponentLayout as _;
use crate::metadata::{rust_path_tokens, rust_type_tokens};

fn tokens_with_span<T: quote::ToTokens>(value: &T, span: Span) -> TokenStream {
    value
        .to_token_stream()
        .into_iter()
        .map(|token| token_tree_with_span(token, span))
        .collect()
}

fn token_tree_with_span(mut token: TokenTree, span: Span) -> TokenTree {
    if let TokenTree::Group(group) = &mut token {
        let mut new_group = Group::new(
            group.delimiter(),
            group
                .stream()
                .into_iter()
                .map(|token| token_tree_with_span(token, span))
                .collect(),
        );
        new_group.set_span(span);
        return TokenTree::Group(new_group);
    }

    match &mut token {
        TokenTree::Group(_) => unreachable!("groups are handled above"),
        TokenTree::Ident(ident) => ident.set_span(span),
        TokenTree::Punct(punct) => punct.set_span(span),
        TokenTree::Literal(literal) => literal.set_span(span),
    }

    token
}

pub fn validate_shape_field_suffix(value: &str) -> Result<(), String> {
    validate_component_suffix(value).map_err(|err| err.to_string())
}

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
    pub required_value: RequiredValue,
}

#[derive(Clone, Debug)]
pub enum RequiredValue {
    Explicit(bool),
    Shape(syn::Path),
}

impl RequiredValue {
    pub fn explicit(value: bool) -> Self {
        Self::Explicit(value)
    }

    pub fn shape(shape: syn::Path) -> Self {
        Self::Shape(shape)
    }

    pub fn metadata_tokens(&self) -> TokenStream {
        match self {
            Self::Explicit(value) => quote! { #value },
            Self::Shape(shape) => {
                let crate_paths = CratePaths::resolve();
                let runtime_crate = crate_paths.gpui_form_facade_runtime();
                quote! {
                    <<#shape as #runtime_crate::shape::ComponentShape>::ValueStoragePolicy
                        as #runtime_crate::shape::ComponentValueStoragePolicy>::REQUIRES_VALUE
                }
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct ShapeOptions {
    /// Path to a type implementing `gpui_form::runtime::shape::ComponentShape`.
    pub shape: syn::Path,
    /// UI component type (e.g. `TagsInput` or `Combobox<_>`).
    /// When provided, the prototyping code generator emits `Component::new(&entity)`.
    pub component: Option<Type>,
    /// Optional explicit generated field/helper suffix.
    pub field_suffix: Option<LitStr>,
    span: Span,
}

impl ShapeOptions {
    fn from_shape(shape: Path) -> Self {
        let span = shape.span();
        let shape = normalize_shape_path(shape);

        Self {
            shape,
            component: None,
            field_suffix: None,
            span,
        }
    }

    pub fn from_explicit_metadata(
        shape: Path,
        component: Option<Type>,
        field_suffix: Option<LitStr>,
    ) -> darling::Result<Self> {
        let mut options = Self::from_shape(shape);

        if let Some(component) = component {
            let span = component.span();
            options.set_component(component, &span)?;
        }

        if let Some(field_suffix) = field_suffix {
            let span = field_suffix.span();
            options.set_field_suffix(field_suffix, &span)?;
        }

        Ok(options)
    }

    pub fn set_component<S: syn::spanned::Spanned>(
        &mut self,
        component: Type,
        span: &S,
    ) -> darling::Result<()> {
        let component = normalize_component_type(component, span)?;
        set_metadata_once(&mut self.component, component, "component", span)
    }

    pub fn set_field_suffix<S: syn::spanned::Spanned>(
        &mut self,
        field_suffix: LitStr,
        span: &S,
    ) -> darling::Result<()> {
        validate_shape_field_suffix(&field_suffix.value())
            .map_err(|message| DarlingError::custom(message).with_span(&field_suffix))?;
        set_metadata_once(&mut self.field_suffix, field_suffix, "field_suffix", span)
    }

    pub fn resolved_shape(&self, field_type: &syn::Type) -> syn::Path {
        substitute_infer_in_path(&self.shape, field_type)
    }

    pub fn runtime_shape(&self, field_type: &syn::Type) -> syn::Path {
        self.resolved_shape(field_type)
    }

    pub fn with_field_type(mut self, field_type: &syn::Type) -> Self {
        self.shape = self.resolved_shape(field_type);
        self
    }

    pub fn component_suffix(&self, field_name: &str) -> String {
        if let Some(field_suffix) = &self.field_suffix {
            return gpui_form_schema::registry::component_suffix_from_suffix(
                field_name,
                &field_suffix.value(),
            )
            .unwrap_or_else(|| "shape".to_string());
        }

        let suffix_source = self
            .component
            .as_ref()
            .and_then(type_suffix_source)
            .or_else(|| path_suffix_source(&self.shape));

        suffix_source
            .and_then(|source| {
                gpui_form_schema::registry::component_suffix_from_suffix(field_name, &source)
            })
            .unwrap_or_else(|| "shape".to_string())
    }

    pub fn constructor_tokens(&self, field_type: &syn::Type) -> TokenStream {
        let shape = self.runtime_shape(field_type);
        let crate_paths = CratePaths::resolve();
        let runtime_crate = crate_paths.gpui_form_facade_runtime();

        quote! {
            <#shape as #runtime_crate::shape::ComponentShape>::new(window, cx)
        }
    }

    pub fn type_check_tokens(&self, field_type: &syn::Type) -> TokenStream {
        let shape = self.resolved_shape(field_type);
        let span = self.span;
        let crate_paths = CratePaths::resolve();
        let runtime_crate = tokens_with_span(&crate_paths.gpui_form_facade_runtime(), span);
        let value_binding_assertion = quote_spanned! {span=>
            {
                <<#shape as #runtime_crate::shape::ComponentShape>::ValueBindingPolicy
                    as #runtime_crate::shape::AssertComponentValueBindingPolicy<
                        #shape,
                        #field_type,
                    >>::assert_component_value_binding_policy();
            }
        };

        quote_spanned! {span=>
            {
                fn __gpui_form_assert_component_shape<
                    Shape: #runtime_crate::shape::ComponentShape
                        + #runtime_crate::shape::ComponentShapeFor<Value>,
                    Value,
                >() {}
                __gpui_form_assert_component_shape::<#shape, #field_type>();
                #value_binding_assertion
            }
        }
    }

    pub fn required_value(&self, field_type: &syn::Type) -> RequiredValue {
        RequiredValue::shape(self.resolved_shape(field_type))
    }
}

fn type_suffix_source(ty: &Type) -> Option<String> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    path_suffix_source(&type_path.path)
}

fn path_suffix_source(path: &Path) -> Option<String> {
    let ident = path.segments.last()?.ident.to_string();
    let suffix_source = ident
        .strip_suffix("Shape")
        .or_else(|| ident.strip_suffix("State"))
        .unwrap_or(&ident);

    Some(suffix_source.to_string())
}

fn normalize_shape_path(mut path: Path) -> Path {
    for segment in &mut path.segments {
        if let syn::PathArguments::AngleBracketed(args) = &mut segment.arguments {
            args.colon2_token = None;
        }
    }

    path
}

fn normalize_component_type<S: syn::spanned::Spanned>(
    component: Type,
    span: &S,
) -> darling::Result<Type> {
    match component {
        Type::Path(mut type_path) => {
            type_path.path = normalize_shape_path(type_path.path);
            Ok(Type::Path(type_path))
        },
        Type::Infer(infer) => Err(DarlingError::custom(
            "component type cannot be inferred with bare `_`; use an explicit component type",
        )
        .with_span(&infer)),
        _ => Err(DarlingError::custom(
            "component metadata must be a path-like type, such as `my_crate::Input`",
        )
        .with_span(span)),
    }
}

fn set_metadata_once<T, S: syn::spanned::Spanned>(
    slot: &mut Option<T>,
    value: T,
    option: &str,
    span: &S,
) -> darling::Result<()> {
    if slot.is_some() {
        return Err(DarlingError::custom(format!(
            "duplicate `{option}` option in `gpui_form` field attribute; remove the duplicate `{option}` entry",
        ))
        .with_span(span));
    }

    *slot = Some(value);
    Ok(())
}

fn substitute_infer_in_type(ty: &syn::Type, replacement: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Infer(_) => replacement.clone(),
        syn::Type::Path(type_path) => {
            let mut type_path = type_path.clone();
            type_path.path = substitute_infer_in_path(&type_path.path, replacement);
            syn::Type::Path(type_path)
        },
        syn::Type::Array(array) => {
            let mut array = array.clone();
            array.elem = Box::new(substitute_infer_in_type(&array.elem, replacement));
            syn::Type::Array(array)
        },
        syn::Type::Slice(slice) => {
            let mut slice = slice.clone();
            slice.elem = Box::new(substitute_infer_in_type(&slice.elem, replacement));
            syn::Type::Slice(slice)
        },
        syn::Type::Ptr(ptr) => {
            let mut ptr = ptr.clone();
            ptr.elem = Box::new(substitute_infer_in_type(&ptr.elem, replacement));
            syn::Type::Ptr(ptr)
        },
        syn::Type::BareFn(bare_fn) => {
            let mut bare_fn = bare_fn.clone();
            for input in &mut bare_fn.inputs {
                input.ty = substitute_infer_in_type(&input.ty, replacement);
            }
            substitute_infer_in_return_type(&mut bare_fn.output, replacement);
            syn::Type::BareFn(bare_fn)
        },
        syn::Type::TraitObject(trait_object) => {
            let mut trait_object = trait_object.clone();
            substitute_infer_in_bounds(&mut trait_object.bounds, replacement);
            syn::Type::TraitObject(trait_object)
        },
        syn::Type::ImplTrait(impl_trait) => {
            let mut impl_trait = impl_trait.clone();
            substitute_infer_in_bounds(&mut impl_trait.bounds, replacement);
            syn::Type::ImplTrait(impl_trait)
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
            *reference.elem = substitute_infer_in_type(&reference.elem, replacement);
            syn::Type::Reference(reference)
        },
        _ => ty.clone(),
    }
}

fn substitute_infer_in_return_type(return_type: &mut syn::ReturnType, replacement: &syn::Type) {
    if let syn::ReturnType::Type(_, ty) = return_type {
        **ty = substitute_infer_in_type(ty, replacement);
    }
}

fn substitute_infer_in_bounds(
    bounds: &mut syn::punctuated::Punctuated<syn::TypeParamBound, syn::Token![+]>,
    replacement: &syn::Type,
) {
    for bound in bounds {
        if let syn::TypeParamBound::Trait(trait_bound) = bound {
            trait_bound.path = substitute_infer_in_path(&trait_bound.path, replacement);
        }
    }
}

fn substitute_infer_in_path(path: &syn::Path, replacement: &syn::Type) -> syn::Path {
    let mut path = path.clone();

    for segment in &mut path.segments {
        substitute_infer_in_path_arguments(&mut segment.arguments, replacement);
    }

    path
}

fn substitute_infer_in_path_arguments(arguments: &mut syn::PathArguments, replacement: &syn::Type) {
    match arguments {
        syn::PathArguments::AngleBracketed(args) => {
            substitute_infer_in_angle_bracketed_arguments(args, replacement);
        },
        syn::PathArguments::Parenthesized(args) => {
            args.inputs = args
                .inputs
                .iter()
                .map(|ty| substitute_infer_in_type(ty, replacement))
                .collect();
            substitute_infer_in_return_type(&mut args.output, replacement);
        },
        syn::PathArguments::None => {},
    }
}

fn substitute_infer_in_angle_bracketed_arguments(
    args: &mut syn::AngleBracketedGenericArguments,
    replacement: &syn::Type,
) {
    for arg in &mut args.args {
        match arg {
            syn::GenericArgument::Type(ty) => {
                *ty = substitute_infer_in_type(ty, replacement);
            },
            syn::GenericArgument::AssocType(assoc_type) => {
                if let Some(generics) = &mut assoc_type.generics {
                    substitute_infer_in_angle_bracketed_arguments(generics, replacement);
                }
                assoc_type.ty = substitute_infer_in_type(&assoc_type.ty, replacement);
            },
            syn::GenericArgument::Constraint(constraint) => {
                if let Some(generics) = &mut constraint.generics {
                    substitute_infer_in_angle_bracketed_arguments(generics, replacement);
                }
                substitute_infer_in_bounds(&mut constraint.bounds, replacement);
            },
            _ => {},
        }
    }
}

impl ComponentOption for ShapeOptions {}

pub struct ShapeComponent(pub FieldInformation<ShapeOptions>);

impl ShapeOptions {
    pub fn generate_field_layout(
        &self,
        field_name: String,
        field_type: syn::Type,
        _field_default: Option<syn::Expr>,
    ) -> GeneratedFieldLayout {
        let mut field_structure_tokens = TokenStream::new();
        let mut field_base_declarations_tokens = TokenStream::new();

        let options = self.clone().with_field_type(&field_type);
        let component = ShapeComponent(FieldInformation::new(
            options,
            field_name,
            field_type.clone(),
        ));
        component.field_tokens(
            &mut field_structure_tokens,
            &mut field_base_declarations_tokens,
        );

        GeneratedFieldLayout {
            field_structure_tokens,
            field_base_declarations_tokens,
            required_value: self.required_value(&field_type),
        }
    }

    pub fn component_variant_tokens(
        &self,
        field_type: &syn::Type,
        field_name: &str,
    ) -> TokenStream {
        let schema_crate = CratePaths::resolve().gpui_form;
        let shape = rust_path_tokens(&schema_crate, &self.resolved_shape(field_type));
        let component_type_tokens = self.component_type_tokens(field_type);
        let value_binding_tokens = self.value_binding_tokens(field_type);
        let prototyping_tokens = self.prototyping_tokens(field_type, field_name);

        quote! {
            #schema_crate::schema::registry::FieldComponentVariant::new(
                #shape
            )
            #component_type_tokens
            #value_binding_tokens
            #prototyping_tokens
        }
    }

    fn component_type_tokens(&self, field_type: &syn::Type) -> TokenStream {
        let shape = self.resolved_shape(field_type);
        let crate_paths = CratePaths::resolve();
        let runtime_crate = crate_paths.gpui_form_facade_runtime();
        let schema_crate = crate_paths.gpui_form;
        if let Some(component) = self.component.as_ref() {
            let component_type = rust_type_tokens(&schema_crate, component);
            quote! {
                .with_component_type(
                    #component_type
                )
            }
        } else {
            quote! {
                .with_component_type_opt(
                    #schema_crate::schema::registry::RustType::new_opt_unchecked(
                        <#shape as #runtime_crate::shape::ComponentShape>::COMPONENT_TYPE
                    )
                )
            }
        }
    }

    fn value_binding_tokens(&self, field_type: &syn::Type) -> TokenStream {
        let shape = self.resolved_shape(field_type);
        let crate_paths = CratePaths::resolve();
        let runtime_crate = crate_paths.gpui_form_facade_runtime();

        quote! {
            .with_value_binding(
                <<#shape as #runtime_crate::shape::ComponentShape>::ValueBindingPolicy
                    as #runtime_crate::shape::ComponentValueBindingPolicy>::VALUE_BINDING
            )
        }
    }

    fn prototyping_tokens(&self, field_type: &syn::Type, field_name: &str) -> TokenStream {
        let suffix = self
            .clone()
            .with_field_type(field_type)
            .component_suffix(field_name);
        let schema_crate = CratePaths::resolve().gpui_form;
        quote! {
            .with_prototyping_field_suffix(Some(
                #schema_crate::schema::registry::ComponentSuffix::new(#suffix)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens as _;

    fn compact_type(ty: &syn::Type) -> String {
        ty.to_token_stream()
            .to_string()
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect()
    }

    #[test]
    fn substitutes_infer_in_arrays_slices_pointers_and_bare_fns() {
        let ty: syn::Type = syn::parse_quote! {
            fn([_; 2], &[_], *const _, *mut _) -> Option<_>
        };
        let replacement: syn::Type = syn::parse_quote!(String);

        let substituted = substitute_infer_in_type(&ty, &replacement);

        assert_eq!(
            compact_type(&substituted),
            "fn([String;2],&[String],*constString,*mutString)->Option<String>"
        );
    }

    #[test]
    fn substitutes_infer_in_trait_objects_and_generic_constraints() {
        let ty: syn::Type = syn::parse_quote! {
            dyn crate::Shape<Assoc: Iterator<Item = _>> + FnOnce(_) -> _
        };
        let replacement: syn::Type = syn::parse_quote!(String);

        let substituted = substitute_infer_in_type(&ty, &replacement);

        assert_eq!(
            compact_type(&substituted),
            "dyncrate::Shape<Assoc:Iterator<Item=String>>+FnOnce(String)->String"
        );
    }
}
