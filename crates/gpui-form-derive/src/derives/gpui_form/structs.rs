use darling::{Error as DarlingError, FromField, FromMeta};
use gpui_form_codegen::components::{Components, RequiredValue};
use koruma_derive_core::ValidationInfo;
use proc_macro2::TokenStream;
use syn::{
    Expr, ExprGroup, ExprParen, ExprPath, Ident, Token, Type,
    parse::{Parse, ParseStream, Parser as _},
    punctuated::Punctuated,
};

#[derive(Clone, Debug)]
pub struct TypeOverride(pub Type);

#[derive(Clone, Debug)]
pub struct DefaultExpr(pub Expr);

/// Information about a field for value holder generation.
pub struct FieldOptionality {
    pub field_name: Ident,
    #[allow(dead_code)]
    pub original_type: Type,
    #[allow(dead_code)]
    pub inner_type: Type,
    pub was_optional: bool,
    pub required_value: RequiredValue,
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
    /// - Can be missing in the form holder
    /// - Must be present in the source struct
    /// - Are not nested structs (nested fields have their own validation)
    pub fn needs_required_validation(&self) -> bool {
        !self.skip
            && matches!(self.required_value, RequiredValue::Explicit(true))
            && !self.was_optional
            && !self.validation.is_nested
    }
}

#[derive(Clone, Debug)]
pub struct KorumaOptions {
    pub fluent: bool,
}

#[derive(Clone, Debug)]
pub struct KorumaField(pub KorumaOptions);

impl FromMeta for KorumaField {
    fn from_word() -> darling::Result<Self> {
        Ok(KorumaField(KorumaOptions { fluent: false }))
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let [darling::ast::NestedMeta::Meta(syn::Meta::Path(path))] = items else {
            return Err(DarlingError::custom(
                "`koruma(...)` only accepts `fluent`, as in `#[gpui_form(koruma(fluent))]`",
            ));
        };

        if path.is_ident("fluent") {
            Ok(KorumaField(KorumaOptions { fluent: true }))
        } else {
            Err(DarlingError::custom(
                "`koruma(...)` only accepts `fluent`, as in `#[gpui_form(koruma(fluent))]`",
            )
            .with_span(path))
        }
    }
}

#[derive(Clone, Debug)]
pub struct EmptyForm;

impl FromMeta for EmptyForm {
    fn from_word() -> darling::Result<Self> {
        Ok(Self)
    }
}

#[derive(Debug)]
enum GpuiFormFieldOption {
    Skip { span: Ident },
    Type { span: Token![type], ty: Type },
    Into { span: Ident, expr: Expr },
    From { span: Ident, expr: Expr },
    Default { span: Ident, expr: Expr },
    Component(Expr),
}

impl Parse for GpuiFormFieldOption {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(Token![type]) {
            let key = input.parse::<Token![type]>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::Type {
                span: key,
                ty: input.parse()?,
            });
        }

        if input.peek(Ident) {
            let fork = input.fork();
            let key: Ident = fork.parse()?;

            if fork.peek(Token![=]) {
                let key: Ident = input.parse()?;
                input.parse::<Token![=]>()?;

                return match key.to_string().as_str() {
                    "into" => Ok(Self::Into {
                        span: key,
                        expr: input.parse()?,
                    }),
                    "from" => Ok(Self::From {
                        span: key,
                        expr: input.parse()?,
                    }),
                    "default" => Ok(Self::Default {
                        span: key,
                        expr: input.parse()?,
                    }),
                    other => Err(syn::Error::new_spanned(
                        key,
                        format!("unknown gpui_form field option `{other}`"),
                    )),
                };
            }

            if fork.peek(syn::token::Paren) {
                return Err(syn::Error::new_spanned(
                    key.clone(),
                    format!("unknown gpui_form field option `{key}`"),
                ));
            }

            if key == "skip" {
                input.parse::<Ident>()?;
                return Ok(Self::Skip { span: key });
            }
        }

        Ok(Self::Component(input.parse()?))
    }
}

#[derive(Debug)]
pub struct ComponentField {
    pub ident: Option<Ident>,
    pub ty: Type,
    r#type: Option<TypeOverride>,
    into: Option<Expr>,
    from: Option<Expr>,
    component: Option<Components>,
    default: Option<DefaultExpr>,
    skip: bool,
    skip_seen: bool,
}

pub struct SkippedField;

pub struct RenderedField<'a> {
    pub r#type: Option<&'a TypeOverride>,
    pub into: Option<&'a Expr>,
    pub from: Option<&'a Expr>,
    pub component: Option<&'a Components>,
    pub default: Option<&'a DefaultExpr>,
}

pub enum ComponentFieldIntent<'a> {
    Skipped(SkippedField),
    Rendered(RenderedField<'a>),
}

impl ComponentField {
    pub fn intent(&self) -> ComponentFieldIntent<'_> {
        if self.skip {
            ComponentFieldIntent::Skipped(SkippedField)
        } else {
            ComponentFieldIntent::Rendered(RenderedField {
                r#type: self.r#type.as_ref(),
                into: self.into.as_ref(),
                from: self.from.as_ref(),
                component: self.component.as_ref(),
                default: self.default.as_ref(),
            })
        }
    }

    pub fn rendered(&self) -> Option<RenderedField<'_>> {
        match self.intent() {
            ComponentFieldIntent::Skipped(_) => None,
            ComponentFieldIntent::Rendered(rendered) => Some(rendered),
        }
    }

    pub fn skip(&self) -> bool {
        matches!(self.intent(), ComponentFieldIntent::Skipped(_))
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
            skip_seen: false,
        };

        for attr in &field.attrs {
            if !attr.path().is_ident("gpui_form") {
                continue;
            }

            let list = attr.meta.require_list().map_err(|_| {
                DarlingError::custom(
                    "`gpui_form` field attribute must be a list, for example \
                     `#[gpui_form(my::Shape)]`",
                )
                .with_span(attr)
            })?;

            let items = Punctuated::<GpuiFormFieldOption, syn::Token![,]>::parse_terminated
                .parse2(list.tokens.clone())
                .map_err(DarlingError::from)?;

            for item in items {
                parse_gpui_form_item(&mut parsed, item)?;
            }
        }

        validate_field_intent(&parsed)?;

        Ok(parsed)
    }
}

fn parse_gpui_form_item(
    field: &mut ComponentField,
    item: GpuiFormFieldOption,
) -> darling::Result<()> {
    match item {
        GpuiFormFieldOption::Skip { span } => set_skip(field, true, &span),
        GpuiFormFieldOption::Type { span, ty } => {
            ensure_not_skipped(field, "type", &span)?;
            set_once(&mut field.r#type, TypeOverride(ty), "type", &span)
        },
        GpuiFormFieldOption::Into { span, expr } => {
            ensure_not_skipped(field, "into", &span)?;
            set_once(&mut field.into, expr, "into", &span)
        },
        GpuiFormFieldOption::From { span, expr } => {
            ensure_not_skipped(field, "from", &span)?;
            set_once(&mut field.from, expr, "from", &span)
        },
        GpuiFormFieldOption::Default { span, expr } => {
            ensure_not_skipped(field, "default", &span)?;
            set_once(&mut field.default, DefaultExpr(expr), "default", &span)
        },
        GpuiFormFieldOption::Component(expr) => parse_gpui_form_expression(field, expr),
    }
}

fn parse_gpui_form_expression(field: &mut ComponentField, expr: Expr) -> darling::Result<()> {
    let expr = unwrap_grouped_expr(expr);

    match expr {
        Expr::Path(path) => parse_path_keyword(field, path),
        _ => {
            let component = Components::from_expr(&expr)?;
            set_component(field, component, &expr)
        },
    }
}

fn parse_path_keyword(field: &mut ComponentField, path: ExprPath) -> darling::Result<()> {
    let key = match path.path.get_ident() {
        Some(ident) => ident.to_string(),
        None => {
            return parse_positional_component(field, Expr::Path(path));
        },
    };

    if key == "skip" {
        set_skip(field, true, &path)?;
        return Ok(());
    }

    parse_positional_component(field, Expr::Path(path))
}

fn parse_positional_component(field: &mut ComponentField, expr: Expr) -> darling::Result<()> {
    ensure_not_skipped(field, "component", &expr)?;
    let component = Components::from_expr(&expr)?;
    set_component(field, component, &expr)
}

fn set_component<T: syn::spanned::Spanned>(
    field: &mut ComponentField,
    component: Components,
    span: &T,
) -> darling::Result<()> {
    ensure_not_skipped(field, "component", span)?;

    if field.component.is_some() {
        return Err(DarlingError::custom(
            "multiple component expressions were provided; keep a single shape \
             expression, such as `#[gpui_form(my::Shape)]`",
        )
        .with_span(span));
    }

    field.component = Some(component);
    Ok(())
}

fn set_once<T, S: syn::spanned::Spanned>(
    slot: &mut Option<T>,
    value: T,
    key: &str,
    span: &S,
) -> darling::Result<()> {
    if slot.is_some() {
        return Err(DarlingError::custom(format!(
            "duplicate `{key}` option in `gpui_form` field attribute; remove the duplicate `{key}` entry"
        ))
        .with_span(span));
    }

    *slot = Some(value);
    Ok(())
}

fn set_skip<S: syn::spanned::Spanned>(
    field: &mut ComponentField,
    value: bool,
    span: &S,
) -> darling::Result<()> {
    if field.skip_seen {
        return Err(DarlingError::custom(
            "duplicate `skip` option in `gpui_form` field attribute; remove the duplicate `skip` entry",
        )
        .with_span(span));
    }

    if value {
        validate_no_skip_conflicts(field, span)?;
    }

    field.skip = value;
    field.skip_seen = true;
    Ok(())
}

fn ensure_not_skipped<S: syn::spanned::Spanned>(
    field: &ComponentField,
    option: &str,
    span: &S,
) -> darling::Result<()> {
    if field.skip {
        return Err(skip_conflict_error(option).with_span(span));
    }

    Ok(())
}

fn skip_conflict_error(option: &str) -> DarlingError {
    DarlingError::custom(format!(
        "`skip` cannot be combined with `{option}` in a `gpui_form` field attribute; \
         remove `skip` or remove the `{option}` option"
    ))
}

fn validate_no_skip_conflicts<S: syn::spanned::Spanned>(
    field: &ComponentField,
    span: &S,
) -> darling::Result<()> {
    let conflicts = [
        ("component", field.component.is_some()),
        ("default", field.default.is_some()),
        ("type", field.r#type.is_some()),
        ("from", field.from.is_some()),
        ("into", field.into.is_some()),
    ];

    if let Some((option, _)) = conflicts.into_iter().find(|(_, present)| *present) {
        return Err(skip_conflict_error(option).with_span(span));
    }

    Ok(())
}

fn validate_field_intent(field: &ComponentField) -> darling::Result<()> {
    if field.skip {
        validate_no_skip_conflicts(field, &field.ident)?;
    }

    Ok(())
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
    pub empty: Option<EmptyForm>,
    #[darling(default)]
    pub koruma: Option<KorumaField>,
}

pub struct ComponentFieldContent {
    pub field_structure_tokens: TokenStream,
    pub field_base_declarations_tokens: TokenStream,
    pub required_value: (String, RequiredValue),
}

pub struct GpuiFormOptions {
    pub generate_shape: bool,
}
