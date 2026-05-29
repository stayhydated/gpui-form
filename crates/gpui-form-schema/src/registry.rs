use heck::{ToKebabCase as _, ToPascalCase as _, ToSnakeCase as _};

inventory::collect!(GpuiFormShape);

/// Rust type syntax stored in inventory metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RustType(&'static str);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RustSyntaxKind {
    Type,
    Path,
    Expr,
}

impl RustSyntaxKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::Path => "path",
            Self::Expr => "expression",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RustSyntaxError {
    kind: RustSyntaxKind,
    value: String,
    error: String,
}

impl RustSyntaxError {
    pub const fn kind(&self) -> RustSyntaxKind {
        self.kind
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn source_error(&self) -> &str {
        &self.error
    }
}

impl std::fmt::Display for RustSyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid Rust {} metadata `{}`: {}",
            self.kind.label(),
            self.value,
            self.error
        )
    }
}

impl std::error::Error for RustSyntaxError {}

fn rust_syntax_error(kind: RustSyntaxKind, value: &str, error: syn::Error) -> RustSyntaxError {
    RustSyntaxError {
        kind,
        value: value.to_string(),
        error: error.to_string(),
    }
}

impl RustType {
    pub fn parse(self) -> Result<syn::Type, RustSyntaxError> {
        syn::parse_str::<syn::Type>(self.0)
            .map_err(|error| rust_syntax_error(RustSyntaxKind::Type, self.0, error))
    }

    pub fn new(value: &'static str) -> Result<Self, RustSyntaxError> {
        Self(value).parse().map(|_| Self(value))
    }

    pub fn new_opt(value: Option<&'static str>) -> Result<Option<Self>, RustSyntaxError> {
        match value {
            Some(value) => Self::new(value).map(Some),
            None => Ok(None),
        }
    }

    /// Construct metadata without validation.
    ///
    /// This is intended for macro-generated inventory, where the derive layer
    /// just stringified syntax that was already parsed by `syn`.
    #[doc(hidden)]
    pub const fn new_unchecked(value: &'static str) -> Self {
        Self(value)
    }

    #[doc(hidden)]
    pub const fn new_opt_unchecked(value: Option<&'static str>) -> Option<Self> {
        match value {
            Some(value) => Some(Self::new_unchecked(value)),
            None => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// Rust path syntax stored in inventory metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RustPath(&'static str);

impl RustPath {
    pub fn parse(self) -> Result<syn::Path, RustSyntaxError> {
        syn::parse_str::<syn::Path>(self.0)
            .map_err(|error| rust_syntax_error(RustSyntaxKind::Path, self.0, error))
    }

    pub fn new(value: &'static str) -> Result<Self, RustSyntaxError> {
        Self(value).parse().map(|_| Self(value))
    }

    /// Construct metadata without validation.
    ///
    /// This is intended for macro-generated inventory, where the derive layer
    /// just stringified syntax that was already parsed by `syn`.
    #[doc(hidden)]
    pub const fn new_unchecked(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// Rust expression syntax stored in inventory metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RustExpr(&'static str);

impl RustExpr {
    pub fn parse(self) -> Result<syn::Expr, RustSyntaxError> {
        syn::parse_str::<syn::Expr>(self.0)
            .map_err(|error| rust_syntax_error(RustSyntaxKind::Expr, self.0, error))
    }

    pub fn new(value: &'static str) -> Result<Self, RustSyntaxError> {
        Self(value).parse().map(|_| Self(value))
    }

    /// Construct metadata without validation.
    ///
    /// This is intended for macro-generated inventory, where the derive layer
    /// just stringified syntax that was already parsed by `syn`.
    #[doc(hidden)]
    pub const fn new_unchecked(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// Preferred component field/helper suffix stored in inventory metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentSuffix(&'static str);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentSuffixError {
    value: String,
}

impl ComponentSuffixError {
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for ComponentSuffixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "`field_suffix` must be a non-empty ASCII identifier suffix, got `{}`",
            self.value
        )
    }
}

impl std::error::Error for ComponentSuffixError {}

pub const fn is_valid_component_suffix(value: &str) -> bool {
    gpui_form_core::component_suffix::is_valid_ascii_identifier_suffix(value)
}

pub fn validate_component_suffix(value: &str) -> Result<(), ComponentSuffixError> {
    if is_valid_component_suffix(value) {
        Ok(())
    } else {
        Err(ComponentSuffixError {
            value: value.to_string(),
        })
    }
}

impl ComponentSuffix {
    pub const fn new(value: &'static str) -> Self {
        assert!(
            is_valid_component_suffix(value),
            "component suffix must be a non-empty ASCII identifier suffix"
        );
        Self(value)
    }

    pub const fn new_opt(value: Option<&'static str>) -> Option<Self> {
        match value {
            Some(value) => Some(Self::new(value)),
            None => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Debug)]
pub struct GpuiFormShape {
    pub struct_name: &'static str,
    pub components: &'static [FieldVariant],
    /// The source file path where the struct with #[derive(GpuiForm)] is declared.
    /// This is the full path from file!() macro, useful for generating imports.
    pub source_path: &'static str,
    /// Whether the struct has koruma validation enabled at the struct level.
    pub koruma_enabled: bool,
    /// Whether the original struct contains any `#[gpui_form(skip)]` fields.
    ///
    /// When true, generated `FormValueHolder` cannot be converted back into the
    /// original struct without additional skipped-field values.
    pub has_skipped_fields: bool,
    /// Whether the generated holder-to-model conversion API is fallible.
    ///
    /// This is emitted by `#[derive(GpuiForm)]` from the same analysis used for
    /// the generated value-holder methods, so downstream generators do not need
    /// to reconstruct conversion behavior from field-level metadata.
    pub holder_conversion_can_fail: bool,
}

impl GpuiFormShape {
    pub const fn new(
        struct_name: &'static str,
        components: &'static [FieldVariant],
        source_path: &'static str,
        koruma_enabled: bool,
    ) -> Self {
        Self {
            struct_name,
            components,
            source_path,
            koruma_enabled,
            has_skipped_fields: false,
            holder_conversion_can_fail: false,
        }
    }

    /// Marks whether the original struct has any `#[gpui_form(skip)]` fields.
    pub const fn with_skipped_fields(mut self, has_skipped_fields: bool) -> Self {
        self.has_skipped_fields = has_skipped_fields;
        self
    }

    /// Marks whether generated holder-to-model conversion uses the fallible
    /// `try_into_original()` shape.
    pub const fn with_holder_conversion_can_fail(
        mut self,
        holder_conversion_can_fail: bool,
    ) -> Self {
        self.holder_conversion_can_fail = holder_conversion_can_fail;
        self
    }

    pub fn has_validations(&self) -> bool {
        self.koruma_enabled
            && self
                .components
                .iter()
                .any(|field| !field.validations.is_empty())
    }

    /// Returns true if the struct has koruma validation enabled at the struct level.
    pub const fn has_koruma(&self) -> bool {
        self.koruma_enabled
    }

    /// Returns true when at least one source field is marked `#[gpui_form(skip)]`.
    pub const fn has_skipped_fields(&self) -> bool {
        self.has_skipped_fields
    }

    /// Returns true when generated holder-to-model conversion exposes
    /// `try_into_original()` for non-skipped fields.
    pub const fn holder_conversion_can_fail(&self) -> bool {
        self.holder_conversion_can_fail
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldValuePresence {
    /// Source field is `Option<T>` and generated storage is optional.
    Optional,
    /// Source field is non-optional and generated storage may be missing.
    RequiresValue,
    /// Source field is non-optional and generated storage is direct `T`.
    DirectStorage,
}

impl FieldValuePresence {
    pub const fn optional(self) -> bool {
        matches!(self, Self::Optional)
    }

    pub const fn requires_value(self) -> bool {
        matches!(self, Self::RequiresValue)
    }

    pub const fn value_holder_uses_option(self) -> bool {
        matches!(self, Self::Optional | Self::RequiresValue)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldComponentVariant {
    shape_path: RustPath,
    component_type: Option<RustType>,
    value_binding: bool,
    prototyping_field_suffix: Option<ComponentSuffix>,
}

impl FieldComponentVariant {
    pub const fn new(shape_path: RustPath) -> Self {
        Self {
            shape_path,
            component_type: None,
            value_binding: false,
            prototyping_field_suffix: None,
        }
    }

    /// Attach a UI component type to this component metadata.
    pub const fn with_component_type(mut self, component: RustType) -> Self {
        self.component_type = Some(component);
        self
    }

    /// Attach an optional UI component type to this component metadata.
    ///
    /// Used when the component type may come from the shape's
    /// `ComponentShape::COMPONENT_TYPE` constant rather than an explicit
    /// field attribute value.
    pub const fn with_component_type_opt(mut self, component: Option<RustType>) -> Self {
        self.component_type = component;
        self
    }

    /// Marks this component shape as value-bound for generated prototyping code.
    pub const fn with_value_binding(mut self, enabled: bool) -> Self {
        self.value_binding = enabled;
        self
    }

    /// Attach the component shape's preferred prototyping field suffix.
    pub const fn with_prototyping_field_suffix(mut self, suffix: Option<ComponentSuffix>) -> Self {
        self.prototyping_field_suffix = suffix;
        self
    }

    pub const fn shape_path(&self) -> RustPath {
        self.shape_path
    }

    pub const fn component_type(&self) -> Option<RustType> {
        self.component_type
    }

    pub const fn value_binding(&self) -> bool {
        self.value_binding
    }

    pub const fn prototyping_field_suffix(&self) -> Option<ComponentSuffix> {
        self.prototyping_field_suffix
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldVariant {
    field_name: &'static str,
    /// Rust type path for the field's value type.
    ///
    /// This is the form-side base value type, not including any generated
    /// `Option<...>` wrapper. It is a full Rust type string (for example
    /// `Country` or `some_lib::country::Country`), not just a bare identifier.
    value_type: RustType,
    /// Rust type path for the source model's base value type before any
    /// `#[gpui_form(type = ...)]` override is applied.
    source_value_type: RustType,
    value_presence: FieldValuePresence,
    /// List of validation rule identifiers applied to this field (for diagnostics/rendering).
    validations: &'static [&'static str],
    /// Default value expression as a string, if one was specified.
    default_expr: Option<RustExpr>,
    /// Source-to-form conversion expression, if one was specified.
    from_expr: Option<RustExpr>,
    /// Form-to-source conversion expression, if one was specified.
    into_expr: Option<RustExpr>,
    component: Option<FieldComponentVariant>,
}

impl FieldVariant {
    pub const fn hidden(
        field_name: &'static str,
        value_type: RustType,
        value_presence: FieldValuePresence,
    ) -> Self {
        Self {
            field_name,
            value_type,
            source_value_type: value_type,
            value_presence,
            validations: &[],
            default_expr: None,
            from_expr: None,
            into_expr: None,
            component: None,
        }
    }

    pub const fn component(
        field_name: &'static str,
        value_type: RustType,
        value_presence: FieldValuePresence,
        component: FieldComponentVariant,
    ) -> Self {
        Self {
            field_name,
            value_type,
            source_value_type: value_type,
            value_presence,
            validations: &[],
            default_expr: None,
            from_expr: None,
            into_expr: None,
            component: Some(component),
        }
    }

    pub const fn field_name(&self) -> &'static str {
        self.field_name
    }

    pub const fn value_type(&self) -> RustType {
        self.value_type
    }

    pub const fn source_value_type(&self) -> RustType {
        self.source_value_type
    }

    pub const fn optional(&self) -> bool {
        self.value_presence.optional()
    }

    pub const fn requires_value(&self) -> bool {
        self.value_presence.requires_value()
    }

    pub const fn default_expr(&self) -> Option<RustExpr> {
        self.default_expr
    }

    pub const fn from_expr(&self) -> Option<RustExpr> {
        self.from_expr
    }

    pub const fn into_expr(&self) -> Option<RustExpr> {
        self.into_expr
    }

    pub const fn component_variant(&self) -> Option<&FieldComponentVariant> {
        self.component.as_ref()
    }

    pub const fn is_component(&self) -> bool {
        self.component.is_some()
    }

    pub const fn shape_path(&self) -> Option<RustPath> {
        match self.component {
            Some(component) => Some(component.shape_path),
            None => None,
        }
    }

    pub const fn component_type(&self) -> Option<RustType> {
        match self.component {
            Some(component) => component.component_type,
            None => None,
        }
    }

    pub const fn value_binding(&self) -> bool {
        match self.component {
            Some(component) => component.value_binding,
            None => false,
        }
    }

    pub const fn prototyping_field_suffix(&self) -> Option<ComponentSuffix> {
        match self.component {
            Some(component) => component.prototyping_field_suffix,
            None => None,
        }
    }

    /// Attach the source model value type when it differs from the form-side value type.
    pub const fn with_source_value_type(mut self, source_value_type: RustType) -> Self {
        self.source_value_type = source_value_type;
        self
    }

    /// Attach optional source/form conversion expressions.
    pub const fn with_conversions(
        mut self,
        from_expr: Option<RustExpr>,
        into_expr: Option<RustExpr>,
    ) -> Self {
        self.from_expr = from_expr;
        self.into_expr = into_expr;
        self
    }

    /// Attach a default value expression to this field metadata.
    pub const fn with_default(mut self, default_expr: RustExpr) -> Self {
        self.default_expr = Some(default_expr);
        self
    }

    /// Returns true when the generated value holder stores this field as `Option<T>`.
    pub const fn value_holder_uses_option(&self) -> bool {
        self.value_presence.value_holder_uses_option()
    }

    /// Returns true when generated code should subscribe to this field.
    pub const fn subscribable(&self) -> bool {
        self.value_binding()
    }

    pub fn component_suffix(&self) -> String {
        self.prototyping_field_suffix()
            .and_then(|suffix| component_suffix_from_suffix(self.field_name, suffix.as_str()))
            .filter(|suffix| !suffix.is_empty())
            .unwrap_or_else(|| "shape".to_string())
    }

    pub fn field_name_pascal(&self) -> String {
        self.field_name.to_pascal_case()
    }

    pub fn field_name_with_component_suffix(&self) -> String {
        format!("{}_{}", self.field_name, self.component_suffix())
    }

    pub fn kebab_id(&self) -> String {
        self.field_name_with_component_suffix().to_kebab_case()
    }

    /// Returns the validation rule identifiers attached to this field.
    pub fn validation_rules(&self) -> &'static [&'static str] {
        self.validations
    }

    /// Attach validation rule identifiers to this field metadata.
    pub const fn with_validations(mut self, validations: &'static [&'static str]) -> Self {
        self.validations = validations;
        self
    }
}

pub fn component_suffix_from_suffix(field_name: &str, suffix: &str) -> Option<String> {
    let mut suffix = suffix.to_snake_case();
    let field_name = field_name.to_snake_case();

    if suffix == field_name {
        return None;
    }

    let field_prefix = format!("{field_name}_");
    if let Some(rest) = suffix.strip_prefix(&field_prefix) {
        suffix = rest.to_string();
    }

    (!suffix.is_empty()).then_some(suffix)
}

#[cfg(test)]
mod tests {
    use super::{
        ComponentSuffix, FieldComponentVariant, FieldValuePresence, FieldVariant, RustExpr,
        RustPath, RustSyntaxKind, RustType, is_valid_component_suffix, validate_component_suffix,
    };

    const fn shape_field(
        field_name: &'static str,
        value_type: &'static str,
        shape_path: &'static str,
    ) -> FieldVariant {
        FieldVariant::component(
            field_name,
            RustType::new_unchecked(value_type),
            FieldValuePresence::RequiresValue,
            FieldComponentVariant::new(RustPath::new_unchecked(shape_path)),
        )
    }

    #[test]
    fn rust_metadata_constructors_validate_syntax() {
        assert_eq!(
            RustType::new("Vec<String>").unwrap().as_str(),
            "Vec<String>"
        );
        assert_eq!(
            RustPath::new("crate::fields::EmailInputShape")
                .unwrap()
                .as_str(),
            "crate::fields::EmailInputShape"
        );
        assert_eq!(
            RustExpr::new("|value| value.to_string()").unwrap().as_str(),
            "|value| value.to_string()"
        );

        let type_error = RustType::new("Vec<").unwrap_err();
        assert_eq!(type_error.kind(), RustSyntaxKind::Type);
        assert_eq!(type_error.value(), "Vec<");

        let path_error = RustPath::new("crate::").unwrap_err();
        assert_eq!(path_error.kind(), RustSyntaxKind::Path);
        assert_eq!(path_error.value(), "crate::");

        let expr_error = RustExpr::new("let").unwrap_err();
        assert_eq!(expr_error.kind(), RustSyntaxKind::Expr);
        assert_eq!(expr_error.value(), "let");
    }

    #[test]
    fn shape_name_without_prototyping_suffix_falls_back_to_shape() {
        let field = shape_field("country", "Country", "crate::fields::CountrySelectShape");

        assert_eq!(field.field_name_with_component_suffix(), "country_shape");
        assert_eq!(field.kebab_id(), "country-shape");
    }

    #[test]
    fn prototyping_suffix_drives_component_suffix() {
        let field = FieldVariant::component(
            "country",
            RustType::new_unchecked("Country"),
            FieldValuePresence::RequiresValue,
            FieldComponentVariant::new(RustPath::new_unchecked(
                "crate::fields::CountrySelectorState",
            ))
            .with_prototyping_field_suffix(Some(ComponentSuffix::new("select"))),
        );

        assert_eq!(field.field_name_with_component_suffix(), "country_select");
    }

    #[test]
    fn prototyping_suffix_removes_duplicate_field_prefix() {
        let field = FieldVariant::component(
            "email",
            RustType::new_unchecked("String"),
            FieldValuePresence::RequiresValue,
            FieldComponentVariant::new(RustPath::new_unchecked("crate::fields::TextInputShape"))
                .with_prototyping_field_suffix(Some(ComponentSuffix::new("email_input"))),
        );

        assert_eq!(field.field_name_with_component_suffix(), "email_input");
    }

    #[test]
    fn prototyping_suffix_exact_duplicate_uses_shape_fallback() {
        let field = FieldVariant::component(
            "tags",
            RustType::new_unchecked("Vec<String>"),
            FieldValuePresence::RequiresValue,
            FieldComponentVariant::new(RustPath::new_unchecked("crate::fields::TagsInputShape"))
                .with_prototyping_field_suffix(Some(ComponentSuffix::new("tags"))),
        );

        assert_eq!(field.field_name_with_component_suffix(), "tags_shape");
    }

    #[test]
    fn optional_presence_uses_value_holder_option() {
        let field = FieldVariant::hidden(
            "email",
            RustType::new_unchecked("String"),
            FieldValuePresence::Optional,
        );

        assert!(field.optional());
        assert!(!field.requires_value());
        assert!(field.value_holder_uses_option());
    }

    #[test]
    fn component_metadata_is_constructed_before_field_variant() {
        let component =
            FieldComponentVariant::new(RustPath::new_unchecked("crate::fields::EmailInputShape"))
                .with_component_type(RustType::new_unchecked("EmailInput"))
                .with_value_binding(true)
                .with_prototyping_field_suffix(Some(ComponentSuffix::new("input")));
        let field = FieldVariant::component(
            "email",
            RustType::new_unchecked("String"),
            FieldValuePresence::DirectStorage,
            component,
        );

        assert_eq!(
            field.shape_path().map(RustPath::as_str),
            Some("crate::fields::EmailInputShape")
        );
        assert_eq!(
            field.component_type().map(RustType::as_str),
            Some("EmailInput")
        );
        assert!(field.value_binding());
        assert!(!field.value_holder_uses_option());
    }

    #[test]
    fn shape_path_does_not_drive_field_suffix() {
        let field = shape_field("email", "String", "crate::fields::EmailInputShape");

        assert_eq!(field.field_name_with_component_suffix(), "email_shape");
    }

    #[test]
    fn multiword_shape_name_without_prototyping_suffix_uses_shape() {
        let field = shape_field(
            "location",
            "Country",
            "gpui_form_component::infinite_select::InfiniteSelect<Country>",
        );

        assert_eq!(field.field_name_with_component_suffix(), "location_shape");
    }

    #[test]
    fn exact_duplicate_shape_name_falls_back_to_shape_suffix() {
        let field = shape_field("tags", "Vec<String>", "crate::state::TagsState");

        assert_eq!(field.field_name_with_component_suffix(), "tags_shape");
    }

    #[test]
    fn component_suffix_validation_accepts_identifier_suffixes() {
        assert!(is_valid_component_suffix("input"));
        assert!(is_valid_component_suffix("number_input"));
        assert!(is_valid_component_suffix("_internal"));
    }

    #[test]
    fn component_suffix_validation_rejects_invalid_suffixes() {
        for value in ["", "_", "123", "field suffix", "field-suffix"] {
            assert!(
                validate_component_suffix(value).is_err(),
                "{value:?} should be rejected"
            );
        }
    }
}

pub use inventory;
