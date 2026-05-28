use heck::{ToKebabCase as _, ToPascalCase as _, ToSnakeCase as _};

inventory::collect!(GpuiFormShape);

/// Rust type syntax stored in inventory metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RustType(&'static str);

impl RustType {
    pub const fn new(value: &'static str) -> Self {
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

/// Rust path syntax stored in inventory metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RustPath(&'static str);

impl RustPath {
    pub const fn new(value: &'static str) -> Self {
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
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// Preferred component field/helper suffix stored in inventory metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentSuffix(&'static str);

impl ComponentSuffix {
    pub const fn new(value: &'static str) -> Self {
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
        }
    }

    /// Marks whether the original struct has any `#[gpui_form(skip)]` fields.
    pub const fn with_skipped_fields(mut self, has_skipped_fields: bool) -> Self {
        self.has_skipped_fields = has_skipped_fields;
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
}

#[derive(Debug)]
pub struct FieldVariant {
    pub field_name: &'static str,
    /// Rust type path for the field's value type.
    ///
    /// This is the form-side base value type, not including any generated
    /// `Option<...>` wrapper. It is a full Rust type string (for example
    /// `Country` or `some_lib::country::Country`), not just a bare identifier.
    pub value_type: RustType,
    /// Rust type path for the source model's base value type before any
    /// `#[gpui_form(type = ...)]` override is applied.
    pub source_value_type: RustType,
    pub optional: bool,
    /// Whether a missing generated value-holder value is invalid for this field.
    /// Source `Option<T>` fields are tracked by [`FieldVariant::optional`] and
    /// do not require a value.
    pub requires_value: bool,
    /// List of validation rule identifiers applied to this field (for diagnostics/rendering).
    pub validations: &'static [&'static str],
    /// Default value expression as a string, if one was specified.
    pub default_expr: Option<RustExpr>,
    /// Source-to-form conversion expression, if one was specified.
    pub from_expr: Option<RustExpr>,
    /// Form-to-source conversion expression, if one was specified.
    pub into_expr: Option<RustExpr>,
    /// Shape type path implementing
    /// `gpui_form_runtime::shape::ComponentShape`.
    pub shape_path: Option<RustPath>,
    /// UI component type (e.g. "TagsInput" or "Combobox<_>").
    /// Used by the prototyping code generator to emit `<Component>::new(&entity)`.
    pub component_type: Option<RustType>,
    /// Whether the component shape opted into
    /// `gpui_form_runtime::shape::ComponentValueBinding` generation.
    pub value_binding: bool,
    /// Preferred generated field/helper suffix supplied by the shape's
    /// prototyping metadata.
    pub prototyping_field_suffix: Option<ComponentSuffix>,
}

impl FieldVariant {
    pub const fn new(field_name: &'static str, value_type: RustType, optional: bool) -> Self {
        Self {
            field_name,
            value_type,
            source_value_type: value_type,
            optional,
            requires_value: !optional,
            validations: &[],
            default_expr: None,
            from_expr: None,
            into_expr: None,
            shape_path: None,
            component_type: None,
            value_binding: false,
            prototyping_field_suffix: None,
        }
    }

    /// Attach the source model value type when it differs from the form-side value type.
    pub const fn with_source_value_type(mut self, source_value_type: RustType) -> Self {
        self.source_value_type = source_value_type;
        self
    }

    /// Attach the required-value policy for generated value-holder conversion.
    pub const fn with_requires_value(mut self, requires_value: bool) -> Self {
        self.requires_value = requires_value && !self.optional;
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

    /// Attach a UI component type to this field metadata.
    pub const fn with_component_type(mut self, component: RustType) -> Self {
        self.component_type = Some(component);
        self
    }

    /// Attach an optional UI component type to this field metadata.
    ///
    /// Used when the component type may come from the shape's
    /// `ComponentShape::COMPONENT_TYPE` constant rather than an explicit
    /// field attribute value.
    pub const fn with_component_type_opt(mut self, component: Option<RustType>) -> Self {
        self.component_type = component;
        self
    }

    /// Attach the component shape type path.
    pub const fn with_shape_path(mut self, shape: RustPath) -> Self {
        self.shape_path = Some(shape);
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

    /// Returns true when the generated value holder stores this field as `Option<T>`.
    pub const fn value_holder_uses_option(&self) -> bool {
        self.optional || self.requires_value
    }

    /// Returns true when generated code should subscribe to this field.
    pub const fn subscribable(&self) -> bool {
        self.value_binding
    }

    pub fn component_suffix(&self) -> String {
        self.prototyping_field_suffix
            .and_then(|suffix| component_suffix_from_suffix(self.field_name, suffix.as_str()))
            .filter(|suffix| !suffix.is_empty())
            .unwrap_or_else(|| "shape".to_string())
    }

    pub fn field_name_pascal(&self) -> String {
        self.field_name.to_pascal_case()
    }

    pub fn field_name_with_behaviour(&self) -> String {
        format!("{}_{}", self.field_name, self.component_suffix())
    }

    pub fn kebab_id(&self) -> String {
        self.field_name_with_behaviour().to_kebab_case()
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
    use super::{ComponentSuffix, FieldVariant, RustPath, RustType};

    #[test]
    fn shape_name_without_prototyping_suffix_falls_back_to_shape() {
        let field = FieldVariant::new("country", RustType::new("Country"), false)
            .with_shape_path(RustPath::new("crate::fields::CountrySelectShape"));

        assert_eq!(field.field_name_with_behaviour(), "country_shape");
        assert_eq!(field.kebab_id(), "country-shape");
    }

    #[test]
    fn prototyping_suffix_drives_component_suffix() {
        let field = FieldVariant::new("country", RustType::new("Country"), false)
            .with_shape_path(RustPath::new("crate::fields::CountrySelectorState"))
            .with_prototyping_field_suffix(Some(ComponentSuffix::new("select")));

        assert_eq!(field.field_name_with_behaviour(), "country_select");
    }

    #[test]
    fn prototyping_suffix_removes_duplicate_field_prefix() {
        let field = FieldVariant::new("email", RustType::new("String"), false)
            .with_shape_path(RustPath::new("crate::fields::TextInputShape"))
            .with_prototyping_field_suffix(Some(ComponentSuffix::new("email_input")));

        assert_eq!(field.field_name_with_behaviour(), "email_input");
    }

    #[test]
    fn prototyping_suffix_exact_duplicate_uses_shape_fallback() {
        let field = FieldVariant::new("tags", RustType::new("Vec<String>"), false)
            .with_shape_path(RustPath::new("crate::fields::TagsInputShape"))
            .with_prototyping_field_suffix(Some(ComponentSuffix::new("tags")));

        assert_eq!(field.field_name_with_behaviour(), "tags_shape");
    }

    #[test]
    fn shape_path_does_not_drive_field_suffix() {
        let field = FieldVariant::new("email", RustType::new("String"), false)
            .with_shape_path(RustPath::new("crate::fields::EmailInputShape"));

        assert_eq!(field.field_name_with_behaviour(), "email_shape");
    }

    #[test]
    fn multiword_shape_name_without_prototyping_suffix_uses_shape() {
        let field = FieldVariant::new("location", RustType::new("Country"), false).with_shape_path(
            RustPath::new("gpui_form_component::infinite_select::InfiniteSelect<Country>"),
        );

        assert_eq!(field.field_name_with_behaviour(), "location_shape");
    }

    #[test]
    fn exact_duplicate_shape_name_falls_back_to_shape_suffix() {
        let field = FieldVariant::new("tags", RustType::new("Vec<String>"), false)
            .with_shape_path(RustPath::new("crate::state::TagsState"));

        assert_eq!(field.field_name_with_behaviour(), "tags_shape");
    }
}

pub use inventory;
