use heck::{ToKebabCase as _, ToPascalCase as _, ToSnakeCase as _};
use syn::{Expr, Ident, Path, Type};

use crate::registry::{
    ComponentCapabilities, FieldVariant, GpuiFormShape, RustSyntaxKind, ValidationRuleId,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolveError {
    InvalidIdentifier {
        kind: &'static str,
        value: String,
    },
    InvalidPath {
        kind: &'static str,
        value: String,
        error: String,
    },
    InvalidType {
        field_name: String,
        value: String,
        error: String,
    },
    InvalidExpression {
        field_name: String,
        value: String,
        error: String,
    },
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidIdentifier { kind, value } => {
                write!(f, "invalid {kind} `{value}` in gpui-form shape metadata")
            },
            Self::InvalidPath { kind, value, error } => {
                write!(
                    f,
                    "invalid {kind} `{value}` in gpui-form shape metadata: {error}"
                )
            },
            Self::InvalidType {
                field_name,
                value,
                error,
            } => {
                write!(
                    f,
                    "invalid value type `{value}` for field `{field_name}` in gpui-form shape metadata: {error}"
                )
            },
            Self::InvalidExpression {
                field_name,
                value,
                error,
            } => {
                write!(
                    f,
                    "invalid expression `{value}` for field `{field_name}` in gpui-form shape metadata: {error}"
                )
            },
        }
    }
}

impl std::error::Error for ResolveError {}

#[derive(Clone, Eq, PartialEq)]
pub struct ShapeName {
    source: &'static str,
    ident: Ident,
    form_ident: Ident,
    fields_ident: Ident,
    components_ident: Ident,
    value_holder_ident: Ident,
    form_id: String,
}

impl ShapeName {
    fn new(source: &'static str) -> Result<Self, ResolveError> {
        let ident = parse_ident("struct name", source)?;
        let form_ident = parse_ident("generated form ident", &format!("{source}Form"))?;
        let fields_ident = parse_ident(
            "generated form fields ident",
            &format!("{source}FormFields"),
        )?;
        let components_ident = parse_ident(
            "generated form components ident",
            &format!("{source}FormComponents"),
        )?;
        let value_holder_ident = parse_ident(
            "generated form value holder ident",
            &format!("{source}FormValueHolder"),
        )?;
        let form_id = format!("{}-form", source.to_snake_case());

        Ok(Self {
            source,
            ident,
            form_ident,
            fields_ident,
            components_ident,
            value_holder_ident,
            form_id,
        })
    }

    pub const fn source(&self) -> &'static str {
        self.source
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn form_ident(&self) -> &Ident {
        &self.form_ident
    }

    pub fn fields_ident(&self) -> &Ident {
        &self.fields_ident
    }

    pub fn components_ident(&self) -> &Ident {
        &self.components_ident
    }

    pub fn value_holder_ident(&self) -> &Ident {
        &self.value_holder_ident
    }

    pub fn form_id(&self) -> &str {
        &self.form_id
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct FieldIdent(Ident);

impl FieldIdent {
    fn new(kind: &'static str, value: &str) -> Result<Self, ResolveError> {
        parse_ident(kind, value).map(Self)
    }

    pub fn as_ident(&self) -> &Ident {
        &self.0
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ComponentFieldIdent(Ident);

impl ComponentFieldIdent {
    fn new(value: &str) -> Result<Self, ResolveError> {
        parse_ident("field component ident", value).map(Self)
    }

    pub fn as_ident(&self) -> &Ident {
        &self.0
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct KebabId(String);

impl KebabId {
    fn new(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct HandlerIdent(Ident);

impl HandlerIdent {
    fn new(value: &str) -> Result<Self, ResolveError> {
        parse_ident("field component event handler", value).map(Self)
    }

    pub fn as_ident(&self) -> &Ident {
        &self.0
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct FieldName {
    source: &'static str,
    ident: FieldIdent,
    pascal_ident: FieldIdent,
    component_ident: ComponentFieldIdent,
    kebab_id: KebabId,
    handler_ident: HandlerIdent,
}

impl FieldName {
    fn new(field: &FieldVariant) -> Result<Self, ResolveError> {
        let source = field.field_name();
        let ident = FieldIdent::new("field name", source)?;
        let pascal_ident = FieldIdent::new("field pascal ident", &source.to_pascal_case())?;
        let component_name = field.field_name_with_component_suffix();
        let component_ident = ComponentFieldIdent::new(&component_name)?;
        let kebab_id = KebabId::new(component_name.to_kebab_case());
        let handler_ident = HandlerIdent::new(&format!("on_{component_name}_event"))?;

        Ok(Self {
            source,
            ident,
            pascal_ident,
            component_ident,
            kebab_id,
            handler_ident,
        })
    }

    pub const fn source(&self) -> &'static str {
        self.source
    }

    pub fn ident(&self) -> &Ident {
        self.ident.as_ident()
    }

    pub fn pascal_ident(&self) -> &Ident {
        self.pascal_ident.as_ident()
    }

    pub fn component_ident(&self) -> &Ident {
        self.component_ident.as_ident()
    }

    pub fn kebab_id(&self) -> &str {
        self.kebab_id.as_str()
    }

    pub fn handler_ident(&self) -> &Ident {
        self.handler_ident.as_ident()
    }
}

#[derive(Clone)]
pub struct ResolvedField<'a> {
    raw: &'a FieldVariant,
    name: FieldName,
    value_type: Type,
    source_value_type: Type,
    shape_path: Option<Path>,
    default_expr: Option<Expr>,
    from_expr: Option<Expr>,
    into_expr: Option<Expr>,
    component_capabilities: Option<ComponentCapabilities>,
}

impl<'a> ResolvedField<'a> {
    pub fn new(field: &'a FieldVariant) -> Result<Self, ResolveError> {
        let name = FieldName::new(field)?;
        let value_type = parse_type(field.field_name(), field.value_type())?;
        let source_value_type = parse_type(field.field_name(), field.source_value_type())?;
        let shape_path = match field.shape_path() {
            Some(path) => Some(parse_path("component shape path", path)?),
            None => None,
        };
        let default_expr = match field.default_expr() {
            Some(expr) => Some(parse_expr(field.field_name(), expr)?),
            None => None,
        };
        let from_expr = match field.from_expr() {
            Some(expr) => Some(parse_expr(field.field_name(), expr)?),
            None => None,
        };
        let into_expr = match field.into_expr() {
            Some(expr) => Some(parse_expr(field.field_name(), expr)?),
            None => None,
        };
        let component_capabilities = field
            .component_variant()
            .map(|component| component.capabilities());

        Ok(Self {
            raw: field,
            name,
            value_type,
            source_value_type,
            shape_path,
            default_expr,
            from_expr,
            into_expr,
            component_capabilities,
        })
    }

    pub fn raw(&self) -> &'a FieldVariant {
        self.raw
    }

    pub fn field_name(&self) -> &'a str {
        self.name.source()
    }

    pub fn name(&self) -> &FieldName {
        &self.name
    }

    pub fn field_ident(&self) -> &Ident {
        self.name.ident()
    }

    pub fn field_ident_pascal(&self) -> &Ident {
        self.name.pascal_ident()
    }

    pub fn field_ident_with_component_suffix(&self) -> &Ident {
        self.name.component_ident()
    }

    pub fn value_type(&self) -> &Type {
        &self.value_type
    }

    pub fn source_value_type(&self) -> &Type {
        &self.source_value_type
    }

    pub fn shape_path(&self) -> Option<&Path> {
        self.shape_path.as_ref()
    }

    pub fn runtime_shape_path(&self) -> Option<Path> {
        self.shape_path.clone()
    }

    pub fn default_expr(&self) -> Option<&Expr> {
        self.default_expr.as_ref()
    }

    pub fn from_expr(&self) -> Option<&Expr> {
        self.from_expr.as_ref()
    }

    pub fn into_expr(&self) -> Option<&Expr> {
        self.into_expr.as_ref()
    }

    pub fn component_capabilities(&self) -> Option<ComponentCapabilities> {
        self.component_capabilities
    }

    pub fn kebab_id(&self) -> String {
        self.name.kebab_id().to_string()
    }

    pub fn suffixed_ident(&self, suffix: &str) -> Ident {
        parse_ident(
            "generated suffixed field ident",
            &format!("{}_{}", self.field_name(), suffix),
        )
        .expect("field names are validated idents")
    }

    pub fn prefixed_ident(&self, prefix: &str) -> Ident {
        parse_ident(
            "generated prefixed field ident",
            &format!("{}_{}", prefix, self.field_name()),
        )
        .expect("field names are validated idents")
    }

    pub fn component_event_handler_ident(&self) -> Ident {
        self.name.handler_ident().clone()
    }

    pub fn optional(&self) -> bool {
        self.raw.optional()
    }

    pub fn value_holder_uses_option(&self) -> bool {
        self.raw.value_holder_uses_option()
    }

    pub fn value_binding(&self) -> bool {
        self.component_capabilities
            .is_some_and(ComponentCapabilities::value_binding_enabled)
    }

    pub fn render_component(&self) -> bool {
        self.component_capabilities
            .is_some_and(ComponentCapabilities::render_component)
    }

    pub fn validation_rules(&self) -> &'static [ValidationRuleId] {
        self.raw.validation_rules()
    }

    pub fn has_validation_rule(&self, rule: ValidationRuleId) -> bool {
        self.validation_rules().contains(&rule)
    }

    pub fn uses_optional_inner_validation_errors(&self) -> bool {
        self.optional()
            && (self.has_validation_rule(ValidationRuleId::Newtype)
                || self.has_validation_rule(ValidationRuleId::Nested))
    }
}

#[derive(Clone)]
pub struct ResolvedGpuiFormShape<'a> {
    raw: &'a GpuiFormShape,
    name: ShapeName,
    source_module_path: Path,
    fields: Vec<ResolvedField<'a>>,
}

impl<'a> ResolvedGpuiFormShape<'a> {
    pub fn new(shape: &'a GpuiFormShape) -> Result<Self, ResolveError> {
        let name = ShapeName::new(shape.struct_name)?;
        let source_module_path = parse_path("source module path", shape.source_module_path)?;
        let fields = shape
            .fields
            .iter()
            .map(ResolvedField::new)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            raw: shape,
            name,
            source_module_path,
            fields,
        })
    }

    pub fn raw(&self) -> &'a GpuiFormShape {
        self.raw
    }

    pub fn name(&self) -> &ShapeName {
        &self.name
    }

    pub fn source_module_path(&self) -> &Path {
        &self.source_module_path
    }

    pub fn fields(&self) -> &[ResolvedField<'a>] {
        &self.fields
    }
}

fn parse_ident(kind: &'static str, value: &str) -> Result<Ident, ResolveError> {
    syn::parse_str::<Ident>(value).map_err(|_| ResolveError::InvalidIdentifier {
        kind,
        value: value.to_string(),
    })
}

fn parse_path(kind: &'static str, value: crate::registry::RustPath) -> Result<Path, ResolveError> {
    value.parse().map_err(|error| ResolveError::InvalidPath {
        kind,
        value: error.value().to_string(),
        error: error.source_error().to_string(),
    })
}

fn parse_type(field_name: &str, value: crate::registry::RustType) -> Result<Type, ResolveError> {
    value.parse().map_err(|error| {
        debug_assert_eq!(error.kind(), RustSyntaxKind::Type);
        ResolveError::InvalidType {
            field_name: field_name.to_string(),
            value: error.value().to_string(),
            error: error.source_error().to_string(),
        }
    })
}

fn parse_expr(field_name: &str, value: crate::registry::RustExpr) -> Result<Expr, ResolveError> {
    value.parse().map_err(|error| {
        debug_assert_eq!(error.kind(), RustSyntaxKind::Expr);
        ResolveError::InvalidExpression {
            field_name: field_name.to_string(),
            value: error.value().to_string(),
            error: error.source_error().to_string(),
        }
    })
}
