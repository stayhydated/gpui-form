use darling::{Error as DarlingError, FromField, FromMeta, ast::NestedMeta};
use gpui_form_codegen::components::Components;
use koruma_derive_core::ValidationInfo;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    Expr, ExprAssign, ExprCall, ExprGroup, ExprParen, ExprPath, Ident, Lit, MacroDelimiter, Meta,
    MetaList, Type, TypePath,
    parse::{Parse, ParseStream, Parser as _, discouraged::Speculative},
    punctuated::Punctuated,
    token::Paren,
};

#[derive(Clone, Debug)]
pub struct TypeOverride(pub Type);

impl FromMeta for TypeOverride {
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        match expr {
            Expr::Path(expr_path) => Ok(TypeOverride(Type::Path(TypePath {
                qself: expr_path.qself.clone(),
                path: expr_path.path.clone(),
            }))),
            Expr::Group(group) => Self::from_expr(&group.expr),
            Expr::Lit(expr_lit) => Self::from_value(&expr_lit.lit),
            _ => Err(DarlingError::unexpected_expr_type(expr)),
        }
    }

    fn from_string(value: &str) -> darling::Result<Self> {
        syn::parse_str::<Type>(value)
            .map(TypeOverride)
            .map_err(|_| DarlingError::unknown_value(value))
    }

    fn from_value(value: &Lit) -> darling::Result<Self> {
        if let Lit::Str(v) = value {
            v.parse::<Type>()
                .map(TypeOverride)
                .map_err(|_| DarlingError::unknown_value(&v.value()).with_span(v))
        } else {
            Err(DarlingError::unexpected_lit_type(value))
        }
    }
}

#[derive(Clone, Debug)]
pub struct DefaultExpr(pub Expr);

impl FromMeta for DefaultExpr {
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        Ok(DefaultExpr(expr.clone()))
    }

    fn from_string(value: &str) -> darling::Result<Self> {
        syn::parse_str::<Expr>(value)
            .map(DefaultExpr)
            .map_err(|_| DarlingError::unknown_value(value))
    }

    fn from_value(value: &Lit) -> darling::Result<Self> {
        Ok(DefaultExpr(Expr::Lit(syn::ExprLit {
            attrs: Vec::new(),
            lit: value.clone(),
        })))
    }
}

/// Information about a field for value holder generation.
pub struct FieldOptionality {
    pub field_name: Ident,
    #[allow(dead_code)]
    pub original_type: Type,
    #[allow(dead_code)]
    pub inner_type: Type,
    pub was_optional: bool,
    pub wrap_in_option: bool,
    pub validation: ValidationInfo,
    pub default_expr: Option<Expr>,
    pub override_type: Option<Type>,
    pub into_expr: Option<Expr>,
    pub from_expr: Option<Expr>,
    pub skip: bool,
}

impl FieldOptionality {
    /// Returns true if this field needs the `RequiredValidation` koruma validator.
    /// This applies to fields that:
    /// - Are wrapped in Option (for form handling)
    /// - Were not originally Optional in the source struct
    /// - Are not nested structs (nested fields have their own validation)
    pub fn needs_required_validation(&self) -> bool {
        !self.skip && self.wrap_in_option && !self.was_optional && !self.validation.is_nested
    }
}

#[derive(Clone, Debug, Default, FromMeta)]
pub struct KorumaOptions {
    #[darling(default)]
    pub fluent: bool,
}

#[derive(Clone, Debug)]
pub struct KorumaField(pub KorumaOptions);

impl FromMeta for KorumaField {
    fn from_word() -> darling::Result<Self> {
        Ok(KorumaField(KorumaOptions::default()))
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        KorumaOptions::from_list(items).map(KorumaField)
    }
}

#[derive(Debug)]
enum GpuiFormArg {
    Meta(NestedMeta),
    Expr(Expr),
}

impl Parse for GpuiFormArg {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let fork = input.fork();
        if let Ok(meta) = fork.parse::<NestedMeta>() {
            if fork.is_empty() || fork.peek(syn::Token![,]) {
                input.advance_to(&fork);
                return Ok(Self::Meta(meta));
            }
        }

        let expr = input.parse::<Expr>()?;
        Ok(Self::Expr(expr))
    }
}

#[derive(Debug)]
pub struct ComponentField {
    pub ident: Option<Ident>,
    pub ty: Type,
    pub r#type: Option<TypeOverride>,
    pub into: Option<Expr>,
    pub from: Option<Expr>,
    pub component: Option<Components>,
    pub default: Option<DefaultExpr>,
    pub skip: bool,
}

impl ComponentField {
    pub fn skip(&self) -> bool {
        self.skip
    }
}

impl FromField for ComponentField {
    fn from_field(field: &syn::Field) -> darling::Result<Self> {
        let mut parsed = ComponentField {
            ident: field.ident.clone(),
            ty: field.ty.clone(),
            r#type: None,
            into: None,
            from: None,
            component: None,
            default: None,
            skip: false,
        };

        for attr in &field.attrs {
            if !attr.path().is_ident("gpui_form") {
                continue;
            }

            let Some(list) = attr.meta.require_list().ok() else {
                continue;
            };

            let items = Punctuated::<GpuiFormArg, syn::Token![,]>::parse_terminated
                .parse2(list.tokens.clone())
                .map_err(|err| DarlingError::custom(err.to_string()).with_span(list))?;

            for item in items {
                parse_gpui_form_item(&mut parsed, item)?;
            }
        }

        Ok(parsed)
    }
}

fn parse_gpui_form_item(field: &mut ComponentField, item: GpuiFormArg) -> darling::Result<()> {
    match item {
        GpuiFormArg::Meta(meta) => parse_gpui_form_meta(field, meta),
        GpuiFormArg::Expr(expr) => parse_gpui_form_expression(field, expr),
    }
}

fn parse_gpui_form_meta(field: &mut ComponentField, meta: NestedMeta) -> darling::Result<()> {
    match meta {
        NestedMeta::Meta(meta) => match meta {
            Meta::Path(path) => parse_meta_path_keyword(field, path),
            Meta::List(list) => parse_meta_list(field, list),
            Meta::NameValue(name_value) => parse_meta_name_value(field, name_value),
        },
        NestedMeta::Lit(literal) => {
            Err(DarlingError::unsupported_format("key-value pair").with_span(&literal))
        },
    }
}

fn parse_gpui_form_expression(field: &mut ComponentField, expr: Expr) -> darling::Result<()> {
    let expr = unwrap_grouped_expr(expr);

    match expr {
        Expr::Assign(ExprAssign { left, right, .. }) => parse_assignment(field, *left, *right),
        Expr::Path(path) => parse_path_keyword(field, path),
        Expr::Call(call) if is_path_ident(call.func.as_ref(), "component") => {
            field.component = Some(parse_component_call_expr(&call)?);
            Ok(())
        },
        _ => {
            if field.component.is_some() {
                return Err(DarlingError::custom(
                    "expected a valid `gpui_form` key-value pair or one component expression",
                ));
            }

            field.component = Some(Components::from_expr(&expr)?);
            Ok(())
        },
    }
}

fn parse_meta_name_value(
    field: &mut ComponentField,
    name_value: syn::MetaNameValue,
) -> darling::Result<()> {
    let key = meta_path_to_key(&name_value.path)?;
    let rhs = unwrap_grouped_expr(name_value.value);

    match key.as_str() {
        "type" => {
            field.r#type = Some(TypeOverride::from_expr(&rhs)?);
            Ok(())
        },
        "into" => {
            field.into = Some(rhs);
            Ok(())
        },
        "from" => {
            field.from = Some(rhs);
            Ok(())
        },
        "component" => {
            field.component = Some(parse_component_expr(rhs)?);
            Ok(())
        },
        "default" => {
            field.default = Some(DefaultExpr::from_expr(&rhs)?);
            Ok(())
        },
        "skip" => {
            field.skip = parse_bool_expr(&rhs)?;
            Ok(())
        },
        _ => Err(
            DarlingError::custom(format!("unknown gpui_form field option `{key}`")).with_span(&rhs),
        ),
    }
}

fn parse_assignment(field: &mut ComponentField, lhs: Expr, rhs: Expr) -> darling::Result<()> {
    let key = expr_to_key(&lhs)?;
    let rhs = unwrap_grouped_expr(rhs);

    match key.as_str() {
        "type" => {
            field.r#type = Some(TypeOverride::from_expr(&rhs)?);
            Ok(())
        },
        "into" => {
            field.into = Some(rhs);
            Ok(())
        },
        "from" => {
            field.from = Some(rhs);
            Ok(())
        },
        "component" => {
            field.component = Some(parse_component_expr(rhs)?);
            Ok(())
        },
        "default" => {
            field.default = Some(DefaultExpr::from_expr(&rhs)?);
            Ok(())
        },
        "skip" => {
            field.skip = parse_bool_expr(&rhs)?;
            Ok(())
        },
        _ => Err(
            DarlingError::custom(format!("unknown gpui_form field option `{key}`")).with_span(&rhs),
        ),
    }
}

fn parse_meta_list(field: &mut ComponentField, list: MetaList) -> darling::Result<()> {
    if list.path.is_ident("component") {
        field.component = Some(parse_component_list(list)?);
        return Ok(());
    }

    let key = meta_path_to_key(&list.path)?;
    Err(
        DarlingError::custom(format!("unknown gpui_form field option `{key}`"))
            .with_span(&list.path),
    )
}

fn parse_component_list(list: MetaList) -> darling::Result<Components> {
    let args = Punctuated::<NestedMeta, syn::Token![,]>::parse_terminated
        .parse2(list.tokens.clone())
        .map_err(|err| DarlingError::custom(err.to_string()).with_span(&list))?;

    let mut args = args.into_iter();
    let Some(custom_arg) = args.next() else {
        return Err(DarlingError::custom(
            "`component(...)` expects one `custom(...)` argument",
        ));
    };

    if args.next().is_some() {
        return Err(DarlingError::custom(
            "`component(...)` expects exactly one argument",
        ));
    }

    let NestedMeta::Meta(Meta::List(custom_list)) = custom_arg else {
        return Err(DarlingError::custom(
            "`component(...)` currently supports `custom(...)` only",
        ));
    };

    if !is_meta_path_ident(&custom_list.path, "custom") {
        return Err(DarlingError::custom(
            "`component(...)` currently supports `custom(...)` only",
        ));
    }

    Components::from_list(&[NestedMeta::Meta(Meta::List(custom_list))])
}

fn parse_meta_path_keyword(field: &mut ComponentField, path: syn::Path) -> darling::Result<()> {
    let Some(ident) = path.get_ident() else {
        return parse_positional_component(
            field,
            Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path,
            }),
        );
    };

    let key = ident.to_string();
    if key == "skip" {
        field.skip = true;
        return Ok(());
    }

    if key == "component" {
        return Err(DarlingError::custom(
            "`component` must include a value, for example `component = ...`",
        ));
    }

    parse_positional_component(
        field,
        Expr::Path(ExprPath {
            attrs: Vec::new(),
            qself: None,
            path,
        }),
    )
}

fn parse_path_keyword(field: &mut ComponentField, path: ExprPath) -> darling::Result<()> {
    let key = match path.path.get_ident() {
        Some(ident) => ident.to_string(),
        None => {
            return parse_positional_component(field, Expr::Path(path));
        },
    };

    if key == "skip" {
        field.skip = true;
        return Ok(());
    }

    if key == "component" {
        return Err(DarlingError::custom(
            "`component` must include a value, for example `component = ...`",
        ));
    }

    parse_positional_component(field, Expr::Path(path))
}

fn parse_positional_component(field: &mut ComponentField, expr: Expr) -> darling::Result<()> {
    if field.component.is_some() {
        return Err(DarlingError::custom(
            "multiple component expressions were provided",
        ));
    }

    field.component = Some(Components::from_expr(&expr)?);
    Ok(())
}

fn parse_component_expr(expr: Expr) -> darling::Result<Components> {
    let expr = unwrap_grouped_expr(expr);
    match expr {
        Expr::Call(call) if is_path_ident(call.func.as_ref(), "component") => {
            parse_component_call_expr(&call)
        },
        _ => Components::from_expr(&expr),
    }
}

fn parse_component_call_expr(expr: &ExprCall) -> darling::Result<Components> {
    let mut args = expr.args.iter();
    let Some(custom_shape_arg) = args.next() else {
        return Err(DarlingError::custom(
            "`component(...)` expects one `custom(...)` argument",
        ));
    };

    if args.next().is_some() {
        return Err(DarlingError::custom(
            "`component(...)` expects exactly one argument",
        ));
    }

    let Expr::Call(custom_call) = custom_shape_arg else {
        return Err(DarlingError::custom(
            "`component(...)` currently supports `custom(...)` only",
        ));
    };

    if !is_path_ident(custom_call.func.as_ref(), "custom") {
        return Err(DarlingError::custom(
            "`component(...)` currently supports `custom(...)` only",
        ));
    }

    let custom_path = match custom_call.func.as_ref() {
        Expr::Path(expr_path) => expr_path.path.clone(),
        _ => {
            return Err(DarlingError::custom(
                "`custom` component syntax requires a path",
            ));
        },
    };
    let list = MetaList {
        path: custom_path,
        delimiter: MacroDelimiter::Paren(Paren::default()),
        tokens: custom_call.args.to_token_stream(),
    };
    Components::from_list(&[NestedMeta::Meta(Meta::List(list))])
}

fn parse_bool_expr(expr: &Expr) -> darling::Result<bool> {
    if let Expr::Lit(expr_lit) = expr {
        if let syn::Lit::Bool(value) = &expr_lit.lit {
            return Ok(value.value);
        }
    }

    Err(DarlingError::unexpected_expr_type(expr))
}

fn expr_to_key(expr: &Expr) -> darling::Result<String> {
    if let Expr::Path(path) = expr {
        path.path
            .get_ident()
            .map(|ident| ident.to_string())
            .ok_or_else(|| DarlingError::unsupported_format("field key"))
    } else {
        Err(DarlingError::custom("field key must be an identifier").with_span(expr))
    }
}

fn is_path_ident(expr: &Expr, key: &str) -> bool {
    if let Expr::Path(path) = expr {
        path.path.is_ident(key)
    } else {
        false
    }
}

fn is_meta_path_ident(path: &syn::Path, key: &str) -> bool {
    path.is_ident(key)
}

fn meta_path_to_key(path: &syn::Path) -> darling::Result<String> {
    path.get_ident()
        .map(|ident| ident.to_string())
        .ok_or_else(|| DarlingError::unsupported_format("field key").with_span(path))
}

fn unwrap_grouped_expr(expr: Expr) -> Expr {
    let mut expr = expr;
    loop {
        match expr {
            Expr::Group(ExprGroup { expr: inner, .. }) => expr = *inner,
            Expr::Paren(ExprParen { expr: inner, .. }) => expr = *inner,
            _ => return expr,
        }
    }
}

#[derive(Debug, darling::FromDeriveInput)]
#[darling(attributes(gpui_form), supports(struct_named, struct_unit))]
pub struct ComponentStruct {
    pub ident: Ident,
    pub data: darling::ast::Data<(), ComponentField>,
    #[darling(default)]
    pub empty: bool,
    #[darling(default)]
    pub koruma: Option<KorumaField>,
}

pub struct ComponentFieldContent {
    pub field_structure_tokens: TokenStream,
    pub field_base_declarations_tokens: TokenStream,
    pub wrap_in_option: (String, bool),
}

pub struct GpuiFormOptions {
    pub generate_shape: bool,
}
