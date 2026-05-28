use darling::Error as DarlingError;
use proc_macro2::{Group, Span, TokenStream, TokenTree};
use quote::{ToTokens as _, quote, quote_spanned};
use syn::spanned::Spanned as _;
use syn::{Expr, Lit, Path, Type};

use crate::CratePaths;
use crate::implementations::ComponentLayout as _;

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
                let runtime_crate = CratePaths::resolve().gpui_form_runtime;
                quote! {
                    <#shape as #runtime_crate::shape::ComponentShape>::REQUIRES_VALUE
                }
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct ComponentMethod {
    pub method: syn::Ident,
    pub args: Vec<Expr>,
}

#[derive(Clone, Debug)]
pub struct ShapeOptions {
    /// Path to a type implementing `gpui_form_runtime::shape::ComponentShape`.
    pub shape: syn::Path,
    /// UI component type (e.g. `TagsInput` or `Combobox<_>`).
    /// When provided, the prototyping code generator emits `Component::new(&entity)`.
    pub component: Option<Type>,
    /// Optional explicit generated field/helper suffix.
    pub field_suffix: Option<String>,
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

    fn from_component_expr(expr: &Expr) -> darling::Result<Self> {
        let (shape, methods) = analyze_component_expr(expr)?;
        let mut options = Self::from_shape(shape);

        for method in &methods {
            options.apply_component_method(&method.method, &method.args)?;
        }

        Ok(options)
    }

    fn apply_component_method(
        &mut self,
        method: &syn::Ident,
        args: &[Expr],
    ) -> darling::Result<()> {
        match method.to_string().as_str() {
            "component" => {
                set_metadata_once(&mut self.component, expect_type_arg(method, args)?, method)?;
            },
            "field_suffix" => {
                set_metadata_once(
                    &mut self.field_suffix,
                    expect_string_arg(method, args)?,
                    method,
                )?;
            },
            _ => {
                return Err(DarlingError::custom(format!(
                    "unknown component metadata `{method}`; supported generic methods are \
                     `component` and `field_suffix`; put component-specific \
                     options in a dedicated ComponentShape wrapper"
                ))
                .with_span(method));
            },
        }

        Ok(())
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
                field_suffix,
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
        let runtime_crate = CratePaths::resolve().gpui_form_runtime;

        quote! {
            <#shape as #runtime_crate::shape::ComponentShape>::new(window, cx)
        }
    }

    pub fn type_check_tokens(
        &self,
        field_type: &syn::Type,
        check_value_holder_storage: bool,
    ) -> Option<TokenStream> {
        let shape = self.resolved_shape(field_type);
        let span = self.span;
        let runtime_crate = tokens_with_span(&CratePaths::resolve().gpui_form_runtime, span);
        let value_binding_assertion = quote_spanned! {span=>
            {
                <<#shape as #runtime_crate::shape::ComponentShape>::ValueBindingPolicy
                    as #runtime_crate::shape::AssertComponentValueBindingPolicy<
                        #shape,
                        #field_type,
                    >>::assert_component_value_binding_policy();
            }
        };
        let value_holder_storage_assertion = if check_value_holder_storage {
            quote_spanned! {span=>
                {
                    fn __gpui_form_assert_value_holder_storage<Policy, Value>()
                    where
                        Policy: #runtime_crate::shape::ValueHolderStorage<Value>,
                    {}

                    __gpui_form_assert_value_holder_storage::<
                        <#shape as #runtime_crate::shape::ComponentShape>::RequiredValuePolicy,
                        #field_type,
                    >();
                }
            }
        } else {
            quote! {}
        };

        Some(quote_spanned! {span=>
            {
                fn __gpui_form_assert_component_shape<Shape: #runtime_crate::shape::ComponentShape>() {}
                __gpui_form_assert_component_shape::<#shape>();
                #value_binding_assertion
                #value_holder_storage_assertion
            }
        })
    }

    pub fn required_value(&self, field_type: &syn::Type) -> RequiredValue {
        RequiredValue::shape(self.resolved_shape(field_type))
    }

    fn validate_field_type(&self, field_type: &syn::Type) -> Result<(), syn::Error> {
        if !is_combobox_with_inferred_item(&self.shape) {
            return Ok(());
        }

        let Some(item_type) = vec_inner_type(field_type) else {
            return Ok(());
        };

        let field_type = type_display(field_type);
        let item_type = type_display(item_type);
        Err(syn::Error::new(
            self.span,
            format!(
                "`Combobox::<_>` inferred `{field_type}`; use `Combobox::<{item_type}>` for `{field_type}` fields"
            ),
        ))
    }
}

fn is_combobox_with_inferred_item(path: &Path) -> bool {
    let Some(segment) = path.segments.last() else {
        return false;
    };
    if segment.ident != "Combobox" {
        return false;
    }

    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return false;
    };

    args.args
        .first()
        .is_some_and(|arg| matches!(arg, syn::GenericArgument::Type(Type::Infer(_))))
}

fn vec_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if segment.ident != "Vec" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    args.args.iter().find_map(|arg| match arg {
        syn::GenericArgument::Type(inner) => Some(inner),
        _ => None,
    })
}

fn type_display(ty: &Type) -> String {
    ty.to_token_stream()
        .to_string()
        .replace(" :: ", "::")
        .replace(" < ", "<")
        .replace(" >", ">")
        .replace(" , ", ", ")
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

fn analyze_component_expr(expr: &Expr) -> darling::Result<(Path, Vec<ComponentMethod>)> {
    match expr {
        Expr::Group(group) => analyze_component_expr(&group.expr),
        Expr::Paren(paren) => analyze_component_expr(&paren.expr),
        Expr::MethodCall(method_call) => {
            let (shape, mut methods) = analyze_component_expr(&method_call.receiver)?;
            methods.push(ComponentMethod {
                method: method_call.method.clone(),
                args: method_call.args.iter().cloned().collect(),
            });
            Ok((shape, methods))
        },
        Expr::Call(call) => analyze_component_call_expr(call),
        Expr::Path(path) => Ok((path.path.clone(), Vec::new())),
        Expr::Lit(expr_lit) => {
            Err(DarlingError::unexpected_lit_type(&expr_lit.lit).with_span(&expr_lit.lit))
        },
        _ => Err(DarlingError::custom(
            "component syntax expects a shape path or shape metadata expression",
        )
        .with_span(expr)),
    }
}

fn analyze_component_call_expr(
    call: &syn::ExprCall,
) -> darling::Result<(Path, Vec<ComponentMethod>)> {
    let func = match &*call.func {
        Expr::Group(group) => &group.expr,
        Expr::Paren(paren) => &paren.expr,
        other => other,
    };

    let Expr::Path(path_expr) = func else {
        return Err(DarlingError::custom(
            "component call must be an associated function on the shape path",
        )
        .with_span(func));
    };

    let mut shape = path_expr.path.clone();
    let method = pop_component_method(&mut shape)?;
    Ok((
        shape,
        vec![ComponentMethod {
            method,
            args: call.args.iter().cloned().collect(),
        }],
    ))
}

fn pop_component_method(path: &mut Path) -> darling::Result<syn::Ident> {
    let method = path
        .segments
        .last()
        .map(|segment| segment.ident.clone())
        .ok_or_else(|| DarlingError::custom("component expression requires a shape path"))?;

    path.segments.pop();
    path.segments.pop_punct();
    if path.segments.is_empty() {
        return Err(DarlingError::custom(
            "component expression requires a shape path before the metadata method",
        )
        .with_span(&method));
    }

    Ok(method)
}

fn normalize_shape_path(mut path: Path) -> Path {
    for segment in &mut path.segments {
        if let syn::PathArguments::AngleBracketed(args) = &mut segment.arguments {
            args.colon2_token = None;
        }
    }

    path
}

fn expect_type_arg(method: &syn::Ident, args: &[Expr]) -> darling::Result<Type> {
    let [arg] = args else {
        return Err(
            DarlingError::custom(format!("`{method}` expects exactly one type argument"))
                .with_span(method),
        );
    };

    match arg {
        Expr::Path(path) => Ok(Type::Path(syn::TypePath {
            qself: path.qself.clone(),
            path: normalize_shape_path(path.path.clone()),
        })),
        Expr::Infer(infer) => Err(DarlingError::custom(
            "component type cannot be inferred with bare `_`; use an explicit component type",
        )
        .with_span(infer)),
        _ => Err(DarlingError::unexpected_expr_type(arg).with_span(arg)),
    }
}

fn expect_string_arg(method: &syn::Ident, args: &[Expr]) -> darling::Result<String> {
    let [arg] = args else {
        return Err(DarlingError::custom(format!(
            "`{method}` expects exactly one string literal argument"
        ))
        .with_span(method));
    };

    match arg {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(value) => Ok(value.value()),
            lit => Err(DarlingError::unexpected_lit_type(lit).with_span(arg)),
        },
        _ => Err(DarlingError::unexpected_expr_type(arg).with_span(arg)),
    }
}

fn set_metadata_once<T>(
    slot: &mut Option<T>,
    value: T,
    method: &syn::Ident,
) -> darling::Result<()> {
    if slot.is_some() {
        return Err(DarlingError::custom(format!(
            "duplicate `{method}` component metadata method; remove the duplicate `.{method}(...)` call"
        ))
        .with_span(method));
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

impl ComponentOption for ShapeOptions {}

pub struct ShapeComponent(pub FieldInformation<ShapeOptions>);

#[derive(Clone, Debug)]
pub enum Components {
    Shape(ShapeOptions),
}

impl Components {
    pub fn from_expr(expr: &Expr) -> darling::Result<Self> {
        ShapeOptions::from_component_expr(expr).map(Self::Shape)
    }

    pub fn required_value(&self, field_type: &syn::Type) -> RequiredValue {
        match self {
            Self::Shape(options) => options.required_value(field_type),
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
            Self::Shape(options) => {
                let options = options.clone().with_field_type(&field_type);
                let component = ShapeComponent(FieldInformation::new(
                    options,
                    field_name,
                    field_type.clone(),
                ));
                component.field_tokens(
                    &mut field_structure_tokens,
                    &mut field_base_declarations_tokens,
                );
            },
        }

        GeneratedFieldLayout {
            field_structure_tokens,
            field_base_declarations_tokens,
            required_value: self.required_value(&field_type),
        }
    }

    pub fn validate_field_type(&self, field_type: &syn::Type) -> Result<(), syn::Error> {
        match self {
            Self::Shape(options) => options.validate_field_type(field_type),
        }
    }

    pub fn component_type_tokens(&self, field_type: &syn::Type) -> Option<TokenStream> {
        let Self::Shape(options) = self;

        let shape = options.resolved_shape(field_type);
        let runtime_crate = CratePaths::resolve().gpui_form_runtime;
        if let Some(component) = options.component.as_ref() {
            let component_str = component.to_token_stream().to_string();
            Some(quote! { .with_component_type(#component_str) })
        } else {
            Some(quote! {
                .with_component_type_opt(
                    <#shape as #runtime_crate::shape::ComponentShape>::COMPONENT_TYPE
                )
            })
        }
    }

    pub fn shape_path_tokens(&self, field_type: &syn::Type) -> Option<TokenStream> {
        let Self::Shape(options) = self;

        let shape = options
            .resolved_shape(field_type)
            .to_token_stream()
            .to_string();
        Some(quote! { .with_shape_path(#shape) })
    }

    pub fn value_binding_tokens(&self, field_type: &syn::Type) -> Option<TokenStream> {
        let Self::Shape(options) = self;

        let shape = options.resolved_shape(field_type);
        let runtime_crate = CratePaths::resolve().gpui_form_runtime;

        Some(quote! {
            .with_value_binding(
                <#shape as #runtime_crate::shape::ComponentShape>::VALUE_BINDING
            )
        })
    }

    pub fn prototyping_tokens(
        &self,
        _field_name: &str,
        field_type: &syn::Type,
    ) -> Option<TokenStream> {
        let Self::Shape(options) = self;

        if let Some(field_suffix) = &options.field_suffix {
            let field_suffix = syn::LitStr::new(field_suffix, proc_macro2::Span::call_site());
            Some(quote! {
                .with_prototyping_field_suffix(Some(#field_suffix))
            })
        } else {
            let shape = options.resolved_shape(field_type);
            let runtime_crate = CratePaths::resolve().gpui_form_runtime;

            Some(quote! {
                .with_prototyping_field_suffix(
                    <#shape as #runtime_crate::shape::ComponentShape>::PROTOTYPING
                        .field_suffix
                )
            })
        }
    }

    pub fn type_check_tokens(
        &self,
        field_type: &syn::Type,
        check_value_holder_storage: bool,
    ) -> Option<TokenStream> {
        match self {
            Self::Shape(options) => {
                options.type_check_tokens(field_type, check_value_holder_storage)
            },
        }
    }
}
