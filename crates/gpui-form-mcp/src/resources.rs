use std::{any::TypeId, collections::BTreeSet};

use serde_json::{Map, Value};

use crate::*;

/// MCP resource URIs generated for one form descriptor.
///
/// The `{tool_name}` segment is the form's MCP tool name, including any
/// struct-level `#[gpui_form(mcp(name = "..."))]` override.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpFormResourceUris {
    /// JSON resource with form, tool, field, validation, and per-field schema metadata.
    pub descriptor: String,
    /// JSON Schema resource for the generated submit input.
    pub schema: String,
    /// JSON resource grouping field-level examples.
    pub examples: String,
}

impl McpFormResourceUris {
    /// Return all generated resource URIs in descriptor, schema, examples order.
    pub fn all(&self) -> [&str; 3] {
        [
            self.descriptor.as_str(),
            self.schema.as_str(),
            self.examples.as_str(),
        ]
    }
}

pub(crate) type McpFormResourceSpec = component_shape_mcp::McpJsonResourceSpec;

/// Return the three MCP resource URIs generated for `descriptor`.
pub fn form_resource_uris(descriptor: McpFormDescriptor) -> McpFormResourceUris {
    let base = format!("gpui-form://forms/{}", descriptor.tool_name());
    McpFormResourceUris {
        descriptor: format!("{base}/descriptor"),
        schema: format!("{base}/schema"),
        examples: format!("{base}/examples"),
    }
}

/// input metadata, validation rules, and per-field schemas.
pub fn form_descriptor_resource_value(descriptor: McpFormDescriptor) -> Value {
    let resources = form_resource_uris(descriptor);
    serde_json::json!({
        "form_name": descriptor.form_name(),
        "source_module_path": descriptor.source_module_path().as_str(),
        "tool_name": descriptor.tool_name(),
        "title": descriptor.title(),
        "description": descriptor.description(),
        "koruma_enabled": descriptor.koruma_enabled(),
        "resources": {
            "descriptor": resources.descriptor,
            "schema": resources.schema,
            "examples": resources.examples,
        },
        "fields": descriptor
            .fields()
            .iter()
            .map(|field| form_field_descriptor_value(*field))
            .collect::<Vec<_>>(),
    })
}

/// Build the JSON Schema value served by a form's schema resource.
pub fn form_schema_resource_value(descriptor: McpFormDescriptor) -> Value {
    descriptor.input_schema().into_value()
}

/// Build the JSON value served by a form's examples resource.
pub fn form_examples_resource_value(descriptor: McpFormDescriptor) -> Value {
    let fields = descriptor
        .fields()
        .iter()
        .filter(|field| !field.examples().is_empty())
        .map(|field| {
            serde_json::json!({
                "name": field.name(),
                "label": field.label(),
                "examples": field.examples(),
            })
        })
        .collect::<Vec<_>>();
    let mut examples = Map::new();
    for field in descriptor.fields() {
        if !field.examples().is_empty() {
            examples.insert(
                field.name().to_string(),
                Value::Array(
                    field
                        .examples()
                        .iter()
                        .map(|example| Value::String((*example).to_string()))
                        .collect(),
                ),
            );
        }
    }

    serde_json::json!({
        "form_name": descriptor.form_name(),
        "tool_name": descriptor.tool_name(),
        "fields": fields,
        "examples": examples,
    })
}

pub(crate) fn form_field_descriptor_value(field: McpField) -> Value {
    let mut object = Map::new();
    object.insert("name".to_string(), Value::String(field.name().to_string()));
    object.insert("label".to_string(), Value::String(field.label()));
    if let Some(label) = field.explicit_label() {
        object.insert(
            "explicit_label".to_string(),
            Value::String(label.to_string()),
        );
    }
    if let Some(description) = field.description() {
        object.insert(
            "description".to_string(),
            Value::String(description.to_string()),
        );
    }
    if !field.examples().is_empty() {
        object.insert(
            "examples".to_string(),
            Value::Array(
                field
                    .examples()
                    .iter()
                    .map(|example| Value::String((*example).to_string()))
                    .collect(),
            ),
        );
    }
    object.insert(
        "value_type".to_string(),
        Value::String(field.value_type().as_str().to_string()),
    );
    object.insert(
        "presence".to_string(),
        Value::String(field.presence().as_str().to_string()),
    );
    object.insert("required".to_string(), Value::Bool(field.required()));
    object.insert("has_default".to_string(), Value::Bool(field.has_default()));
    object.insert(
        "component_backed".to_string(),
        Value::Bool(field.component_backed()),
    );
    object.insert(
        "mcp_input".to_string(),
        component_shape_mcp::mcp_input_descriptor_value(field.mcp_input()),
    );
    if !field.validation_rules().is_empty() {
        object.insert(
            "validation_rules".to_string(),
            Value::Array(
                field
                    .validation_rules()
                    .iter()
                    .map(|rule| rule.to_value())
                    .collect(),
            ),
        );
    }
    object.insert("schema".to_string(), schema_for_field(field).into_value());
    Value::Object(object)
}

/// Register descriptor, schema, and examples resources for one form.
///
/// Direct submit/editor registration calls this automatically. Use this helper
/// only when a server should expose form resources without registering form
/// tools.
pub fn register_form_resources<Form>(server: &mut McpServer) -> Result<(), McpToolError>
where
    Form: McpForm,
{
    register_form_resources_for_descriptor(server, Form::descriptor())
}

pub(crate) fn register_form_resources_for_descriptor(
    server: &mut McpServer,
    descriptor: McpFormDescriptor,
) -> Result<(), McpToolError> {
    let specs = form_resource_specs(descriptor)?;
    component_shape_mcp::register_json_resource_specs(server, specs)
}

pub(crate) fn form_resource_specs(
    descriptor: McpFormDescriptor,
) -> Result<Vec<McpFormResourceSpec>, McpToolError> {
    let uris = form_resource_uris(descriptor);
    let tool_name = descriptor.tool_name();
    let title = descriptor.title();
    Ok(vec![
        form_json_resource_spec(
            uris.descriptor,
            format!("{tool_name}_descriptor"),
            format!("{title} descriptor"),
            format!("Field metadata for the {title} generated form."),
            form_descriptor_resource_value(descriptor),
        )?,
        form_json_resource_spec(
            uris.schema,
            format!("{tool_name}_schema"),
            format!("{title} schema"),
            format!("Input JSON Schema for the {title} generated form."),
            form_schema_resource_value(descriptor),
        )?,
        form_json_resource_spec(
            uris.examples,
            format!("{tool_name}_examples"),
            format!("{title} examples"),
            format!("Field examples for the {title} generated form."),
            form_examples_resource_value(descriptor),
        )?,
    ])
}

pub(crate) fn form_json_resource_spec(
    uri: String,
    name: String,
    title: String,
    description: String,
    value: Value,
) -> Result<McpFormResourceSpec, McpToolError> {
    component_shape_mcp::McpJsonResourceSpec::new(uri, name, Some(title), Some(description), value)
}

pub(crate) fn register_form_resource_specs_if_missing(
    server: &mut McpServer,
    specs: Vec<McpFormResourceSpec>,
) -> Result<(), McpToolError> {
    component_shape_mcp::register_json_resource_specs_if_missing(server, specs)
}

pub(crate) fn form_resource_specs_for_options(
    options: McpFormRegistrationOptions,
) -> Result<Vec<McpFormResourceSpec>, McpToolError> {
    if !options.registers_tools() {
        return Ok(Vec::new());
    }

    let mut seen_tool_names = BTreeSet::new();
    let mut specs = Vec::new();
    if options.submit_handlers {
        for registration in registry::submit_handler_registrations() {
            push_descriptor_resource_specs(
                &mut seen_tool_names,
                &mut specs,
                registration.descriptor(),
            )?;
        }
    }
    if options.editors {
        for registration in registry::editor_registrations() {
            push_descriptor_resource_specs(
                &mut seen_tool_names,
                &mut specs,
                registration.descriptor(),
            )?;
        }
    }
    Ok(specs)
}

pub(crate) fn context_submit_resource_specs<Context>()
-> Result<Vec<McpFormResourceSpec>, McpToolError>
where
    Context: Clone + Send + Sync + 'static,
{
    let mut seen_tool_names = BTreeSet::new();
    let mut specs = Vec::new();
    let context_type_id = TypeId::of::<Context>();
    for registration in registry::context_submit_registrations() {
        if registration.context_type_id() == context_type_id {
            push_descriptor_resource_specs(
                &mut seen_tool_names,
                &mut specs,
                registration.descriptor(),
            )?;
        }
    }
    Ok(specs)
}

pub(crate) fn push_descriptor_resource_specs(
    seen_tool_names: &mut BTreeSet<String>,
    specs: &mut Vec<McpFormResourceSpec>,
    descriptor: McpFormDescriptor,
) -> Result<(), McpToolError> {
    if !seen_tool_names.insert(descriptor.tool_name()) {
        return Ok(());
    }
    specs.extend(form_resource_specs(descriptor)?);
    Ok(())
}
