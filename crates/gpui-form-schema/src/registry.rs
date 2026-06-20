use component_shape::{
    ComponentCapabilities as ShapeComponentCapabilities,
    ComponentPrototyping as ShapeComponentPrototyping, ComponentShapeUse,
};
use heck::{ToKebabCase as _, ToPascalCase as _};
use strum::IntoStaticStr;

pub use component_shape::{
    ComponentFieldName, RenderCapability, RustExpr, RustPath, RustSyntaxError, RustSyntaxKind,
    RustType, ValueBindingCapability,
};
pub use gpui_form_core::component_suffix::{
    ComponentSuffix, ComponentSuffixError, is_valid_component_suffix, validate_component_suffix,
};

inventory::collect!(GpuiFormShape);
pub use inventory;

#[derive(Debug)]
pub struct GpuiFormShape {
    pub struct_name: &'static str,
    pub fields: &'static [FieldVariant],
    /// Rust module path where the struct with #[derive(GpuiForm)] is declared.
    pub source_module_path: RustPath,
    /// Whether the struct has koruma validation enabled at the struct level.
    pub koruma_enabled: bool,
    /// Holder-to-model conversion shape emitted by `#[derive(GpuiForm)]`.
    ///
    /// This is the same analysis used for generated value-holder methods, so
    /// downstream generators do not need to reconstruct conversion behavior
    /// from field-level metadata.
    pub holder_conversion: HolderConversionMetadata,
}

impl GpuiFormShape {
    pub const fn new(
        struct_name: &'static str,
        fields: &'static [FieldVariant],
        source_module_path: RustPath,
        koruma_enabled: bool,
    ) -> Self {
        Self {
            struct_name,
            fields,
            source_module_path,
            koruma_enabled,
            holder_conversion: HolderConversionMetadata::infallible(),
        }
    }

    /// Attach generated holder-to-model conversion metadata.
    pub const fn with_holder_conversion(
        mut self,
        holder_conversion: HolderConversionMetadata,
    ) -> Self {
        self.holder_conversion = holder_conversion;
        self
    }

    pub fn has_validations(&self) -> bool {
        self.koruma_enabled
            && self
                .fields
                .iter()
                .any(|field| !field.validations.is_empty())
    }

    /// Returns true if the struct has koruma validation enabled at the struct level.
    pub const fn has_koruma(&self) -> bool {
        self.koruma_enabled
    }

    /// Returns true when at least one source field is marked `#[gpui_form(skip)]`.
    pub const fn has_skipped_fields(&self) -> bool {
        self.holder_conversion.shape.needs_skipped_fields()
    }

    pub const fn holder_conversion(&self) -> HolderConversionMetadata {
        self.holder_conversion
    }

    pub const fn holder_conversion_shape(&self) -> HolderConversionShape {
        self.holder_conversion.shape
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HolderConversionShape {
    Infallible,
    FallibleRequired,
    NeedsSkippedFields,
}

impl HolderConversionShape {
    pub const fn uses_fallible_api(self) -> bool {
        matches!(self, Self::FallibleRequired | Self::NeedsSkippedFields)
    }

    pub const fn needs_skipped_fields(self) -> bool {
        matches!(self, Self::NeedsSkippedFields)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HolderConversionMetadata {
    shape: HolderConversionShape,
    runtime_can_fail: bool,
}

impl HolderConversionMetadata {
    pub const fn new(shape: HolderConversionShape, runtime_can_fail: bool) -> Self {
        Self {
            shape,
            runtime_can_fail,
        }
    }

    pub const fn infallible() -> Self {
        Self::new(HolderConversionShape::Infallible, false)
    }

    pub const fn shape(self) -> HolderConversionShape {
        self.shape
    }

    pub const fn runtime_can_fail(self) -> bool {
        self.runtime_can_fail
    }

    pub const fn uses_fallible_api(self) -> bool {
        self.shape.uses_fallible_api()
    }
}

#[derive(Clone, Copy, Debug, Eq, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "snake_case", const_into_str)]
pub enum FieldValuePresence {
    /// Source field is `Option<T>` and generated storage is optional.
    Optional,
    /// Source field is non-optional and generated storage may be missing.
    RequiresValue,
    /// Source field is non-optional and generated storage is direct `T`.
    DirectStorage,
}

impl FieldValuePresence {
    pub const fn as_str(self) -> &'static str {
        self.into_str()
    }

    pub const fn optional(self) -> bool {
        matches!(self, Self::Optional)
    }

    pub const fn requires_value(self) -> bool {
        matches!(self, Self::RequiresValue)
    }

    pub const fn value_holder_uses_option(self) -> bool {
        matches!(self, Self::Optional | Self::RequiresValue)
    }

    pub const fn component_storage_capability(self) -> StorageCapability {
        match self {
            Self::Optional => StorageCapability::OptionalValue,
            Self::RequiresValue => StorageCapability::RequiredValue,
            Self::DirectStorage => StorageCapability::DirectValue,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationRuleId {
    Required,
    Newtype,
    Nested,
    Custom(&'static str),
}

impl ValidationRuleId {
    pub const fn custom(name: &'static str) -> Self {
        Self::Custom(name)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Required => "RequiredValidation",
            Self::Newtype => "NewtypeValidation",
            Self::Nested => "NestedValidation",
            Self::Custom(name) => name,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageCapability {
    OptionalValue,
    RequiredValue,
    DirectValue,
    ShapePolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentCapabilities {
    shape: ShapeComponentCapabilities,
    storage: StorageCapability,
}

impl ComponentCapabilities {
    pub const fn new() -> Self {
        Self {
            shape: ShapeComponentCapabilities::new(),
            storage: StorageCapability::ShapePolicy,
        }
    }

    pub const fn with_render(mut self, render: RenderCapability) -> Self {
        self.shape = self.shape.with_render(render);
        self
    }

    pub const fn with_value_binding(mut self, value_binding: ValueBindingCapability) -> Self {
        self.shape = self.shape.with_value_binding(value_binding);
        self
    }

    pub const fn with_storage(mut self, storage: StorageCapability) -> Self {
        self.storage = storage;
        self
    }

    pub const fn render(self) -> RenderCapability {
        self.shape.render()
    }

    pub const fn value_binding(self) -> ValueBindingCapability {
        self.shape.value_binding()
    }

    pub const fn storage(self) -> StorageCapability {
        self.storage
    }

    const fn shape(self) -> ShapeComponentCapabilities {
        self.shape
    }

    pub const fn render_component(self) -> bool {
        self.shape.render_component()
    }

    pub const fn value_binding_enabled(self) -> bool {
        self.shape.value_binding_enabled()
    }
}

impl Default for ComponentCapabilities {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldComponentVariant {
    shape_use: ComponentShapeUse,
    storage: StorageCapability,
}

impl FieldComponentVariant {
    pub const fn new(shape_path: RustPath) -> Self {
        Self {
            shape_use: ComponentShapeUse::new(ComponentFieldName::new(""), shape_path),
            storage: StorageCapability::ShapePolicy,
        }
    }

    /// Attach the complete component capabilities metadata for this field.
    pub const fn with_capabilities(mut self, capabilities: ComponentCapabilities) -> Self {
        self.shape_use = self.shape_use.with_capabilities(capabilities.shape());
        self.storage = capabilities.storage();
        self
    }

    /// Mark that this component shape publishes render metadata for generated
    /// prototyping code.
    pub const fn with_render_component(mut self, enabled: bool) -> Self {
        let capabilities = self.shape_use.capabilities().with_render(if enabled {
            RenderCapability::Component
        } else {
            RenderCapability::None
        });
        self.shape_use = self.shape_use.with_capabilities(capabilities);
        self
    }

    /// Marks this component shape as value-bound for generated prototyping code.
    pub const fn with_value_binding(mut self, enabled: bool) -> Self {
        let capabilities = self
            .shape_use
            .capabilities()
            .with_value_binding(if enabled {
                ValueBindingCapability::Inherited
            } else {
                ValueBindingCapability::None
            });
        self.shape_use = self.shape_use.with_capabilities(capabilities);
        self
    }

    /// Attach the value-holder storage capability inferred for this field.
    pub const fn with_storage_capability(mut self, storage: StorageCapability) -> Self {
        self.storage = storage;
        self
    }

    /// Attach the component shape's preferred prototyping field suffix.
    pub const fn with_prototyping_field_suffix(mut self, suffix: Option<ComponentSuffix>) -> Self {
        self.shape_use = self.shape_use.with_prototyping(ShapeComponentPrototyping {
            field_suffix: suffix,
        });
        self
    }

    const fn with_field_metadata(
        mut self,
        field_name: ComponentFieldName<'static>,
        field_type: RustType,
    ) -> Self {
        self.shape_use = ComponentShapeUse::new(field_name, self.shape_use.shape_path())
            .with_field_type(field_type)
            .with_capabilities(self.shape_use.capabilities())
            .with_prototyping(self.shape_use.prototyping());
        self
    }

    /// Returns the serialized component shape path.
    ///
    /// Tooling that needs a parsed Rust path should prefer
    /// `gpui_form_schema::resolved::ResolvedComponentMetadata::shape_path`.
    pub const fn shape_path(&self) -> RustPath {
        self.shape_use.shape_path()
    }

    pub const fn capabilities(&self) -> ComponentCapabilities {
        ComponentCapabilities::new()
            .with_render(self.shape_use.capabilities().render())
            .with_value_binding(self.shape_use.capabilities().value_binding())
            .with_storage(self.storage)
    }

    pub const fn render_component(&self) -> bool {
        self.capabilities().render_component()
    }

    pub const fn value_binding(&self) -> bool {
        self.capabilities().value_binding_enabled()
    }

    pub const fn prototyping_field_suffix(&self) -> Option<ComponentSuffix> {
        self.shape_use.prototyping().field_suffix
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConversionMetadata {
    from_expr: Option<RustExpr>,
    into_expr: Option<RustExpr>,
}

impl ConversionMetadata {
    pub const fn identity() -> Self {
        Self {
            from_expr: None,
            into_expr: None,
        }
    }

    pub const fn new(from_expr: Option<RustExpr>, into_expr: Option<RustExpr>) -> Self {
        Self {
            from_expr,
            into_expr,
        }
    }

    pub const fn from_expr(self) -> Option<RustExpr> {
        self.from_expr
    }

    pub const fn into_expr(self) -> Option<RustExpr> {
        self.into_expr
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldValueSpec {
    value_type: RustType,
    source_value_type: RustType,
    value_presence: FieldValuePresence,
    conversions: ConversionMetadata,
    default_expr: Option<RustExpr>,
    validations: &'static [ValidationRuleId],
}

impl FieldValueSpec {
    pub const fn new(
        value_type: RustType,
        source_value_type: RustType,
        value_presence: FieldValuePresence,
    ) -> Self {
        Self {
            value_type,
            source_value_type,
            value_presence,
            conversions: ConversionMetadata::identity(),
            default_expr: None,
            validations: &[],
        }
    }

    pub const fn with_conversions(mut self, conversions: ConversionMetadata) -> Self {
        self.conversions = conversions;
        self
    }

    pub const fn with_default(mut self, default_expr: RustExpr) -> Self {
        self.default_expr = Some(default_expr);
        self
    }

    pub const fn with_validations(mut self, validations: &'static [ValidationRuleId]) -> Self {
        self.validations = validations;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldVariant {
    field_name: ComponentFieldName<'static>,
    label: Option<&'static str>,
    description: Option<&'static str>,
    examples: &'static [&'static str],
    /// Rust type path for the field's value type.
    ///
    /// This is the form-side base value type, not including any generated
    /// `Option<...>` wrapper. It is a full Rust type string (for example
    /// `Country` or `some_lib::country::Country`), not just a bare identifier.
    value_type: RustType,
    /// Rust type path for the source model's base value type before any
    /// intent-scoped `value(type = ...)` override is applied.
    source_value_type: RustType,
    value_presence: FieldValuePresence,
    /// List of validation rule identifiers applied to this field (for diagnostics/rendering).
    validations: &'static [ValidationRuleId],
    /// Default value expression as a string, if one was specified.
    default_expr: Option<RustExpr>,
    /// Source-to-form conversion expression, if one was specified.
    from_expr: Option<RustExpr>,
    /// Form-to-source conversion expression, if one was specified.
    into_expr: Option<RustExpr>,
    component: Option<FieldComponentVariant>,
}

impl FieldVariant {
    pub const fn hidden(field_name: ComponentFieldName<'static>, value: FieldValueSpec) -> Self {
        Self::from_parts(field_name, value, None)
    }

    pub const fn hidden_for_field(field_name: &'static str, value: FieldValueSpec) -> Self {
        Self::hidden(ComponentFieldName::new(field_name), value)
    }

    pub const fn component(
        field_name: ComponentFieldName<'static>,
        value: FieldValueSpec,
        component: FieldComponentVariant,
    ) -> Self {
        let component = component
            .with_field_metadata(field_name, value.source_value_type)
            .with_storage_capability(value.value_presence.component_storage_capability());
        Self::from_parts(field_name, value, Some(component))
    }

    pub const fn component_for_field(
        field_name: &'static str,
        value: FieldValueSpec,
        component: FieldComponentVariant,
    ) -> Self {
        Self::component(ComponentFieldName::new(field_name), value, component)
    }

    const fn from_parts(
        field_name: ComponentFieldName<'static>,
        value: FieldValueSpec,
        component: Option<FieldComponentVariant>,
    ) -> Self {
        Self {
            field_name,
            label: None,
            description: None,
            examples: &[],
            value_type: value.value_type,
            source_value_type: value.source_value_type,
            value_presence: value.value_presence,
            validations: value.validations,
            default_expr: value.default_expr,
            from_expr: value.conversions.from_expr,
            into_expr: value.conversions.into_expr,
            component,
        }
    }

    pub const fn field_name(&self) -> ComponentFieldName<'static> {
        self.field_name
    }

    pub const fn with_label(mut self, label: &'static str) -> Self {
        self.label = Some(label);
        self
    }

    pub const fn with_description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    pub const fn with_examples(mut self, examples: &'static [&'static str]) -> Self {
        self.examples = examples;
        self
    }

    pub const fn explicit_label(&self) -> Option<&'static str> {
        self.label
    }

    pub const fn description(&self) -> Option<&'static str> {
        self.description
    }

    pub const fn examples(&self) -> &'static [&'static str] {
        self.examples
    }

    /// Returns the serialized form-side value type.
    ///
    /// Tooling that needs a parsed Rust type should prefer
    /// `gpui_form_schema::resolved::ResolvedField::value_type`.
    pub const fn value_type(&self) -> RustType {
        self.value_type
    }

    /// Returns the serialized source-model value type.
    ///
    /// Tooling that needs a parsed Rust type should prefer
    /// `gpui_form_schema::resolved::ResolvedField::source_value_type`.
    pub const fn source_value_type(&self) -> RustType {
        self.source_value_type
    }

    pub const fn value_presence(&self) -> FieldValuePresence {
        self.value_presence
    }

    pub const fn optional(&self) -> bool {
        self.value_presence.optional()
    }

    pub const fn requires_value(&self) -> bool {
        self.value_presence.requires_value()
    }

    /// Returns the serialized default expression, if present.
    ///
    /// Tooling that needs parsed expression metadata should prefer
    /// `gpui_form_schema::resolved::ResolvedField::default_expr`.
    pub const fn default_expr(&self) -> Option<RustExpr> {
        self.default_expr
    }

    /// Returns the serialized source-to-form conversion expression, if present.
    ///
    /// Tooling that needs parsed expression metadata should prefer
    /// `gpui_form_schema::resolved::ResolvedField::from_expr`.
    pub const fn from_expr(&self) -> Option<RustExpr> {
        self.from_expr
    }

    /// Returns the serialized form-to-source conversion expression, if present.
    ///
    /// Tooling that needs parsed expression metadata should prefer
    /// `gpui_form_schema::resolved::ResolvedField::into_expr`.
    pub const fn into_expr(&self) -> Option<RustExpr> {
        self.into_expr
    }

    /// Returns the raw component inventory payload, if this field is component-backed.
    ///
    /// Tooling that needs parsed component metadata should prefer
    /// `gpui_form_schema::resolved::ResolvedField::component`.
    pub const fn component_variant(&self) -> Option<&FieldComponentVariant> {
        self.component.as_ref()
    }

    pub const fn is_component(&self) -> bool {
        self.component.is_some()
    }

    /// Returns the serialized component shape path, if this field is component-backed.
    ///
    /// Tooling that needs a parsed Rust path should prefer
    /// `gpui_form_schema::resolved::ResolvedField::shape_path`.
    pub const fn shape_path(&self) -> Option<RustPath> {
        match self.component {
            Some(component) => Some(component.shape_path()),
            None => None,
        }
    }

    pub const fn render_component(&self) -> bool {
        match self.component {
            Some(component) => component.render_component(),
            None => false,
        }
    }

    pub const fn value_binding(&self) -> bool {
        match self.component {
            Some(component) => component.value_binding(),
            None => false,
        }
    }

    pub const fn prototyping_field_suffix(&self) -> Option<ComponentSuffix> {
        match self.component {
            Some(component) => component.prototyping_field_suffix(),
            None => None,
        }
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
            .and_then(|suffix| {
                component_shape::component_suffix_from_suffix(
                    self.field_name.as_str(),
                    suffix.as_str(),
                )
            })
            .filter(|suffix| !suffix.is_empty())
            .unwrap_or_else(|| "shape".to_string())
    }

    pub fn field_name_pascal(&self) -> String {
        self.field_name.as_str().to_pascal_case()
    }

    pub fn field_name_with_component_suffix(&self) -> String {
        format!("{}_{}", self.field_name, self.component_suffix())
    }

    pub fn kebab_id(&self) -> String {
        self.field_name_with_component_suffix().to_kebab_case()
    }

    /// Returns the validation rule identifiers attached to this field.
    pub fn validation_rules(&self) -> &'static [ValidationRuleId] {
        self.validations
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ComponentFieldName, ComponentSuffix, FieldComponentVariant, FieldValuePresence,
        FieldValueSpec, FieldVariant, GpuiFormShape, HolderConversionMetadata,
        HolderConversionShape, RustExpr, RustPath, RustSyntaxKind, RustType,
        is_valid_component_suffix, validate_component_suffix,
    };

    const fn value_spec(
        value_type: &'static str,
        value_presence: FieldValuePresence,
    ) -> FieldValueSpec {
        let value_type = RustType::from_macro_tokens_unchecked(value_type);
        FieldValueSpec::new(value_type, value_type, value_presence)
    }

    const fn shape_field(
        field_name: &'static str,
        value_type: &'static str,
        shape_path: &'static str,
    ) -> FieldVariant {
        FieldVariant::component(
            ComponentFieldName::new(field_name),
            value_spec(value_type, FieldValuePresence::RequiresValue),
            FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(shape_path)),
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
            ComponentFieldName::new("country"),
            value_spec("Country", FieldValuePresence::RequiresValue),
            FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
                "crate::fields::CountrySelectorState",
            ))
            .with_prototyping_field_suffix(Some(ComponentSuffix::new("select"))),
        );

        assert_eq!(field.field_name_with_component_suffix(), "country_select");
    }

    #[test]
    fn prototyping_suffix_removes_duplicate_field_prefix() {
        let field = FieldVariant::component(
            ComponentFieldName::new("email"),
            value_spec("String", FieldValuePresence::RequiresValue),
            FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
                "crate::fields::TextInputShape",
            ))
            .with_prototyping_field_suffix(Some(ComponentSuffix::new("email_input"))),
        );

        assert_eq!(field.field_name_with_component_suffix(), "email_input");
    }

    #[test]
    fn prototyping_suffix_exact_duplicate_uses_shape_fallback() {
        let field = FieldVariant::component(
            ComponentFieldName::new("tags"),
            value_spec("Vec<String>", FieldValuePresence::RequiresValue),
            FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
                "crate::fields::TagsInputShape",
            ))
            .with_prototyping_field_suffix(Some(ComponentSuffix::new("tags"))),
        );

        assert_eq!(field.field_name_with_component_suffix(), "tags_shape");
    }

    #[test]
    fn optional_presence_uses_value_holder_option() {
        let field = FieldVariant::hidden(
            ComponentFieldName::new("email"),
            value_spec("String", FieldValuePresence::Optional),
        );

        assert_eq!(field.field_name().as_str(), "email");
        assert!(field.optional());
        assert!(!field.requires_value());
        assert!(field.value_holder_uses_option());
    }

    #[test]
    fn holder_conversion_metadata_encodes_api_shape() {
        const SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &[],
            RustPath::from_macro_tokens_unchecked("crate::demo"),
            false,
        )
        .with_holder_conversion(HolderConversionMetadata::new(
            HolderConversionShape::NeedsSkippedFields,
            true,
        ));

        assert_eq!(
            SHAPE.holder_conversion_shape(),
            HolderConversionShape::NeedsSkippedFields
        );
        assert!(SHAPE.has_skipped_fields());
        assert!(SHAPE.holder_conversion().uses_fallible_api());
        assert!(SHAPE.holder_conversion().runtime_can_fail());
    }

    #[test]
    fn component_metadata_is_constructed_before_field_variant() {
        let component = FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
            "crate::fields::EmailInputShape",
        ))
        .with_render_component(true)
        .with_value_binding(true)
        .with_prototyping_field_suffix(Some(ComponentSuffix::new("input")));
        let field = FieldVariant::component(
            ComponentFieldName::new("email"),
            value_spec("String", FieldValuePresence::DirectStorage),
            component,
        );

        assert_eq!(
            field.shape_path().map(RustPath::as_str),
            Some("crate::fields::EmailInputShape")
        );
        assert!(field.render_component());
        assert!(field.value_binding());
        assert!(!field.value_holder_uses_option());
    }

    #[test]
    fn component_variant_stores_neutral_shape_use_metadata() {
        let field = FieldVariant::component(
            ComponentFieldName::new("email"),
            value_spec("String", FieldValuePresence::DirectStorage),
            FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
                "crate::fields::EmailInputShape",
            ))
            .with_prototyping_field_suffix(Some(ComponentSuffix::new("input"))),
        );

        let shape_use = field
            .component_variant()
            .expect("component metadata should be attached")
            .shape_use;

        assert_eq!(shape_use.field_name().as_str(), "email");
        assert_eq!(shape_use.field_type().map(RustType::as_str), Some("String"));
        assert_eq!(
            shape_use.shape_path().as_str(),
            "crate::fields::EmailInputShape"
        );
        assert_eq!(
            shape_use
                .prototyping()
                .field_suffix
                .map(ComponentSuffix::as_str),
            Some("input")
        );
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
            "crate::shapes::InfiniteSelect<Country>",
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

    #[test]
    fn field_value_presence_names_are_stable_schema_metadata() {
        assert_eq!(FieldValuePresence::Optional.as_str(), "optional");
        assert_eq!(FieldValuePresence::RequiresValue.as_str(), "requires_value");
        assert_eq!(FieldValuePresence::DirectStorage.as_str(), "direct_storage");
    }
}
