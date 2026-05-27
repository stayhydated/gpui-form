use darling::{Error as DarlingError, FromMeta};
use proc_macro2::TokenStream;
use quote::{ToTokens as _, quote};
use syn::{Expr, Lit, Path};

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
                quote! {
                    <#shape as ::gpui_form_runtime::shape::ComponentShape>::REQUIRES_VALUE
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
    /// UI component type path (e.g. `TagsInput`).
    /// When provided, the prototyping code generator emits `Component::new(&entity)`.
    pub component: Option<syn::Path>,
    /// Whether prototyping code should wire this component through
    /// `ComponentValueBinding`.
    pub value_binding: Option<bool>,
    /// Optional explicit generated field/helper suffix.
    pub field_suffix: Option<String>,
}

impl ShapeOptions {
    fn from_shape(shape: Path) -> Self {
        let shape = normalize_shape_path(shape);

        Self {
            shape,
            component: None,
            value_binding: None,
            field_suffix: None,
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
            "value_binding" => {
                self.value_binding = Some(expect_optional_bool_arg(method, args)?);
            },
            "component" => {
                self.component = Some(expect_path_arg(method, args)?);
            },
            "field_suffix" => {
                self.field_suffix = Some(expect_string_arg(method, args)?);
            },
            _ => {
                return Err(DarlingError::custom(format!(
                    "unknown component metadata `{method}`; supported generic methods are \
                     `value_binding`, `component`, and `field_suffix`; put component-specific \
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

        let shape = self.shape.to_token_stream().to_string();
        gpui_form_schema::registry::component_suffix_from_shape(field_name, &shape)
            .unwrap_or_else(|| "shape".to_string())
    }

    pub fn constructor_tokens(&self, field_type: &syn::Type) -> TokenStream {
        let shape = self.runtime_shape(field_type);

        quote! {
            <#shape as ::gpui_form_runtime::shape::ComponentShape>::new(window, cx)
        }
    }

    pub fn type_check_tokens(&self, field_type: &syn::Type) -> Option<TokenStream> {
        let shape = self.resolved_shape(field_type);

        Some(quote! {
            {
                fn __gpui_form_assert_component_shape<Shape: ::gpui_form_runtime::shape::ComponentShape>() {}
                __gpui_form_assert_component_shape::<#shape>();
            }
        })
    }

    pub fn required_value(&self, field_type: &syn::Type) -> RequiredValue {
        RequiredValue::shape(self.resolved_shape(field_type))
    }
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
        Expr::Lit(expr_lit) => Err(DarlingError::custom(
            "component string syntax was removed; use a shape path expression",
        )
        .with_span(&expr_lit.lit)),
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

fn expect_bool_arg(method: &syn::Ident, args: &[Expr]) -> darling::Result<bool> {
    let [arg] = args else {
        return Err(DarlingError::custom(format!(
            "`{method}` expects exactly one boolean argument"
        ))
        .with_span(method));
    };

    match arg {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Bool(value) => Ok(value.value),
            lit => Err(DarlingError::unexpected_lit_type(lit).with_span(arg)),
        },
        _ => Err(DarlingError::unexpected_expr_type(arg).with_span(arg)),
    }
}

fn expect_optional_bool_arg(method: &syn::Ident, args: &[Expr]) -> darling::Result<bool> {
    match args {
        [] => Ok(true),
        [_] => expect_bool_arg(method, args),
        _ => Err(DarlingError::custom(format!(
            "`{method}` expects zero arguments or one boolean argument"
        ))
        .with_span(method)),
    }
}

fn expect_path_arg(method: &syn::Ident, args: &[Expr]) -> darling::Result<Path> {
    let [arg] = args else {
        return Err(
            DarlingError::custom(format!("`{method}` expects exactly one path argument"))
                .with_span(method),
        );
    };

    match arg {
        Expr::Path(path) => Ok(path.path.clone()),
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(value) => value
                .parse::<Path>()
                .map_err(|err| DarlingError::custom(err.to_string()).with_span(value)),
            lit => Err(DarlingError::unexpected_lit_type(lit).with_span(arg)),
        },
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

impl FromMeta for Components {
    fn from_word() -> darling::Result<Self> {
        Err(DarlingError::custom(
            "component requires a shape expression, for example \
             `component = my::Shape`",
        ))
    }

    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        ShapeOptions::from_component_expr(expr).map(Self::Shape)
    }

    fn from_string(_value: &str) -> darling::Result<Self> {
        Err(DarlingError::custom(
            "component string syntax was removed; use a shape path expression",
        ))
    }

    fn from_list(_items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        Err(DarlingError::custom(
            "component list syntax was removed; use `component = my::Shape`",
        ))
    }
}

impl Components {
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

    pub fn component_path_tokens(&self, field_type: &syn::Type) -> Option<TokenStream> {
        let Self::Shape(options) = self;

        let shape = options.resolved_shape(field_type);
        if let Some(component) = options.component.as_ref() {
            let component_str = component.to_token_stream().to_string();
            Some(quote! { .with_component_path(#component_str) })
        } else {
            Some(quote! {
                .with_component_path_opt(
                    <#shape as ::gpui_form_runtime::shape::ComponentShape>::COMPONENT_PATH
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

        Some(match options.value_binding {
            Some(true) => quote! { .with_value_binding(true) },
            Some(false) => quote! { .with_value_binding(false) },
            None => {
                quote! {
                    .with_value_binding(
                        <#shape as ::gpui_form_runtime::shape::ComponentShape>::VALUE_BINDING
                    )
                }
            },
        })
    }

    pub fn prototyping_tokens(&self, field_type: &syn::Type) -> Option<TokenStream> {
        let Self::Shape(options) = self;

        if let Some(field_suffix) = &options.field_suffix {
            let field_suffix = syn::LitStr::new(field_suffix, proc_macro2::Span::call_site());
            Some(quote! {
                .with_prototyping_field_suffix(Some(#field_suffix))
            })
        } else {
            let shape = options.resolved_shape(field_type);

            Some(quote! {
                .with_prototyping_field_suffix(
                    <#shape as ::gpui_form_runtime::shape::ComponentShape>::PROTOTYPING
                        .field_suffix
                )
            })
        }
    }

    pub fn type_check_tokens(&self, field_type: &syn::Type) -> Option<TokenStream> {
        match self {
            Self::Shape(options) => options.type_check_tokens(field_type),
        }
    }
}
