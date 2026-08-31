use gpui_form_schema::registry::{FieldValuePresence, RustPath, RustType};

use crate::*;

pub type FieldToolValueSchemaFn = McpSchemaFn;

#[derive(Clone, Copy, Debug)]
pub struct McpField {
    name: &'static str,
    value_type: RustType,
    presence: FieldValuePresence,
    has_default: bool,
    component_backed: bool,
    mcp_input: McpInput,
    tool_value_schema: FieldToolValueSchemaFn,
    label: Option<&'static str>,
    description: Option<&'static str>,
    examples: &'static [&'static str],
    validation_rules: &'static [McpValidationRule],
}

impl McpField {
    pub const fn typed<T>(
        name: &'static str,
        value_type: RustType,
        presence: FieldValuePresence,
    ) -> Self
    where
        T: McpToolValue,
    {
        Self {
            name,
            value_type,
            presence,
            has_default: false,
            component_backed: false,
            mcp_input: McpInput::unsupported(),
            tool_value_schema: T::tool_value_schema,
            label: None,
            description: None,
            examples: &[],
            validation_rules: &[],
        }
    }

    /// Describe a component-backed form field.
    ///
    /// The generated JSON Schema comes from `T`'s [`McpToolValue`]
    /// implementation. The shape/value pair contributes additional
    /// [`ComponentShapeFor::MCP_INPUT`] metadata when it publishes one.
    pub const fn component<T, Shape>(
        name: &'static str,
        value_type: RustType,
        presence: FieldValuePresence,
    ) -> Self
    where
        T: McpToolValue,
        Shape: ComponentShapeFor<T>,
    {
        Self {
            name,
            value_type,
            presence,
            has_default: false,
            component_backed: true,
            mcp_input: <Shape as ComponentShapeFor<T>>::MCP_INPUT,
            tool_value_schema: T::tool_value_schema,
            label: None,
            description: None,
            examples: &[],
            validation_rules: &[],
        }
    }

    pub const fn with_default(mut self, has_default: bool) -> Self {
        self.has_default = has_default;
        self
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

    pub const fn with_validation_rules(
        mut self,
        validation_rules: &'static [McpValidationRule],
    ) -> Self {
        self.validation_rules = validation_rules;
        self
    }

    pub const fn name(self) -> &'static str {
        self.name
    }

    pub const fn value_type(self) -> RustType {
        self.value_type
    }

    pub const fn presence(self) -> FieldValuePresence {
        self.presence
    }

    pub const fn has_default(self) -> bool {
        self.has_default
    }

    pub const fn component_backed(self) -> bool {
        self.component_backed
    }

    pub const fn mcp_input(self) -> McpInput {
        self.mcp_input
    }

    pub const fn tool_value_schema(self) -> FieldToolValueSchemaFn {
        self.tool_value_schema
    }

    pub const fn explicit_label(self) -> Option<&'static str> {
        self.label
    }

    pub fn label(self) -> String {
        self.label
            .map(str::to_string)
            .unwrap_or_else(|| label_from_field_name(self.name))
    }

    pub const fn description(self) -> Option<&'static str> {
        self.description
    }

    pub const fn examples(self) -> &'static [&'static str] {
        self.examples
    }

    pub const fn validation_rules(self) -> &'static [McpValidationRule] {
        self.validation_rules
    }

    pub const fn required(self) -> bool {
        !self.presence.optional() && !self.has_default
    }
}

fn label_from_field_name(name: &str) -> String {
    let mut label = String::with_capacity(name.len());
    let mut previous_was_separator = false;
    for ch in name.chars() {
        if ch == '_' || ch == '-' {
            if !label.is_empty() {
                previous_was_separator = true;
            }
            continue;
        }
        if label.is_empty() {
            label.extend(ch.to_uppercase());
        } else if previous_was_separator {
            label.push(' ');
            label.extend(ch.to_lowercase());
        } else {
            label.push(ch);
        }
        previous_was_separator = false;
    }
    label
}

#[derive(Clone, Copy, Debug)]
pub struct McpFormDescriptor {
    form_name: &'static str,
    source_module_path: RustPath,
    fields: &'static [McpField],
    koruma_enabled: bool,
    tool_metadata: McpToolMetadata,
}

impl McpFormDescriptor {
    pub const fn new(
        form_name: &'static str,
        source_module_path: RustPath,
        fields: &'static [McpField],
        koruma_enabled: bool,
        tool_metadata: McpToolMetadata,
    ) -> Self {
        Self {
            form_name,
            source_module_path,
            fields,
            koruma_enabled,
            tool_metadata,
        }
    }

    pub const fn form_name(self) -> &'static str {
        self.form_name
    }

    pub const fn source_module_path(self) -> RustPath {
        self.source_module_path
    }

    pub const fn fields(self) -> &'static [McpField] {
        self.fields
    }

    pub const fn koruma_enabled(self) -> bool {
        self.koruma_enabled
    }

    pub const fn tool_metadata(self) -> McpToolMetadata {
        self.tool_metadata
    }

    pub fn tool_name(self) -> String {
        self.tool_metadata
            .name()
            .map(str::to_string)
            .unwrap_or_else(|| tool_name(self.source_module_path.as_str(), self.form_name))
    }

    pub fn title(self) -> String {
        self.tool_metadata
            .title()
            .map(str::to_string)
            .unwrap_or_else(|| format!("{} submit", self.form_name))
    }

    pub fn description(self) -> String {
        self.tool_metadata
            .description()
            .map(str::to_string)
            .unwrap_or_else(|| format!("Submit a {} gpui-form request.", self.form_name))
    }

    pub fn input_schema(self) -> McpSchema {
        input_schema_for_fields(self.fields)
    }

    pub fn output_schema<Response>(self) -> McpSchema
    where
        Response: McpJsonSchema,
    {
        Response::json_schema()
    }

    pub fn tool_annotations(self) -> Option<McpToolAnnotations> {
        let metadata = self.tool_metadata;
        let read_only = metadata.read_only_hint().unwrap_or(false);
        let destructive = metadata.destructive_hint().unwrap_or(!read_only);
        let idempotent = metadata.idempotent_hint().unwrap_or(false);
        let open_world = metadata.open_world_hint().unwrap_or(true);

        Some(McpToolAnnotations::from_raw(
            Some(self.title()),
            Some(read_only),
            Some(destructive),
            Some(idempotent),
            Some(open_world),
        ))
    }

    pub fn tool_icons(self) -> Option<Vec<component_shape_mcp::McpIcon>> {
        self.tool_metadata.tool_icons()
    }

    pub(crate) fn apply_tool_metadata(self, tool: &mut ToolDefinition) {
        tool.annotations = self.tool_annotations();
        tool.icons = self.tool_icons();
    }

    pub(crate) fn tool_definition(self) -> Result<ToolDefinition, McpToolError> {
        self.tool_metadata.validate()?;
        let mut tool = component_shape_mcp::tool_definition_with_annotations(
            self.tool_name(),
            Some(self.title()),
            Some(self.description()),
            self.input_schema(),
            None,
            None,
        )?;
        self.apply_tool_metadata(&mut tool);
        component_shape_mcp::validate_tool_definition(&tool)?;
        Ok(tool)
    }
}
