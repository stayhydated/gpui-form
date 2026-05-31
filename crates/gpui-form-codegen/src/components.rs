use gpui_form_schema::registry::validate_component_suffix;
use proc_macro2::{Group, Span, TokenStream, TokenTree};
use quote::{format_ident, quote, quote_spanned};
use syn::Path;
use syn::spanned::Spanned as _;

use crate::CratePaths;
use crate::metadata::rust_path_tokens;

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

#[derive(Clone, Debug)]
pub struct ComponentFieldIr {
    pub shape: ResolvedComponentShape,
    pub field_ident: syn::Ident,
    pub storage_policy: ComponentShapeStoragePolicy,
}

#[derive(Clone, Debug)]
pub struct ComponentShapeStoragePolicy {
    shape: syn::Path,
}

impl ComponentShapeStoragePolicy {
    pub fn shape_owned(shape: syn::Path) -> Self {
        Self { shape }
    }

    pub fn shape(&self) -> &syn::Path {
        &self.shape
    }
}

#[derive(Clone, Debug)]
pub struct ShapeOptions {
    /// Path to a type implementing `gpui_form::runtime::shape::ComponentShape`.
    pub shape: syn::Path,
    span: Span,
}

impl ShapeOptions {
    pub fn from_shape(shape: Path) -> Self {
        let span = shape.span();
        Self::from_shape_with_span(shape, span)
    }

    pub fn from_shape_with_span(shape: Path, span: Span) -> Self {
        let shape = normalize_shape_path(shape);

        Self { shape, span }
    }

    fn resolved_shape(&self, field_type: &syn::Type) -> syn::Path {
        substitute_infer_in_path(&self.shape, field_type)
    }

    pub fn resolve(&self, field_name: String, field_type: syn::Type) -> ResolvedComponentShape {
        let shape = self.resolved_shape(&field_type);
        let component_suffix = component_suffix_for_shape(&shape, &field_name);

        ResolvedComponentShape {
            shape,
            field_name,
            field_type,
            component_suffix,
            span: self.span,
        }
    }
}

fn path_suffix_source(path: &Path) -> Option<String> {
    let ident = path.segments.last()?.ident.to_string();
    let suffix_source = ident
        .strip_suffix("Shape")
        .or_else(|| ident.strip_suffix("State"))
        .unwrap_or(&ident);

    Some(suffix_source.to_string())
}

fn component_suffix_for_shape(path: &Path, field_name: &str) -> String {
    path_suffix_source(path)
        .and_then(|source| {
            gpui_form_schema::registry::component_suffix_from_suffix(field_name, &source)
        })
        .unwrap_or_else(|| "shape".to_string())
}

fn field_assertion_ident_fragment(field_name: &str) -> String {
    let field_name = field_name.strip_prefix("r#").unwrap_or(field_name);
    let mut fragment = String::new();
    for ch in field_name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            fragment.push(ch);
        } else {
            fragment.push('_');
        }
    }

    if fragment.is_empty() {
        fragment.push_str("field");
    }
    if fragment
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_digit())
    {
        fragment.insert(0, '_');
    }

    fragment
}

fn normalize_shape_path(mut path: Path) -> Path {
    for segment in &mut path.segments {
        if let syn::PathArguments::AngleBracketed(args) = &mut segment.arguments {
            args.colon2_token = None;
        }
    }

    path
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

#[derive(Clone, Debug)]
pub struct ResolvedComponentShape {
    /// Shape path after substituting field type for any `_` generic arguments.
    pub shape: syn::Path,
    pub field_name: String,
    pub field_type: syn::Type,
    component_suffix: String,
    span: Span,
}

impl ResolvedComponentShape {
    pub fn shape(&self) -> &syn::Path {
        &self.shape
    }

    pub fn component_suffix(&self) -> &str {
        &self.component_suffix
    }

    pub fn constructor_tokens(&self) -> TokenStream {
        let shape = self.shape();
        let crate_paths = CratePaths::resolve();
        let runtime_crate = crate_paths.gpui_form_facade_runtime();

        quote! {
            <#shape as #runtime_crate::shape::ComponentShape>::new(window, cx)
        }
    }

    pub fn storage_policy(&self) -> ComponentShapeStoragePolicy {
        ComponentShapeStoragePolicy::shape_owned(self.shape.clone())
    }

    pub fn type_check_tokens(&self) -> TokenStream {
        let shape = self.shape();
        let field_type = &self.field_type;
        let span = self.span;
        let crate_paths = CratePaths::resolve();
        let runtime_crate = tokens_with_span(&crate_paths.gpui_form_facade_runtime(), span);
        let field_fragment = field_assertion_ident_fragment(&self.field_name);
        let declared_shape_assertion = format_ident!(
            "__gpui_form_assert_{field_fragment}_declared_component_shape",
            span = span
        );
        let value_compat_assertion = format_ident!(
            "__gpui_form_assert_{field_fragment}_component_value_compatibility",
            span = span
        );
        let value_binding_assertion = format_ident!(
            "__gpui_form_assert_{field_fragment}_component_value_binding",
            span = span
        );

        quote_spanned! {span=>
            {
                fn #declared_shape_assertion<
                    Shape: #runtime_crate::shape::DeclaredComponentShape
                        + #runtime_crate::shape::ComponentShape,
                >() {}
                #declared_shape_assertion::<#shape>();

                fn #value_compat_assertion<
                    Shape: #runtime_crate::shape::ComponentShapeFor<Value>,
                    Value,
                >() {}
                #value_compat_assertion::<#shape, #field_type>();

                fn #value_binding_assertion<
                    Shape: #runtime_crate::shape::ComponentShape,
                    Value,
                >()
                where
                    <Shape as #runtime_crate::shape::ComponentShape>::ValueBindingPolicy:
                        #runtime_crate::shape::AssertComponentValueBindingPolicy<
                            Shape,
                            Value,
                        >,
                {
                    <<Shape as #runtime_crate::shape::ComponentShape>::ValueBindingPolicy
                        as #runtime_crate::shape::AssertComponentValueBindingPolicy<
                            Shape,
                            Value,
                        >>::assert_component_value_binding_policy();
                }
                #value_binding_assertion::<#shape, #field_type>();
            }
        }
    }

    pub fn field_ir(&self) -> syn::Result<ComponentFieldIr> {
        let field_ident =
            crate::names::ComponentEntityFieldName::try_new_spanned(&self.field_name, self.span)?;
        Ok(ComponentFieldIr {
            shape: self.clone(),
            field_ident: field_ident.0,
            storage_policy: self.storage_policy(),
        })
    }

    pub fn component_variant_tokens(&self) -> TokenStream {
        let schema_crate = CratePaths::resolve().gpui_form;
        let shape = rust_path_tokens(&schema_crate, self.shape());
        let capabilities_tokens = self.capabilities_tokens();
        let prototyping_tokens = self.prototyping_tokens();

        quote! {
            #schema_crate::schema::registry::FieldComponentVariant::new(
                #shape
            )
            .with_capabilities(#capabilities_tokens)
            #prototyping_tokens
        }
    }

    fn capabilities_tokens(&self) -> TokenStream {
        let shape = self.shape();
        let crate_paths = CratePaths::resolve();
        let runtime_crate = crate_paths.gpui_form_facade_runtime();
        let schema_crate = crate_paths.gpui_form;

        quote! {
            #schema_crate::schema::registry::ComponentCapabilities::new()
                .with_render(
                    if <<#shape as #runtime_crate::shape::ComponentShape>::RenderComponent
                        as #runtime_crate::shape::ComponentRender<
                            <#shape as #runtime_crate::shape::ComponentShape>::State
                        >>::RENDERS
                    {
                        #schema_crate::schema::registry::RenderCapability::Component
                    } else {
                        #schema_crate::schema::registry::RenderCapability::None
                    }
                )
                .with_value_binding(
                    if <<#shape as #runtime_crate::shape::ComponentShape>::ValueBindingPolicy
                        as #runtime_crate::shape::ComponentValueBindingPolicy>::VALUE_BINDING
                    {
                        #schema_crate::schema::registry::ValueBindingCapability::Inherited
                    } else {
                        #schema_crate::schema::registry::ValueBindingCapability::None
                    }
                )
        }
    }

    fn prototyping_tokens(&self) -> TokenStream {
        let shape = self.shape();
        let suffix = self.component_suffix();
        let crate_paths = CratePaths::resolve();
        let runtime_crate = crate_paths.gpui_form_facade_runtime();
        let schema_crate = crate_paths.gpui_form;
        quote! {
            .with_prototyping_field_suffix(
                match <#shape as #runtime_crate::shape::ComponentShape>::PROTOTYPING.field_suffix {
                    Some(suffix) => Some(suffix),
                    None => Some(
                        #schema_crate::schema::registry::ComponentSuffix::new(#suffix)
                    ),
                }
            )
        }
    }
}

impl ShapeOptions {
    pub fn field_ir(
        &self,
        field_name: String,
        field_type: syn::Type,
        _field_default: Option<syn::Expr>,
    ) -> syn::Result<ComponentFieldIr> {
        self.resolve(field_name, field_type).field_ir()
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

    fn compact_path(path: &syn::Path) -> String {
        path.to_token_stream()
            .to_string()
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect()
    }

    #[test]
    fn component_field_ir_plans_identifier_shape_and_storage_policy() {
        let options = ShapeOptions::from_shape(syn::parse_quote!(crate::Input<_>));
        let field_type: syn::Type = syn::parse_quote!(String);

        let ir = options
            .field_ir("email".to_string(), field_type, None)
            .expect("valid component field should plan semantic IR");

        assert_eq!(ir.field_ident.to_string(), "email");
        assert_eq!(compact_path(ir.shape.shape()), "crate::Input<String>");
        assert_eq!(
            compact_path(ir.storage_policy.shape()),
            "crate::Input<String>"
        );
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
