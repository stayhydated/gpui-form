use heck::{ToKebabCase as _, ToPascalCase as _, ToSnakeCase as _};
use syn::{Expr, Ident, Path, Type};

use crate::registry::{
    ComponentCapabilities, FieldValuePresence, FieldVariant, GpuiFormShape, RustSyntaxKind,
    ValidationRuleId,
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
    InvalidFieldPath {
        field_name: String,
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
            Self::InvalidFieldPath {
                field_name,
                kind,
                value,
                error,
            } => {
                write!(
                    f,
                    "invalid {kind} `{value}` for field `{field_name}` in gpui-form shape metadata: {error}"
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

#[derive(Clone)]
pub struct ResolvedComponentMetadata {
    shape_path: Path,
    capabilities: ComponentCapabilities,
}

impl ResolvedComponentMetadata {
    fn new(field: &FieldVariant) -> Result<Option<Self>, ResolveError> {
        let Some(component) = field.component_variant() else {
            return Ok(None);
        };
        Ok(Some(Self {
            shape_path: parse_field_path(
                field.field_name(),
                "component shape path",
                component.shape_path(),
            )?,
            capabilities: component.capabilities(),
        }))
    }

    pub fn shape_path(&self) -> &Path {
        &self.shape_path
    }

    pub fn runtime_shape_path(&self) -> Path {
        self.shape_path.clone()
    }

    pub fn capabilities(&self) -> ComponentCapabilities {
        self.capabilities
    }
}

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
pub struct ComponentHelperIdent(Ident);

impl ComponentHelperIdent {
    fn new(value: &str) -> Result<Self, ResolveError> {
        parse_ident("field component helper ident", value).map(Self)
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
    component_helper_ident: ComponentHelperIdent,
    kebab_id: KebabId,
    handler_ident: HandlerIdent,
}

impl FieldName {
    fn new(field: &FieldVariant) -> Result<Self, ResolveError> {
        let source = field.field_name();
        let ident = FieldIdent::new("field name", source)?;
        let pascal_ident = FieldIdent::new("field pascal ident", &source.to_pascal_case())?;
        let component_helper_name = field.field_name_with_component_suffix();
        let component_helper_ident = ComponentHelperIdent::new(&component_helper_name)?;
        let kebab_id = KebabId::new(component_helper_name.to_kebab_case());
        let handler_ident = HandlerIdent::new(&format!("on_{component_helper_name}_event"))?;

        Ok(Self {
            source,
            ident,
            pascal_ident,
            component_helper_ident,
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

    pub fn component_helper_ident(&self) -> &Ident {
        self.component_helper_ident.as_ident()
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
    value_presence: FieldValuePresence,
    default_expr: Option<Expr>,
    from_expr: Option<Expr>,
    into_expr: Option<Expr>,
    validations: &'static [ValidationRuleId],
    component: Option<ResolvedComponentMetadata>,
}

impl<'a> ResolvedField<'a> {
    pub fn new(field: &'a FieldVariant) -> Result<Self, ResolveError> {
        let name = FieldName::new(field)?;
        let value_type = parse_type(field.field_name(), field.value_type())?;
        let source_value_type = parse_type(field.field_name(), field.source_value_type())?;
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
        let component = ResolvedComponentMetadata::new(field)?;

        Ok(Self {
            raw: field,
            name,
            value_type,
            source_value_type,
            value_presence: field.value_presence(),
            default_expr,
            from_expr,
            into_expr,
            validations: field.validation_rules(),
            component,
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

    pub fn component_helper_ident(&self) -> &Ident {
        self.name.component_helper_ident()
    }

    pub fn value_type(&self) -> &Type {
        &self.value_type
    }

    pub fn source_value_type(&self) -> &Type {
        &self.source_value_type
    }

    pub fn component(&self) -> Option<&ResolvedComponentMetadata> {
        self.component.as_ref()
    }

    pub fn is_component(&self) -> bool {
        self.component.is_some()
    }

    pub fn shape_path(&self) -> Option<&Path> {
        self.component().map(ResolvedComponentMetadata::shape_path)
    }

    pub fn runtime_shape_path(&self) -> Option<Path> {
        self.component()
            .map(ResolvedComponentMetadata::runtime_shape_path)
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
        self.component()
            .map(ResolvedComponentMetadata::capabilities)
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
        self.value_presence.optional()
    }

    pub fn value_holder_uses_option(&self) -> bool {
        self.value_presence.value_holder_uses_option()
    }

    pub fn value_binding(&self) -> bool {
        self.component_capabilities()
            .is_some_and(ComponentCapabilities::value_binding_enabled)
    }

    pub fn subscribable(&self) -> bool {
        self.value_binding()
    }

    pub fn render_component(&self) -> bool {
        self.component_capabilities()
            .is_some_and(ComponentCapabilities::render_component)
    }

    pub fn validation_rules(&self) -> &'static [ValidationRuleId] {
        self.validations
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

fn parse_field_path(
    field_name: &str,
    kind: &'static str,
    value: crate::registry::RustPath,
) -> Result<Path, ResolveError> {
    value
        .parse()
        .map_err(|error| ResolveError::InvalidFieldPath {
            field_name: field_name.to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{
        FieldComponentVariant, FieldValuePresence, FieldValueSpec, FieldVariant, RustPath, RustType,
    };

    const fn value_spec(
        value_type: &'static str,
        value_presence: FieldValuePresence,
    ) -> FieldValueSpec {
        let value_type = RustType::from_macro_tokens_unchecked(value_type);
        FieldValueSpec::new(value_type, value_type, value_presence)
    }

    #[test]
    fn component_shape_path_errors_include_field_context() {
        const FIELD: FieldVariant = FieldVariant::component(
            "country",
            value_spec("String", FieldValuePresence::DirectStorage),
            FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked("crate::")),
        );

        let error = match ResolvedField::new(&FIELD) {
            Ok(_) => panic!("invalid shape path should fail"),
            Err(error) => error,
        };

        assert_eq!(
            error,
            ResolveError::InvalidFieldPath {
                field_name: "country".to_string(),
                kind: "component shape path",
                value: "crate::".to_string(),
                error: "unexpected end of input, expected identifier".to_string(),
            }
        );
    }

    #[test]
    fn resolved_field_exposes_component_and_storage_facts_without_raw_helpers() {
        const FIELD: FieldVariant = FieldVariant::component(
            "country",
            value_spec("String", FieldValuePresence::Optional),
            FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
                "crate::CountryShape",
            ))
            .with_value_binding(true),
        );

        let field = ResolvedField::new(&FIELD).expect("metadata should resolve");

        assert!(field.is_component());
        assert!(field.value_holder_uses_option());
        assert!(field.value_binding());
        assert_eq!(field.field_ident().to_string(), "country");
        assert_eq!(field.component_helper_ident().to_string(), "country_shape");
        let shape_path = field
            .component()
            .expect("component metadata should exist")
            .shape_path();
        assert_eq!(
            shape_path.segments.last().unwrap().ident.to_string(),
            "CountryShape"
        );
    }
}
