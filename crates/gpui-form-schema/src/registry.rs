use crate::components::ComponentsBehaviour;
use heck::{ToKebabCase as _, ToPascalCase as _};

inventory::collect!(GpuiFormShape);

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
    pub value_type: &'static str,
    /// Rust type path for the source model's base value type before any
    /// `#[gpui_form(type = ...)]` override is applied.
    pub source_value_type: &'static str,
    pub optional: bool,
    /// Whether the generated value holder wraps this field in `Option<T>`
    /// because of component behavior. Source `Option<T>` fields are tracked by
    /// [`FieldVariant::optional`].
    pub wraps_in_option: bool,
    pub behaviour: ComponentsBehaviour,
    /// List of validation rule identifiers applied to this field (for diagnostics/rendering).
    pub validations: &'static [&'static str],
    /// Default value expression as a string, if one was specified.
    pub default_expr: Option<&'static str>,
    /// Source-to-form conversion expression, if one was specified.
    pub from_expr: Option<&'static str>,
    /// Form-to-source conversion expression, if one was specified.
    pub into_expr: Option<&'static str>,
    /// For custom components: the shape type path implementing
    /// `gpui_form_component::custom::CustomComponentShape`.
    pub custom_shape: Option<&'static str>,
    /// For custom components: the UI component type path (e.g. "TagsInput").
    /// Used by the prototyping code generator to emit `Component::new(&entity)`.
    pub custom_component: Option<&'static str>,
    /// Whether the custom component opted into
    /// `gpui_form_component::custom::CustomComponentValueAdapter` generation.
    pub custom_value_binding: bool,
}

impl FieldVariant {
    pub const fn new(
        field_name: &'static str,
        value_type: &'static str,
        optional: bool,
        behaviour: ComponentsBehaviour,
    ) -> Self {
        Self {
            field_name,
            value_type,
            source_value_type: value_type,
            optional,
            wraps_in_option: behaviour.kind().default_wraps_in_option(),
            behaviour,
            validations: &[],
            default_expr: None,
            from_expr: None,
            into_expr: None,
            custom_shape: None,
            custom_component: None,
            custom_value_binding: false,
        }
    }

    /// Attach the source model value type when it differs from the form-side value type.
    pub const fn with_source_value_type(mut self, source_value_type: &'static str) -> Self {
        self.source_value_type = source_value_type;
        self
    }

    /// Attach the component-driven generated value-holder wrapping policy.
    pub const fn with_wraps_in_option(mut self, wraps_in_option: bool) -> Self {
        self.wraps_in_option = wraps_in_option;
        self
    }

    /// Attach optional source/form conversion expressions.
    pub const fn with_conversions(
        mut self,
        from_expr: Option<&'static str>,
        into_expr: Option<&'static str>,
    ) -> Self {
        self.from_expr = from_expr;
        self.into_expr = into_expr;
        self
    }

    /// Attach a default value expression to this field metadata.
    pub const fn with_default(mut self, default_expr: &'static str) -> Self {
        self.default_expr = Some(default_expr);
        self
    }

    /// Attach a custom UI component path to this field metadata.
    pub const fn with_custom_component(mut self, component: &'static str) -> Self {
        self.custom_component = Some(component);
        self
    }

    /// Attach an optional custom UI component path to this field metadata.
    ///
    /// Used when the component path may come from the shape's
    /// `CustomComponentShape::COMPONENT_PATH` constant rather than an explicit
    /// field attribute value.
    pub const fn with_custom_component_opt(mut self, component: Option<&'static str>) -> Self {
        self.custom_component = component;
        self
    }

    /// Attach the custom component shape type path.
    pub const fn with_custom_shape(mut self, shape: &'static str) -> Self {
        self.custom_shape = Some(shape);
        self
    }

    /// Marks this custom component as value-bound for generated prototyping code.
    pub const fn with_custom_value_binding(mut self, enabled: bool) -> Self {
        self.custom_value_binding = enabled;
        self
    }

    /// Returns true when the generated value holder stores this field as `Option<T>`.
    pub const fn value_holder_wraps_in_option(&self) -> bool {
        self.optional || self.wraps_in_option
    }

    /// Returns true when generated code should subscribe to this field.
    pub const fn subscribable(&self) -> bool {
        self.behaviour.kind().subscribable() || self.custom_value_binding
    }

    pub fn behaviour_suffix(&self) -> &'static str {
        self.behaviour.component_name()
    }

    pub fn field_name_pascal(&self) -> String {
        self.field_name.to_pascal_case()
    }

    pub fn field_name_with_behaviour(&self) -> String {
        format!("{}_{}", self.field_name, self.behaviour_suffix())
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

pub use inventory;
