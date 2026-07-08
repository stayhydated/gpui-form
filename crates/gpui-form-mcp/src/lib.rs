//! Experimental MCP submit integration for generated `gpui-form` value holders.
//!
//! This crate intentionally keeps GPUI out of the submit execution path. It
//! owns form holder/model decoding and validation while delegating shared MCP
//! server and stdio serving mechanics to `component-shape-mcp`.

use std::{
    any::{Any, TypeId},
    collections::{BTreeMap, BTreeSet},
    fmt,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use gpui_form_schema::registry::{FieldValuePresence, RustPath, RustType};
pub use serde::Serialize;
use serde_json::{Map, Value};

pub use component_shape_mcp::{
    ComponentShapeFor, ComponentShapeMetadata, ContentBlock, MCP_PROTOCOL_VERSION,
    MCP_VALIDATION_PARAMS_NONE, McpAny, McpArguments, McpIconTheme, McpInput, McpInputShape,
    McpJsonSchema, McpPrimitiveKind, McpPromptArgument, McpPromptMessage, McpPromptMessageContent,
    McpPromptMessageRole, McpPromptResult, McpRange, McpRangeBoundKind, McpResourceContents,
    McpResourceResult, McpSchema, McpSchemaFn, McpSchemaProperties, McpServer, McpServerBuilder,
    McpToolAnnotations, McpToolArguments, McpToolCall, McpToolError, McpToolIcon, McpToolInput,
    McpToolMetadata, McpToolTaskSupport, McpToolValue, McpTypedTool, McpValidationIssue,
    McpValidationParam, McpValidationRule, McpValidationScope, McpValidationTarget,
    McpValidationTypeArgMode, PromptDefinition, RawResourceDefinition,
    RawResourceTemplateDefinition, ResourceDefinition, ResourceTemplateDefinition,
    ServeStdioResult, ToolCallResult, ToolDefinition, object_schema, rmcp, serde, serde_json,
    validation_issues_error,
};

pub type FieldToolValueSchemaFn = McpSchemaFn;
type ToolFuture = Pin<Box<dyn Future<Output = ToolCallResult> + Send + 'static>>;
pub type McpSubmitContext = Arc<dyn Any + Send + Sync>;

pub const DEFAULT_EDITOR_SESSION_LIMIT: usize = 128;
pub const DEFAULT_EDITOR_SESSION_IDLE_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// Generated MCP tool registration profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct McpFormRegistrationOptions {
    pub submit_handlers: bool,
    pub editors: bool,
    pub edit_submit: bool,
}

impl McpFormRegistrationOptions {
    /// Register submit handlers, standalone editors, and handler-backed
    /// edit-submit tools.
    pub const fn all() -> Self {
        Self {
            submit_handlers: true,
            editors: true,
            edit_submit: true,
        }
    }

    /// Register only `#[gpui_form::mcp_submit]` submit tools.
    pub const fn submit_only() -> Self {
        Self {
            submit_handlers: true,
            editors: false,
            edit_submit: false,
        }
    }

    /// Register only editable holder-session tools.
    pub const fn editor_only() -> Self {
        Self {
            submit_handlers: false,
            editors: true,
            edit_submit: false,
        }
    }

    /// Build registration metadata without mutating the server.
    pub const fn metadata_only() -> Self {
        Self {
            submit_handlers: false,
            editors: false,
            edit_submit: false,
        }
    }

    const fn registers_tools(self) -> bool {
        self.submit_handlers || self.editors || self.edit_submit
    }
}

impl Default for McpFormRegistrationOptions {
    fn default() -> Self {
        Self::all()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpSkippedTool {
    pub name: String,
    pub reason: &'static str,
}

#[derive(Clone, Debug, Default)]
pub struct McpFormRegistrationReport {
    pub tool_definitions: Vec<ToolDefinition>,
    pub registered_tool_names: Vec<String>,
    pub skipped_tools: Vec<McpSkippedTool>,
    pub context_registration_count: usize,
}

impl McpFormRegistrationReport {
    fn from_tool_definitions(tool_definitions: Vec<ToolDefinition>) -> Self {
        let registered_tool_names = tool_definitions
            .iter()
            .map(|definition| definition.name.to_string())
            .collect();
        Self {
            tool_definitions,
            registered_tool_names,
            skipped_tools: Vec::new(),
            context_registration_count: 0,
        }
    }

    fn metadata_only(mut self) -> Self {
        self.skipped_tools.extend(
            self.registered_tool_names
                .iter()
                .map(|name| McpSkippedTool {
                    name: name.clone(),
                    reason: "metadata_only",
                }),
        );
        self.registered_tool_names.clear();
        self
    }
}

/// Runtime policy for generated MCP edit-session tools.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct McpFormEditorOptions {
    session_limit: Option<usize>,
    session_idle_timeout: Option<Duration>,
}

impl McpFormEditorOptions {
    /// Create editor options using the default bounded session pool.
    pub const fn new() -> Self {
        Self {
            session_limit: Some(DEFAULT_EDITOR_SESSION_LIMIT),
            session_idle_timeout: Some(DEFAULT_EDITOR_SESSION_IDLE_TIMEOUT),
        }
    }

    /// Limit the number of active edit sessions retained for one form.
    ///
    /// Opening a session beyond this limit evicts the oldest active sessions.
    /// A zero value is clamped to one because opening a session should always
    /// return a readable session.
    pub fn with_session_limit(mut self, session_limit: usize) -> Self {
        self.session_limit = Some(session_limit.max(1));
        self
    }

    /// Keep all sessions until the client closes them.
    pub const fn without_session_limit(mut self) -> Self {
        self.session_limit = None;
        self
    }

    pub const fn session_limit(self) -> Option<usize> {
        self.session_limit
    }

    /// Expire edit sessions that have not been accessed for this duration.
    ///
    /// Expiry is enforced lazily before editor operations. `Duration::ZERO`
    /// expires a session before the next operation after it is opened.
    pub const fn with_session_idle_timeout(mut self, session_idle_timeout: Duration) -> Self {
        self.session_idle_timeout = Some(session_idle_timeout);
        self
    }

    /// Keep idle sessions until the client closes them or the session cap
    /// evicts them.
    pub const fn without_session_idle_timeout(mut self) -> Self {
        self.session_idle_timeout = None;
        self
    }

    pub const fn session_idle_timeout(self) -> Option<Duration> {
        self.session_idle_timeout
    }
}

impl Default for McpFormEditorOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Dynamic JSON object response for MCP submit tools.
///
/// Use this when a handler returns object-shaped JSON but does not need a
/// dedicated response struct. It serializes transparently as the wrapped object
/// and publishes a generic object output schema.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct McpObject(Map<String, Value>);

impl McpObject {
    pub fn new(object: Map<String, Value>) -> Self {
        Self(object)
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn from_value(value: Value) -> Result<Self, McpToolError> {
        match value {
            Value::Object(object) => Ok(Self::new(object)),
            _ => Err(McpToolError::decode("response", "expected an object")),
        }
    }

    pub fn as_map(&self) -> &Map<String, Value> {
        &self.0
    }

    pub fn into_map(self) -> Map<String, Value> {
        self.0
    }

    pub fn into_value(self) -> Value {
        Value::Object(self.into_map())
    }
}

impl From<Map<String, Value>> for McpObject {
    fn from(object: Map<String, Value>) -> Self {
        Self::new(object)
    }
}

impl TryFrom<Value> for McpObject {
    type Error = McpToolError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Self::from_value(value)
    }
}

impl From<McpObject> for Value {
    fn from(object: McpObject) -> Self {
        object.into_value()
    }
}

impl McpJsonSchema for McpObject {
    fn json_schema() -> McpSchema {
        McpSchema::object()
    }
}

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

    pub fn tool_execution(self) -> Option<component_shape_mcp::McpToolExecution> {
        self.tool_metadata.tool_execution()
    }

    fn apply_tool_metadata(self, tool: &mut ToolDefinition) {
        tool.annotations = self.tool_annotations();
        tool.icons = self.tool_icons();
        tool.execution = self.tool_execution();
    }

    fn tool_definition(self) -> Result<ToolDefinition, McpToolError> {
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

type McpFormResourceSpec = component_shape_mcp::McpJsonResourceSpec;

/// MCP prompt template names generated for one form descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpFormPromptNames {
    /// Prompt for drafting a complete direct-submit argument object.
    pub fill: String,
    /// Prompt for repairing invalid arguments or an edit-session patch.
    pub repair: String,
    /// Prompt for choosing direct submit versus edit-session submit.
    pub submit: String,
}

impl McpFormPromptNames {
    /// Return all generated prompt names in fill, repair, submit order.
    pub fn all(&self) -> [&str; 3] {
        [
            self.fill.as_str(),
            self.repair.as_str(),
            self.submit.as_str(),
        ]
    }
}

#[derive(Clone, Copy)]
enum McpFormPromptKind {
    Fill,
    Repair,
    Submit,
}

struct McpFormPromptSpec {
    name: String,
    definition: PromptDefinition,
    descriptor: McpFormDescriptor,
    kind: McpFormPromptKind,
}

/// Return the three MCP resource URIs generated for `descriptor`.
pub fn form_resource_uris(descriptor: McpFormDescriptor) -> McpFormResourceUris {
    let base = format!("gpui-form://forms/{}", descriptor.tool_name());
    McpFormResourceUris {
        descriptor: format!("{base}/descriptor"),
        schema: format!("{base}/schema"),
        examples: format!("{base}/examples"),
    }
}

/// Return generated MCP prompt template names for `descriptor`.
pub fn form_prompt_names(descriptor: McpFormDescriptor) -> McpFormPromptNames {
    let tool_name = descriptor.tool_name();
    McpFormPromptNames {
        fill: format!("fill_{tool_name}_form"),
        repair: format!("repair_{tool_name}_form"),
        submit: format!("submit_{tool_name}_form"),
    }
}

/// Build the JSON value served by a form's descriptor resource.
///
/// The descriptor includes resource links, form/tool metadata, field labels,
/// descriptions, examples, required/default/storage metadata, component MCP
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

fn form_field_descriptor_value(field: McpField) -> Value {
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

fn register_form_resources_for_descriptor(
    server: &mut McpServer,
    descriptor: McpFormDescriptor,
) -> Result<(), McpToolError> {
    let specs = form_resource_specs(descriptor)?;
    component_shape_mcp::register_json_resource_specs(server, specs)
}

fn form_resource_specs(
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

fn form_json_resource_spec(
    uri: String,
    name: String,
    title: String,
    description: String,
    value: Value,
) -> Result<McpFormResourceSpec, McpToolError> {
    component_shape_mcp::McpJsonResourceSpec::new(uri, name, Some(title), Some(description), value)
}

fn register_form_resource_specs_if_missing(
    server: &mut McpServer,
    specs: Vec<McpFormResourceSpec>,
) -> Result<(), McpToolError> {
    component_shape_mcp::register_json_resource_specs_if_missing(server, specs)
}

fn form_resource_specs_for_options(
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

fn context_submit_resource_specs<Context>() -> Result<Vec<McpFormResourceSpec>, McpToolError>
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

fn push_descriptor_resource_specs(
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

/// Register generated fill, repair, and submit prompt templates for one form.
///
/// Prompt templates are opt-in and reference the generated descriptor, schema,
/// and examples resources. This helper registers those resources first if they
/// are not already present.
pub fn register_form_prompt_templates<Form>(server: &mut McpServer) -> Result<(), McpToolError>
where
    Form: McpForm,
{
    register_form_prompt_templates_for_descriptor(server, Form::descriptor())
}

fn register_form_prompt_templates_for_descriptor(
    server: &mut McpServer,
    descriptor: McpFormDescriptor,
) -> Result<(), McpToolError> {
    let resource_specs = form_resource_specs(descriptor)?;
    let prompt_specs = form_prompt_specs(descriptor)?;
    register_form_resource_specs_if_missing(server, resource_specs)?;
    ensure_prompt_specs_available(server, &prompt_specs)?;
    register_form_prompt_specs(server, prompt_specs)
}

fn form_prompt_specs(
    descriptor: McpFormDescriptor,
) -> Result<Vec<McpFormPromptSpec>, McpToolError> {
    let names = form_prompt_names(descriptor);
    let title = descriptor.title();
    Ok(vec![
        form_prompt_spec(
            names.fill,
            format!("Fill {title}"),
            format!("Draft valid direct-submit arguments for {title}."),
            vec![optional_prompt_argument(
                "goal",
                "Optional user goal or context for filling the form.",
            )],
            descriptor,
            McpFormPromptKind::Fill,
        )?,
        form_prompt_spec(
            names.repair,
            format!("Repair {title}"),
            format!("Repair invalid arguments or edit-session values for {title}."),
            vec![
                optional_prompt_argument(
                    "validation_issues",
                    "Validation issue details from the last failed submit or edit validation.",
                ),
                optional_prompt_argument(
                    "current_values",
                    "Current direct-submit arguments or edit-session values.",
                ),
            ],
            descriptor,
            McpFormPromptKind::Repair,
        )?,
        form_prompt_spec(
            names.submit,
            format!("Submit {title}"),
            format!("Choose a direct submit or edit-session submit path for {title}."),
            vec![optional_prompt_argument(
                "workflow",
                "Optional workflow notes, such as direct submit or edit-session submit.",
            )],
            descriptor,
            McpFormPromptKind::Submit,
        )?,
    ])
}

fn form_prompt_spec(
    name: String,
    title: String,
    description: String,
    arguments: Vec<McpPromptArgument>,
    descriptor: McpFormDescriptor,
    kind: McpFormPromptKind,
) -> Result<McpFormPromptSpec, McpToolError> {
    let definition = component_shape_mcp::prompt_definition(
        name.clone(),
        Some(title),
        Some(description),
        Some(arguments),
    )?;
    Ok(McpFormPromptSpec {
        name,
        definition,
        descriptor,
        kind,
    })
}

fn optional_prompt_argument(name: &'static str, description: &'static str) -> McpPromptArgument {
    McpPromptArgument::new(name)
        .with_description(description)
        .with_required(false)
}

fn register_form_prompt_specs(
    server: &mut McpServer,
    specs: Vec<McpFormPromptSpec>,
) -> Result<(), McpToolError> {
    for spec in specs {
        let descriptor = spec.descriptor;
        let kind = spec.kind;
        server.add_prompt(spec.definition, move |arguments| {
            form_prompt_result(descriptor, kind, arguments)
        })?;
    }
    Ok(())
}

fn ensure_prompt_specs_available(
    server: &McpServer,
    specs: &[McpFormPromptSpec],
) -> Result<(), McpToolError> {
    for spec in specs {
        if server.contains_prompt(&spec.name) {
            return Err(McpToolError::duplicate_prompt(spec.name.clone()));
        }
    }
    Ok(())
}

fn form_prompt_result(
    descriptor: McpFormDescriptor,
    kind: McpFormPromptKind,
    arguments: Option<Map<String, Value>>,
) -> McpPromptResult {
    let description = match kind {
        McpFormPromptKind::Fill => format!("Fill {}.", descriptor.title()),
        McpFormPromptKind::Repair => format!("Repair {}.", descriptor.title()),
        McpFormPromptKind::Submit => format!("Submit {}.", descriptor.title()),
    };
    component_shape_mcp::text_prompt_result(
        Some(description),
        form_prompt_text(descriptor, kind, arguments),
    )
}

fn form_prompt_text(
    descriptor: McpFormDescriptor,
    kind: McpFormPromptKind,
    arguments: Option<Map<String, Value>>,
) -> String {
    let resources = form_resource_uris(descriptor);
    let names = editor_tool_names(descriptor);
    let mut text = match kind {
        McpFormPromptKind::Fill => format!(
            "Fill the gpui-form `{form_name}` for MCP tool `{tool_name}`.\n\
             Read `{descriptor_uri}` for field labels, descriptions, examples, validation rules, and per-field schemas.\n\
             Read `{schema_uri}` for the direct-submit JSON Schema and `{examples_uri}` for example values.\n\
             Return a JSON object that can be used as `arguments` for `{tool_name}`.",
            form_name = descriptor.form_name(),
            tool_name = descriptor.tool_name(),
            descriptor_uri = resources.descriptor,
            schema_uri = resources.schema,
            examples_uri = resources.examples,
        ),
        McpFormPromptKind::Repair => format!(
            "Repair invalid values for gpui-form `{form_name}` and MCP tool `{tool_name}`.\n\
             Use `{descriptor_uri}` for field metadata and validation rules, and `{schema_uri}` for the expected argument shape.\n\
             If editing an open session, return a minimal patch with `values` and optional `clear` entries for `{patch_tool}`.\n\
             If submitting directly, return a corrected `arguments` object for `{tool_name}`.",
            form_name = descriptor.form_name(),
            tool_name = descriptor.tool_name(),
            descriptor_uri = resources.descriptor,
            schema_uri = resources.schema,
            patch_tool = names.patch,
        ),
        McpFormPromptKind::Submit => format!(
            "Submit gpui-form `{form_name}` through MCP.\n\
             Use direct tool `{tool_name}` when all required arguments are ready.\n\
             When iterative repair is useful and editor tools are registered, use `{open_tool}`, `{patch_tool}`, `{validate_tool}`, and `{submit_tool}`.\n\
             Use `{descriptor_uri}` and `{schema_uri}` to inspect field metadata and the direct-submit schema before calling tools.",
            form_name = descriptor.form_name(),
            tool_name = descriptor.tool_name(),
            open_tool = names.open,
            patch_tool = names.patch,
            validate_tool = names.validate,
            submit_tool = names.submit,
            descriptor_uri = resources.descriptor,
            schema_uri = resources.schema,
        ),
    };

    if let Some(arguments) = arguments
        && !arguments.is_empty()
    {
        text.push_str("\n\nCaller-provided context:\n");
        text.push_str(
            &serde_json::to_string_pretty(&Value::Object(arguments)).unwrap_or_else(|error| {
                format!("{{\"error\":\"failed to render prompt arguments: {error}\"}}")
            }),
        );
    }

    text
}

/// Register prompt templates for every inventory-discovered generated MCP form.
///
/// This is opt-in. It can be chained with `McpServer::builder(...).register(...)`
/// after `gpui_form::mcp::register`, or called directly on a mutable server.
pub fn register_prompt_templates(server: &mut McpServer) -> Result<(), McpToolError> {
    let options = McpFormRegistrationOptions::all();
    let resource_specs = form_resource_specs_for_options(options)?;
    let prompt_specs = form_prompt_specs_for_options(options)?;
    register_form_resource_specs_if_missing(server, resource_specs)?;
    ensure_prompt_specs_available(server, &prompt_specs)?;
    register_form_prompt_specs(server, prompt_specs)
}

/// Register prompt templates for every context-backed form matching `Context`.
///
/// Use this with [`register_context_submitters`] when context-backed forms
/// should also expose reusable fill, repair, and submit prompt templates.
pub fn register_context_submitter_prompt_templates<Context>(
    server: &mut McpServer,
) -> Result<(), McpToolError>
where
    Context: Clone + Send + Sync + 'static,
{
    let resource_specs = context_submit_resource_specs::<Context>()?;
    let prompt_specs = context_submit_prompt_specs::<Context>()?;
    register_form_resource_specs_if_missing(server, resource_specs)?;
    ensure_prompt_specs_available(server, &prompt_specs)?;
    register_form_prompt_specs(server, prompt_specs)
}

fn form_prompt_specs_for_options(
    options: McpFormRegistrationOptions,
) -> Result<Vec<McpFormPromptSpec>, McpToolError> {
    if !options.registers_tools() {
        return Ok(Vec::new());
    }

    let mut seen_tool_names = BTreeSet::new();
    let mut specs = Vec::new();
    if options.submit_handlers {
        for registration in registry::submit_handler_registrations() {
            push_descriptor_prompt_specs(
                &mut seen_tool_names,
                &mut specs,
                registration.descriptor(),
            )?;
        }
    }
    if options.editors {
        for registration in registry::editor_registrations() {
            push_descriptor_prompt_specs(
                &mut seen_tool_names,
                &mut specs,
                registration.descriptor(),
            )?;
        }
    }
    Ok(specs)
}

fn context_submit_prompt_specs<Context>() -> Result<Vec<McpFormPromptSpec>, McpToolError>
where
    Context: Clone + Send + Sync + 'static,
{
    let mut seen_tool_names = BTreeSet::new();
    let mut specs = Vec::new();
    let context_type_id = TypeId::of::<Context>();
    for registration in registry::context_submit_registrations() {
        if registration.context_type_id() == context_type_id {
            push_descriptor_prompt_specs(
                &mut seen_tool_names,
                &mut specs,
                registration.descriptor(),
            )?;
        }
    }
    Ok(specs)
}

fn push_descriptor_prompt_specs(
    seen_tool_names: &mut BTreeSet<String>,
    specs: &mut Vec<McpFormPromptSpec>,
    descriptor: McpFormDescriptor,
) -> Result<(), McpToolError> {
    if !seen_tool_names.insert(descriptor.tool_name()) {
        return Ok(());
    }
    specs.extend(form_prompt_specs(descriptor)?);
    Ok(())
}

pub trait McpForm: Sized + 'static {
    type ValueHolder: 'static;

    fn descriptor() -> McpFormDescriptor;

    fn decode_arguments(call: McpToolCall) -> Result<Self::ValueHolder, McpToolError>;

    fn validate_holder(holder: &Self::ValueHolder) -> Result<(), McpToolError>;

    /// Validate a holder and return structured validation issues.
    ///
    /// Manual implementations can rely on the default adapter from
    /// [`McpForm::validate_holder`]. Generated Koruma-backed forms override
    /// this to return field and validator metadata.
    fn validate_holder_issues(holder: &Self::ValueHolder) -> Result<(), Vec<McpValidationIssue>> {
        Self::validate_holder(holder)
            .map_err(|error| vec![McpValidationIssue::form(error.to_string())])
    }
}

/// Marker contract for generated forms with structured validation issue support.
pub trait McpFormValidation: McpForm {
    /// Validate a holder and return structured validation issues.
    fn validate_holder_issues(holder: &Self::ValueHolder) -> Result<(), Vec<McpValidationIssue>>;
}

/// Contract for generated forms that can be edited over MCP.
///
/// The MCP editor tools use the same generated field decoders as submit tools,
/// so component-backed fields keep their shape-owned input metadata and
/// value-storage policy.
pub trait McpEditableForm: McpForm
where
    Self::ValueHolder: Clone + Default,
{
    fn decode_field(
        holder: &mut Self::ValueHolder,
        field: &str,
        value: McpAny,
    ) -> Result<(), McpToolError>;
}

/// Marker trait for `derive(GpuiForm)`-generated value holders.
///
/// The generated `*FormValueHolder` type for a form implements this trait with
/// `Form` set to the model type used by the MCP tool registration APIs.
pub trait McpFormValueHolder: Sized {
    type Form: McpForm;
}

pub trait McpFormModel: McpForm {
    fn holder_into_model(holder: Self::ValueHolder) -> Result<Self, McpToolError>;
}

/// Typed MCP submit input for a generated form.
///
/// Registration uses this wrapper to keep the descriptor schema, generated
/// holder decoding, and handler input type paired at the shared MCP server
/// boundary.
pub struct McpFormInput<Form>
where
    Form: McpForm,
{
    holder: Form::ValueHolder,
}

impl<Form> McpFormInput<Form>
where
    Form: McpForm,
{
    pub fn tool_definition() -> Result<McpTypedTool<Self>, McpToolError> {
        let descriptor = Form::descriptor();
        descriptor.tool_metadata().validate()?;
        let mut tool = component_shape_mcp::tool_definition_for_input_with_annotations::<Self>(
            descriptor.tool_name(),
            Some(descriptor.title()),
            Some(descriptor.description()),
            None,
            None,
        )?;
        descriptor.apply_tool_metadata(tool.definition_mut());
        component_shape_mcp::validate_tool_definition(tool.definition())?;
        Ok(tool)
    }

    pub fn tool_definition_for_response<Response>() -> Result<McpTypedTool<Self>, McpToolError>
    where
        Response: McpJsonSchema,
    {
        let descriptor = Form::descriptor();
        descriptor.tool_metadata().validate()?;
        let mut tool = component_shape_mcp::tool_definition_for_input_with_annotations::<Self>(
            descriptor.tool_name(),
            Some(descriptor.title()),
            Some(descriptor.description()),
            Some(descriptor.output_schema::<Response>()),
            None,
        )?;
        descriptor.apply_tool_metadata(tool.definition_mut());
        component_shape_mcp::validate_tool_definition(tool.definition())?;
        Ok(tool)
    }

    pub fn into_value_holder(self) -> Form::ValueHolder {
        self.holder
    }
}

impl<Form> McpToolInput for McpFormInput<Form>
where
    Form: McpForm,
{
    fn input_schema() -> McpSchema {
        Form::descriptor().input_schema()
    }

    fn from_tool_call(call: McpToolCall) -> Result<Self, McpToolError> {
        Ok(Self {
            holder: decode_valid_holder::<Form>(call)?,
        })
    }
}

/// Handler-argument contract used by `#[gpui_form::mcp_submit]`.
///
/// `#[derive(GpuiForm)]` implements this trait for the source model when the
/// generated value holder can be converted back into the model, and for the
/// generated value-holder type itself. This lets the submit attribute route
/// model and holder handlers through Rust trait resolution instead of naming
/// conventions.
pub trait McpSubmitArgument: Sized + 'static {
    type Form: McpForm;

    fn register_sync<Handler, Response, Error>(
        server: &mut McpServer,
        handler: Handler,
    ) -> Result<(), McpToolError>
    where
        Handler: Fn(Self) -> Result<Response, Error> + Send + Sync + 'static,
        Response: Serialize + McpJsonSchema,
        Error: fmt::Display;

    fn register_async<Handler, Fut, Response, Error>(
        server: &mut McpServer,
        handler: Handler,
    ) -> Result<(), McpToolError>
    where
        Handler: Fn(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, Error>> + Send + 'static,
        Response: Serialize + McpJsonSchema,
        Error: fmt::Display;

    fn register_sync_with_editor<Handler, Response, Error>(
        server: &mut McpServer,
        handler: Handler,
    ) -> Result<(), McpToolError>
    where
        Self::Form: McpEditableForm,
        <Self::Form as McpForm>::ValueHolder: Clone + Default + Send + 'static,
        Handler: Fn(Self) -> Result<Response, Error> + Send + Sync + 'static,
        Response: Serialize + McpJsonSchema,
        Error: fmt::Display;

    fn register_async_with_editor<Handler, Fut, Response, Error>(
        server: &mut McpServer,
        handler: Handler,
    ) -> Result<(), McpToolError>
    where
        Self::Form: McpEditableForm,
        <Self::Form as McpForm>::ValueHolder: Clone + Default + Send + 'static,
        Handler: Fn(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, Error>> + Send + 'static,
        Response: Serialize + McpJsonSchema,
        Error: fmt::Display;

    fn tool_definitions_with_editor<Response>() -> Result<Vec<ToolDefinition>, McpToolError>
    where
        Self::Form: McpEditableForm,
        <Self::Form as McpForm>::ValueHolder: Clone + Default,
        Response: McpJsonSchema,
    {
        let descriptor = Self::Form::descriptor();
        let mut definitions = vec![submit_tool_definition_for_response::<Self::Form, Response>()?];
        definitions.extend(editor_tool_definitions::<Self::Form>()?);
        definitions.push(session_submit_tool_definition::<Response>(descriptor)?.into_definition());
        Ok(definitions)
    }
}

/// Context-aware submit contract for a generated MCP form.
///
/// Implement this trait on a `#[derive(GpuiForm)]` source struct that also
/// declares `#[gpui_form(mcp(context(...), response(...)))]`. The derive emits
/// inventory registration for the struct, and
/// [`register_context_submitters_with_editor_options`] wires every matching
/// submitter into a server using the shared runtime context value.
pub trait McpContextSubmit<Context>: McpSubmitArgument
where
    Context: Clone + Send + Sync + 'static,
{
    type Response: Serialize + McpJsonSchema;
    type Error: fmt::Display;

    fn submit_with_context(
        self,
        context: Context,
    ) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send + 'static;
}

pub mod registry {
    use std::any::TypeId;

    use super::{
        McpFormDescriptor, McpFormEditorOptions, McpServer, McpSubmitContext, McpToolError,
        ToolDefinition,
    };
    pub use inventory;

    inventory::collect!(McpEditorRegistration);
    inventory::collect!(McpSubmitRegistration);
    inventory::collect!(McpSubmitHandlerRegistration);
    inventory::collect!(McpContextSubmitRegistration);

    pub struct McpSubmitRegistration {
        descriptor: fn() -> McpFormDescriptor,
    }

    impl McpSubmitRegistration {
        pub const fn new(descriptor: fn() -> McpFormDescriptor) -> Self {
            Self { descriptor }
        }

        pub fn descriptor(&self) -> McpFormDescriptor {
            (self.descriptor)()
        }
    }

    pub fn submit_registrations() -> impl Iterator<Item = &'static McpSubmitRegistration> {
        inventory::iter::<McpSubmitRegistration>.into_iter()
    }

    pub struct McpEditorRegistration {
        descriptor: fn() -> McpFormDescriptor,
        register: fn(&mut McpServer) -> Result<(), McpToolError>,
        tool_definitions: fn() -> Result<Vec<ToolDefinition>, McpToolError>,
    }

    impl McpEditorRegistration {
        pub const fn new(
            descriptor: fn() -> McpFormDescriptor,
            register: fn(&mut McpServer) -> Result<(), McpToolError>,
            tool_definitions: fn() -> Result<Vec<ToolDefinition>, McpToolError>,
        ) -> Self {
            Self {
                descriptor,
                register,
                tool_definitions,
            }
        }

        pub fn descriptor(&self) -> McpFormDescriptor {
            (self.descriptor)()
        }

        pub fn register(&self, server: &mut McpServer) -> Result<(), McpToolError> {
            (self.register)(server)
        }

        pub fn tool_definitions(&self) -> Result<Vec<ToolDefinition>, McpToolError> {
            (self.tool_definitions)()
        }
    }

    pub fn editor_registrations() -> impl Iterator<Item = &'static McpEditorRegistration> {
        inventory::iter::<McpEditorRegistration>.into_iter()
    }

    pub struct McpSubmitHandlerRegistration {
        descriptor: fn() -> McpFormDescriptor,
        register: fn(&mut McpServer) -> Result<(), McpToolError>,
        register_submit: fn(&mut McpServer) -> Result<(), McpToolError>,
        tool_definitions: fn() -> Result<Vec<ToolDefinition>, McpToolError>,
        submit_tool_definitions: fn() -> Result<Vec<ToolDefinition>, McpToolError>,
    }

    impl McpSubmitHandlerRegistration {
        pub const fn new(
            descriptor: fn() -> McpFormDescriptor,
            register: fn(&mut McpServer) -> Result<(), McpToolError>,
            tool_definitions: fn() -> Result<Vec<ToolDefinition>, McpToolError>,
        ) -> Self {
            Self {
                descriptor,
                register,
                register_submit: register,
                tool_definitions,
                submit_tool_definitions: tool_definitions,
            }
        }

        pub const fn new_with_submit_only(
            descriptor: fn() -> McpFormDescriptor,
            register: fn(&mut McpServer) -> Result<(), McpToolError>,
            register_submit: fn(&mut McpServer) -> Result<(), McpToolError>,
            tool_definitions: fn() -> Result<Vec<ToolDefinition>, McpToolError>,
            submit_tool_definitions: fn() -> Result<Vec<ToolDefinition>, McpToolError>,
        ) -> Self {
            Self {
                descriptor,
                register,
                register_submit,
                tool_definitions,
                submit_tool_definitions,
            }
        }

        pub fn descriptor(&self) -> McpFormDescriptor {
            (self.descriptor)()
        }

        pub fn register(&self, server: &mut McpServer) -> Result<(), McpToolError> {
            (self.register)(server)
        }

        pub fn register_submit(&self, server: &mut McpServer) -> Result<(), McpToolError> {
            (self.register_submit)(server)
        }

        pub fn tool_definitions(&self) -> Result<Vec<ToolDefinition>, McpToolError> {
            (self.tool_definitions)()
        }

        pub fn submit_tool_definitions(&self) -> Result<Vec<ToolDefinition>, McpToolError> {
            (self.submit_tool_definitions)()
        }
    }

    pub fn submit_handler_registrations()
    -> impl Iterator<Item = &'static McpSubmitHandlerRegistration> {
        inventory::iter::<McpSubmitHandlerRegistration>.into_iter()
    }

    pub struct McpContextSubmitRegistration {
        descriptor: fn() -> McpFormDescriptor,
        context_type_id: fn() -> TypeId,
        register:
            fn(&mut McpServer, McpSubmitContext, McpFormEditorOptions) -> Result<(), McpToolError>,
        tool_definitions: fn() -> Result<Vec<ToolDefinition>, McpToolError>,
    }

    impl McpContextSubmitRegistration {
        pub const fn new(
            descriptor: fn() -> McpFormDescriptor,
            context_type_id: fn() -> TypeId,
            register: fn(
                &mut McpServer,
                McpSubmitContext,
                McpFormEditorOptions,
            ) -> Result<(), McpToolError>,
            tool_definitions: fn() -> Result<Vec<ToolDefinition>, McpToolError>,
        ) -> Self {
            Self {
                descriptor,
                context_type_id,
                register,
                tool_definitions,
            }
        }

        pub fn descriptor(&self) -> McpFormDescriptor {
            (self.descriptor)()
        }

        pub fn context_type_id(&self) -> TypeId {
            (self.context_type_id)()
        }

        pub fn register(
            &self,
            server: &mut McpServer,
            context: McpSubmitContext,
            options: McpFormEditorOptions,
        ) -> Result<(), McpToolError> {
            (self.register)(server, context, options)
        }

        pub fn tool_definitions(&self) -> Result<Vec<ToolDefinition>, McpToolError> {
            (self.tool_definitions)()
        }
    }

    pub fn context_submit_registrations()
    -> impl Iterator<Item = &'static McpContextSubmitRegistration> {
        inventory::iter::<McpContextSubmitRegistration>.into_iter()
    }
}

pub const DEFAULT_SERVER_NAME: &str = "gpui-form-mcp";

/// Start registering an MCP submit tool for a generated form.
pub fn form<Form>(server: &mut McpServer) -> FormTool<'_, Form>
where
    Form: McpForm,
{
    FormTool::new(server)
}

/// Build the submit tool definition for a generated MCP form.
pub fn submit_tool_definition<Form>() -> Result<ToolDefinition, McpToolError>
where
    Form: McpForm,
{
    Form::descriptor().tool_definition()
}

/// Build the submit tool definition for a generated MCP form and response.
pub fn submit_tool_definition_for_response<Form, Response>() -> Result<ToolDefinition, McpToolError>
where
    Form: McpForm,
    Response: McpJsonSchema,
{
    Ok(McpFormInput::<Form>::tool_definition_for_response::<Response>()?.into_definition())
}

/// Register editable holder-session tools for a generated MCP form.
pub fn register_editor<Form>(server: &mut McpServer) -> Result<(), McpToolError>
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default + Send + 'static,
{
    form::<Form>(server).editor()
}

/// Register editable holder-session tools with explicit runtime session policy.
pub fn register_editor_with_options<Form>(
    server: &mut McpServer,
    options: McpFormEditorOptions,
) -> Result<(), McpToolError>
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default + Send + 'static,
{
    form::<Form>(server).editor_with_options(options)
}

/// Register every inventory-discovered context submitter for `Context`.
///
/// Forms opt into this path with `#[gpui_form(mcp(context(...), response(...)))]`
/// and an implementation of [`McpContextSubmit<Context>`]. Each matching form
/// gets its direct submit tool plus edit-session tools with an edit-submit
/// endpoint that uses the same context.
pub fn register_context_submitters<Context>(
    server: &mut McpServer,
    context: Context,
) -> Result<(), McpToolError>
where
    Context: Clone + Send + Sync + 'static,
{
    register_context_submitters_with_editor_options(
        server,
        context,
        McpFormEditorOptions::default(),
    )
}

/// Register every inventory-discovered context submitter for `Context` with an
/// explicit edit-session policy.
pub fn register_context_submitters_with_editor_options<Context>(
    server: &mut McpServer,
    context: Context,
    options: McpFormEditorOptions,
) -> Result<(), McpToolError>
where
    Context: Clone + Send + Sync + 'static,
{
    register_context_submitters_report(server, context, options, false).map(|_| ())
}

/// Register every inventory-discovered context submitter for `Context`,
/// failing when no registration matches the context type.
pub fn register_context_submitters_strict<Context>(
    server: &mut McpServer,
    context: Context,
) -> Result<McpFormRegistrationReport, McpToolError>
where
    Context: Clone + Send + Sync + 'static,
{
    register_context_submitters_strict_with_editor_options(
        server,
        context,
        McpFormEditorOptions::default(),
    )
}

/// Register every inventory-discovered context submitter for `Context` with an
/// explicit edit-session policy, failing when no registration matches.
pub fn register_context_submitters_strict_with_editor_options<Context>(
    server: &mut McpServer,
    context: Context,
    options: McpFormEditorOptions,
) -> Result<McpFormRegistrationReport, McpToolError>
where
    Context: Clone + Send + Sync + 'static,
{
    register_context_submitters_report(server, context, options, true)
}

fn register_context_submitters_report<Context>(
    server: &mut McpServer,
    context: Context,
    options: McpFormEditorOptions,
    strict: bool,
) -> Result<McpFormRegistrationReport, McpToolError>
where
    Context: Clone + Send + Sync + 'static,
{
    let definitions = context_submit_tool_definitions::<Context>()?;
    let context_registration_count = context_submit_registration_count::<Context>();
    if strict && context_registration_count == 0 {
        return Err(McpToolError::validation(format!(
            "no MCP context submitters registered for context `{}`",
            std::any::type_name::<Context>()
        )));
    }
    let resource_specs = context_submit_resource_specs::<Context>()?;
    ensure_tool_definitions_available(server, &definitions)?;
    register_form_resource_specs_if_missing(server, resource_specs)?;

    let context: McpSubmitContext = Arc::new(context);
    let context_type_id = TypeId::of::<Context>();
    for registration in registry::context_submit_registrations() {
        if registration.context_type_id() == context_type_id {
            registration.register(server, Arc::clone(&context), options)?;
        }
    }

    let mut report = McpFormRegistrationReport::from_tool_definitions(definitions);
    report.context_registration_count = context_registration_count;
    Ok(report)
}

/// Return tool definitions for every inventory-discovered context submitter
/// matching `Context`, failing on duplicate names.
pub fn context_submit_tool_definitions<Context>() -> Result<Vec<ToolDefinition>, McpToolError>
where
    Context: Clone + Send + Sync + 'static,
{
    let mut seen = BTreeSet::new();
    let mut tools = Vec::new();
    let context_type_id = TypeId::of::<Context>();
    for registration in registry::context_submit_registrations() {
        if registration.context_type_id() != context_type_id {
            continue;
        }

        for definition in registration.tool_definitions()? {
            push_tool_definition(&mut seen, &mut tools, definition)?;
        }
    }
    Ok(tools)
}

fn context_submit_registration_count<Context>() -> usize
where
    Context: Clone + Send + Sync + 'static,
{
    let context_type_id = TypeId::of::<Context>();
    registry::context_submit_registrations()
        .filter(|registration| registration.context_type_id() == context_type_id)
        .count()
}

pub struct FormTool<'server, Form> {
    server: &'server mut McpServer,
    _form: PhantomData<fn() -> Form>,
}

impl<'server, Form> FormTool<'server, Form>
where
    Form: McpForm,
{
    fn new(server: &'server mut McpServer) -> Self {
        Self {
            server,
            _form: PhantomData,
        }
    }

    pub fn holder<Handler, Response, Error>(self, handler: Handler) -> Result<(), McpToolError>
    where
        Handler: Fn(Form::ValueHolder) -> Result<Response, Error> + Send + Sync + 'static,
        Response: Serialize + McpJsonSchema,
        Error: fmt::Display,
    {
        insert_executor::<Form, Response, _>(self.server, move |input| {
            component_shape_mcp::serialize_handler_response(handler(input.into_value_holder()))
        })
    }
    pub fn holder_async<Handler, Fut, Response, Error>(
        self,
        handler: Handler,
    ) -> Result<(), McpToolError>
    where
        Handler: Fn(Form::ValueHolder) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, Error>> + Send + 'static,
        Response: Serialize + McpJsonSchema,
        Error: fmt::Display,
    {
        let handler = Arc::new(handler);
        insert_executor_async::<Form, Response, _>(self.server, move |input| {
            let handler = Arc::clone(&handler);
            let future = handler(input.into_value_holder());
            Box::pin(async move { component_shape_mcp::serialize_handler_response(future.await) })
        })
    }
}

impl<'server, Form> FormTool<'server, Form>
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default + Send + 'static,
{
    /// Register a value-holder submit tool and editable holder-session tools
    /// for the same form.
    pub fn holder_with_editor<Handler, Response, Error>(
        self,
        handler: Handler,
    ) -> Result<(), McpToolError>
    where
        Handler: Fn(Form::ValueHolder) -> Result<Response, Error> + Send + Sync + 'static,
        Response: Serialize + McpJsonSchema,
        Error: fmt::Display,
    {
        self.holder_with_editor_options(McpFormEditorOptions::default(), handler)
    }

    /// Register a value-holder submit tool and editable holder-session tools
    /// with explicit runtime session policy.
    pub fn holder_with_editor_options<Handler, Response, Error>(
        self,
        options: McpFormEditorOptions,
        handler: Handler,
    ) -> Result<(), McpToolError>
    where
        Handler: Fn(Form::ValueHolder) -> Result<Response, Error> + Send + Sync + 'static,
        Response: Serialize + McpJsonSchema,
        Error: fmt::Display,
    {
        let server = self.server;
        ensure_form_tool_and_editor_submit_tools_available::<Form, Response>(server)?;
        let handler = Arc::new(handler);
        insert_executor::<Form, Response, _>(server, {
            let handler = Arc::clone(&handler);
            move |input| {
                component_shape_mcp::serialize_handler_response(handler(input.into_value_holder()))
            }
        })?;
        insert_editor_tools_with_session_submit::<Form, Response, _>(
            server,
            Form::descriptor(),
            {
                let handler = Arc::clone(&handler);
                move |holder| component_shape_mcp::serialize_handler_response(handler(holder))
            },
            options,
        )
    }

    /// Register an async value-holder submit tool and editable holder-session
    /// tools for the same form.
    pub fn holder_async_with_editor<Handler, Fut, Response, Error>(
        self,
        handler: Handler,
    ) -> Result<(), McpToolError>
    where
        Handler: Fn(Form::ValueHolder) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, Error>> + Send + 'static,
        Response: Serialize + McpJsonSchema,
        Error: fmt::Display,
    {
        self.holder_async_with_editor_options(McpFormEditorOptions::default(), handler)
    }

    /// Register an async value-holder submit tool and editable holder-session
    /// tools with explicit runtime session policy.
    pub fn holder_async_with_editor_options<Handler, Fut, Response, Error>(
        self,
        options: McpFormEditorOptions,
        handler: Handler,
    ) -> Result<(), McpToolError>
    where
        Handler: Fn(Form::ValueHolder) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, Error>> + Send + 'static,
        Response: Serialize + McpJsonSchema,
        Error: fmt::Display,
    {
        let handler = Arc::new(handler);
        let server = self.server;
        ensure_form_tool_and_editor_submit_tools_available::<Form, Response>(server)?;
        insert_executor_async::<Form, Response, _>(server, {
            let handler = Arc::clone(&handler);
            move |input| {
                let handler = Arc::clone(&handler);
                let future = handler(input.into_value_holder());
                Box::pin(
                    async move { component_shape_mcp::serialize_handler_response(future.await) },
                )
            }
        })?;
        insert_editor_tools_with_session_submit_async::<Form, Response, _>(
            server,
            Form::descriptor(),
            {
                let handler = Arc::clone(&handler);
                move |holder| {
                    let handler = Arc::clone(&handler);
                    Box::pin(async move {
                        component_shape_mcp::serialize_handler_response(handler(holder).await)
                    }) as ToolFuture
                }
            },
            options,
        )
    }

    /// Register stateful MCP tools that let an agent open, inspect, patch,
    /// validate, and close editable form-holder sessions.
    pub fn editor(self) -> Result<(), McpToolError> {
        self.editor_with_options(McpFormEditorOptions::default())
    }

    /// Register stateful MCP tools with explicit runtime session policy.
    pub fn editor_with_options(self, options: McpFormEditorOptions) -> Result<(), McpToolError> {
        insert_editor_tools::<Form>(self.server, Form::descriptor(), options)
    }
}

impl<'server, Form> FormTool<'server, Form>
where
    Form: McpFormModel,
{
    pub fn model<Handler, Response, Error>(self, handler: Handler) -> Result<(), McpToolError>
    where
        Handler: Fn(Form) -> Result<Response, Error> + Send + Sync + 'static,
        Response: Serialize + McpJsonSchema,
        Error: fmt::Display,
    {
        insert_executor::<Form, Response, _>(self.server, move |input| {
            let model = match Form::holder_into_model(input.into_value_holder()) {
                Ok(model) => model,
                Err(error) => return component_shape_mcp::tool_error_result_for(error),
            };

            component_shape_mcp::serialize_handler_response(handler(model))
        })
    }

    /// Register a model submit tool and editable holder-session tools for the
    /// same form.
    pub fn model_with_editor<Handler, Response, Error>(
        self,
        handler: Handler,
    ) -> Result<(), McpToolError>
    where
        Form: McpEditableForm,
        Form::ValueHolder: Clone + Default + Send + 'static,
        Handler: Fn(Form) -> Result<Response, Error> + Send + Sync + 'static,
        Response: Serialize + McpJsonSchema,
        Error: fmt::Display,
    {
        self.model_with_editor_options(McpFormEditorOptions::default(), handler)
    }

    /// Register a model submit tool and editable holder-session tools with
    /// explicit runtime session policy.
    pub fn model_with_editor_options<Handler, Response, Error>(
        self,
        options: McpFormEditorOptions,
        handler: Handler,
    ) -> Result<(), McpToolError>
    where
        Form: McpEditableForm,
        Form::ValueHolder: Clone + Default + Send + 'static,
        Handler: Fn(Form) -> Result<Response, Error> + Send + Sync + 'static,
        Response: Serialize + McpJsonSchema,
        Error: fmt::Display,
    {
        let server = self.server;
        ensure_form_tool_and_editor_submit_tools_available::<Form, Response>(server)?;
        let handler = Arc::new(handler);
        insert_executor::<Form, Response, _>(server, {
            let handler = Arc::clone(&handler);
            move |input| {
                let model = match Form::holder_into_model(input.into_value_holder()) {
                    Ok(model) => model,
                    Err(error) => return component_shape_mcp::tool_error_result_for(error),
                };

                component_shape_mcp::serialize_handler_response(handler(model))
            }
        })?;
        insert_editor_tools_with_session_submit::<Form, Response, _>(
            server,
            Form::descriptor(),
            {
                let handler = Arc::clone(&handler);
                move |holder| {
                    let model = match Form::holder_into_model(holder) {
                        Ok(model) => model,
                        Err(error) => return component_shape_mcp::tool_error_result_for(error),
                    };

                    component_shape_mcp::serialize_handler_response(handler(model))
                }
            },
            options,
        )
    }

    pub fn model_async<Handler, Fut, Response, Error>(
        self,
        handler: Handler,
    ) -> Result<(), McpToolError>
    where
        Handler: Fn(Form) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, Error>> + Send + 'static,
        Response: Serialize + McpJsonSchema,
        Error: fmt::Display,
    {
        let handler = Arc::new(handler);
        insert_executor_async::<Form, Response, _>(self.server, move |input| {
            let handler = Arc::clone(&handler);
            match Form::holder_into_model(input.into_value_holder()) {
                Ok(model) => {
                    let future = handler(model);
                    Box::pin(async move {
                        component_shape_mcp::serialize_handler_response(future.await)
                    })
                },
                Err(error) => Box::pin(std::future::ready(
                    component_shape_mcp::tool_error_result_for(error),
                )),
            }
        })
    }

    /// Register an async model submit tool and editable holder-session tools
    /// for the same form.
    pub fn model_async_with_editor<Handler, Fut, Response, Error>(
        self,
        handler: Handler,
    ) -> Result<(), McpToolError>
    where
        Form: McpEditableForm,
        Form::ValueHolder: Clone + Default + Send + 'static,
        Handler: Fn(Form) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, Error>> + Send + 'static,
        Response: Serialize + McpJsonSchema,
        Error: fmt::Display,
    {
        self.model_async_with_editor_options(McpFormEditorOptions::default(), handler)
    }

    /// Register an async model submit tool and editable holder-session tools
    /// with explicit runtime session policy.
    pub fn model_async_with_editor_options<Handler, Fut, Response, Error>(
        self,
        options: McpFormEditorOptions,
        handler: Handler,
    ) -> Result<(), McpToolError>
    where
        Form: McpEditableForm,
        Form::ValueHolder: Clone + Default + Send + 'static,
        Handler: Fn(Form) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, Error>> + Send + 'static,
        Response: Serialize + McpJsonSchema,
        Error: fmt::Display,
    {
        let handler = Arc::new(handler);
        let server = self.server;
        ensure_form_tool_and_editor_submit_tools_available::<Form, Response>(server)?;
        insert_executor_async::<Form, Response, _>(server, {
            let handler = Arc::clone(&handler);
            move |input| {
                let handler = Arc::clone(&handler);
                match Form::holder_into_model(input.into_value_holder()) {
                    Ok(model) => {
                        let future = handler(model);
                        Box::pin(async move {
                            component_shape_mcp::serialize_handler_response(future.await)
                        })
                    },
                    Err(error) => Box::pin(std::future::ready(
                        component_shape_mcp::tool_error_result_for(error),
                    )),
                }
            }
        })?;
        insert_editor_tools_with_session_submit_async::<Form, Response, _>(
            server,
            Form::descriptor(),
            {
                let handler = Arc::clone(&handler);
                move |holder| {
                    let handler = Arc::clone(&handler);
                    match Form::holder_into_model(holder) {
                        Ok(model) => {
                            let future = handler(model);
                            Box::pin(async move {
                                component_shape_mcp::serialize_handler_response(future.await)
                            }) as ToolFuture
                        },
                        Err(error) => Box::pin(std::future::ready(
                            component_shape_mcp::tool_error_result_for(error),
                        )) as ToolFuture,
                    }
                }
            },
            options,
        )
    }
}

fn insert_executor<Form, Response, Call>(
    server: &mut McpServer,
    call: Call,
) -> Result<(), McpToolError>
where
    Form: McpForm,
    Response: McpJsonSchema,
    Call: Fn(McpFormInput<Form>) -> ToolCallResult + Send + Sync + 'static,
{
    let definition = McpFormInput::<Form>::tool_definition_for_response::<Response>()?;
    let resource_specs = form_resource_specs(Form::descriptor())?;
    ensure_tool_definitions_available(server, &[definition.definition().clone()])?;
    register_form_resource_specs_if_missing(server, resource_specs)?;
    server.add_typed_tool(definition, call)
}

fn insert_executor_async<Form, Response, Call>(
    server: &mut McpServer,
    call: Call,
) -> Result<(), McpToolError>
where
    Form: McpForm,
    Response: McpJsonSchema,
    Call: Fn(McpFormInput<Form>) -> ToolFuture + Send + Sync + 'static,
{
    let definition = McpFormInput::<Form>::tool_definition_for_response::<Response>()?;
    let resource_specs = form_resource_specs(Form::descriptor())?;
    ensure_tool_definitions_available(server, &[definition.definition().clone()])?;
    register_form_resource_specs_if_missing(server, resource_specs)?;
    server.add_typed_tool_async(definition, call)
}

pub fn tool_name(source_module_path: &str, form_name: &str) -> String {
    component_shape_mcp::tool_name(source_module_path, form_name, "form_")
}

fn decode_valid_holder<Form>(call: McpToolCall) -> Result<Form::ValueHolder, McpToolError>
where
    Form: McpForm,
{
    Form::decode_arguments(call).and_then(|holder| {
        <Form as McpForm>::validate_holder_issues(&holder)
            .map(|()| holder)
            .map_err(validation_issues_error)
    })
}

fn ensure_form_tool_and_editor_submit_tools_available<Form, Response>(
    server: &McpServer,
) -> Result<(), McpToolError>
where
    Form: McpEditableForm,
    Response: McpJsonSchema,
    Form::ValueHolder: Clone + Default,
{
    let mut definitions =
        vec![McpFormInput::<Form>::tool_definition_for_response::<Response>()?.into_definition()];
    definitions.extend(editor_tool_definitions::<Form>()?);
    definitions
        .push(session_submit_tool_definition::<Response>(Form::descriptor())?.into_definition());
    ensure_tool_definitions_available(server, &definitions)
}

fn ensure_tool_definitions_available(
    server: &McpServer,
    definitions: &[ToolDefinition],
) -> Result<(), McpToolError> {
    for definition in definitions {
        let name = definition.name.to_string();
        if server.contains_tool(&name) {
            return Err(McpToolError::duplicate_tool(name));
        }
    }

    Ok(())
}

#[derive(Debug)]
struct EditSession<Holder> {
    holder: Holder,
    present_fields: BTreeSet<String>,
    values: Map<String, Value>,
    revision: u64,
    opened_order: u64,
    last_accessed_at: Instant,
}

#[derive(Debug)]
struct EditSessions<Holder> {
    next_id: u64,
    sessions: BTreeMap<String, EditSession<Holder>>,
    options: McpFormEditorOptions,
}

#[derive(Debug, Default)]
struct EditSessionCleanup {
    expired_session_ids: Vec<String>,
    evicted_session_ids: Vec<String>,
}

#[derive(Debug)]
struct OpenedEditSession {
    session_id: String,
    cleanup: EditSessionCleanup,
}

impl<Holder> Default for EditSessions<Holder> {
    fn default() -> Self {
        Self::new(McpFormEditorOptions::default())
    }
}

impl<Holder> EditSessions<Holder> {
    fn new(options: McpFormEditorOptions) -> Self {
        Self {
            next_id: 1,
            sessions: BTreeMap::new(),
            options,
        }
    }

    fn open(
        &mut self,
        holder: Holder,
        present_fields: BTreeSet<String>,
        values: Map<String, Value>,
    ) -> OpenedEditSession {
        let now = Instant::now();
        let expired_session_ids = self.expire_idle_sessions(now);
        let session_id = self.next_id.to_string();
        self.next_id += 1;
        self.sessions.insert(
            session_id.clone(),
            EditSession {
                holder,
                present_fields,
                values,
                revision: 0,
                opened_order: self.next_id - 1,
                last_accessed_at: now,
            },
        );
        let evicted_session_ids = self.enforce_session_limit();
        OpenedEditSession {
            session_id,
            cleanup: EditSessionCleanup {
                expired_session_ids,
                evicted_session_ids,
            },
        }
    }

    fn get(&mut self, session_id: &str) -> Result<&EditSession<Holder>, McpToolError> {
        let now = Instant::now();
        self.expire_idle_sessions(now);
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| McpToolError::invalid_field_value("session_id", session_id))?;
        session.last_accessed_at = now;
        Ok(session)
    }

    fn get_mut(&mut self, session_id: &str) -> Result<&mut EditSession<Holder>, McpToolError> {
        let now = Instant::now();
        self.expire_idle_sessions(now);
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| McpToolError::invalid_field_value("session_id", session_id))?;
        session.last_accessed_at = now;
        Ok(session)
    }

    fn close(&mut self, session_id: &str) -> Result<bool, McpToolError> {
        if self.sessions.remove(session_id).is_some() {
            Ok(true)
        } else {
            Err(McpToolError::invalid_field_value("session_id", session_id))
        }
    }

    fn close_all(&mut self) -> Vec<String> {
        let session_ids = self.sessions.keys().cloned().collect();
        self.sessions.clear();
        session_ids
    }

    fn session_limit(&self) -> Option<usize> {
        self.options.session_limit()
    }

    fn session_idle_timeout(&self) -> Option<Duration> {
        self.options.session_idle_timeout()
    }

    fn expire_idle_sessions(&mut self, now: Instant) -> Vec<String> {
        let Some(timeout) = self.options.session_idle_timeout() else {
            return Vec::new();
        };
        let expired = self
            .sessions
            .iter()
            .filter(|&(_, session)| now.duration_since(session.last_accessed_at) >= timeout)
            .map(|(session_id, _)| session_id.clone())
            .collect::<Vec<_>>();
        for session_id in &expired {
            self.sessions.remove(session_id);
        }
        expired
    }

    fn enforce_session_limit(&mut self) -> Vec<String> {
        let Some(limit) = self.options.session_limit() else {
            return Vec::new();
        };
        let excess = self.sessions.len().saturating_sub(limit);
        if excess == 0 {
            return Vec::new();
        }

        let mut session_ids = self
            .sessions
            .iter()
            .map(|(session_id, session)| (session.opened_order, session_id.clone()))
            .collect::<Vec<_>>();
        session_ids.sort_by_key(|(opened_order, _)| *opened_order);
        let evicted = session_ids
            .into_iter()
            .take(excess)
            .map(|(_, session_id)| session_id)
            .collect::<Vec<_>>();
        for session_id in &evicted {
            self.sessions.remove(session_id.as_str());
        }
        evicted
    }
}

type SharedEditSessions<Holder> = Arc<Mutex<EditSessions<Holder>>>;

fn insert_editor_tools<Form>(
    server: &mut McpServer,
    descriptor: McpFormDescriptor,
    options: McpFormEditorOptions,
) -> Result<(), McpToolError>
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default + Send + 'static,
{
    let definitions = editor_typed_tool_definitions::<Form>(descriptor)?;
    let resource_specs = form_resource_specs(descriptor)?;
    ensure_tool_definitions_available(server, &definitions.as_tool_definitions())?;
    register_form_resource_specs_if_missing(server, resource_specs)?;

    let sessions = Arc::new(Mutex::new(EditSessions::new(options)));
    insert_editor_tools_with_sessions::<Form>(server, definitions, sessions)
}

fn insert_editor_tools_with_session_submit<Form, Response, Call>(
    server: &mut McpServer,
    descriptor: McpFormDescriptor,
    call: Call,
    options: McpFormEditorOptions,
) -> Result<(), McpToolError>
where
    Form: McpEditableForm,
    Response: McpJsonSchema,
    Form::ValueHolder: Clone + Default + Send + 'static,
    Call: Fn(Form::ValueHolder) -> ToolCallResult + Send + Sync + 'static,
{
    let definitions = editor_typed_tool_definitions::<Form>(descriptor)?;
    let session_submit = session_submit_tool_definition::<Response>(descriptor)?;
    let sessions = Arc::new(Mutex::new(EditSessions::new(options)));
    insert_editor_tools_with_sessions::<Form>(server, definitions, Arc::clone(&sessions))?;
    server.add_typed_tool(session_submit, move |input| {
        submit_edit_session::<Form, _>(input, &sessions, &call)
    })
}

fn insert_editor_tools_with_session_submit_async<Form, Response, Call>(
    server: &mut McpServer,
    descriptor: McpFormDescriptor,
    call: Call,
    options: McpFormEditorOptions,
) -> Result<(), McpToolError>
where
    Form: McpEditableForm,
    Response: McpJsonSchema,
    Form::ValueHolder: Clone + Default + Send + 'static,
    Call: Fn(Form::ValueHolder) -> ToolFuture + Send + Sync + 'static,
{
    let definitions = editor_typed_tool_definitions::<Form>(descriptor)?;
    let session_submit = session_submit_tool_definition::<Response>(descriptor)?;
    let sessions = Arc::new(Mutex::new(EditSessions::new(options)));
    insert_editor_tools_with_sessions::<Form>(server, definitions, Arc::clone(&sessions))?;
    server.add_typed_tool_async(session_submit, move |input| {
        submit_edit_session_async::<Form, _>(input, &sessions, &call)
    })
}

fn insert_editor_tools_with_sessions<Form>(
    server: &mut McpServer,
    definitions: McpFormEditorToolDefinitions<Form>,
    sessions: SharedEditSessions<Form::ValueHolder>,
) -> Result<(), McpToolError>
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default + Send + 'static,
{
    {
        let sessions = Arc::clone(&sessions);
        server.add_typed_tool(definitions.open, move |input| {
            open_edit_session::<Form>(input, &sessions)
        })?;
    }

    {
        let sessions = Arc::clone(&sessions);
        server.add_typed_tool(definitions.list, move |()| {
            list_edit_sessions::<Form>(&sessions)
        })?;
    }

    {
        let sessions = Arc::clone(&sessions);
        server.add_typed_tool(definitions.read, move |input| {
            read_edit_session::<Form>(input, &sessions)
        })?;
    }

    {
        let sessions = Arc::clone(&sessions);
        server.add_typed_tool(definitions.patch, move |input| {
            patch_edit_session::<Form>(input, &sessions)
        })?;
    }

    {
        let sessions = Arc::clone(&sessions);
        server.add_typed_tool(definitions.validate, move |input| {
            validate_edit_session::<Form>(input, &sessions)
        })?;
    }

    {
        let sessions = Arc::clone(&sessions);
        server.add_typed_tool(definitions.close, move |input| {
            close_edit_session::<Form>(input, &sessions)
        })?;
    }

    server.add_typed_tool(definitions.close_all, move |()| {
        close_all_edit_sessions::<Form>(&sessions)
    })?;

    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpFormEditorToolNames {
    pub open: String,
    pub list: String,
    pub read: String,
    pub patch: String,
    pub validate: String,
    pub close: String,
    pub close_all: String,
    pub submit: String,
}

pub fn editor_tool_names(descriptor: McpFormDescriptor) -> McpFormEditorToolNames {
    let prefix = descriptor.tool_name();
    McpFormEditorToolNames {
        open: format!("{prefix}_edit_open"),
        list: format!("{prefix}_edit_list"),
        read: format!("{prefix}_edit_read"),
        patch: format!("{prefix}_edit_patch"),
        validate: format!("{prefix}_edit_validate"),
        close: format!("{prefix}_edit_close"),
        close_all: format!("{prefix}_edit_close_all"),
        submit: format!("{prefix}_edit_submit"),
    }
}

/// Build the editable holder-session tool definitions for a generated MCP form.
pub fn editor_tool_definitions<Form>() -> Result<Vec<ToolDefinition>, McpToolError>
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
{
    Ok(editor_typed_tool_definitions::<Form>(Form::descriptor())?
        .into_tool_definitions()
        .into_iter()
        .collect())
}

struct McpFormEditorToolDefinitions<Form>
where
    Form: McpForm,
{
    open: McpTypedTool<McpFormEditOpenInput<Form>>,
    list: McpTypedTool<()>,
    read: McpTypedTool<SessionIdInput>,
    patch: McpTypedTool<McpFormEditPatchInput<Form>>,
    validate: McpTypedTool<SessionIdInput>,
    close: McpTypedTool<McpFormEditCloseInput>,
    close_all: McpTypedTool<()>,
}

impl<Form> McpFormEditorToolDefinitions<Form>
where
    Form: McpForm,
{
    fn as_tool_definitions(&self) -> Vec<ToolDefinition> {
        vec![
            self.open.definition().clone(),
            self.list.definition().clone(),
            self.read.definition().clone(),
            self.patch.definition().clone(),
            self.validate.definition().clone(),
            self.close.definition().clone(),
            self.close_all.definition().clone(),
        ]
    }

    fn into_tool_definitions(self) -> Vec<ToolDefinition> {
        vec![
            self.open.into_definition(),
            self.list.into_definition(),
            self.read.into_definition(),
            self.patch.into_definition(),
            self.validate.into_definition(),
            self.close.into_definition(),
            self.close_all.into_definition(),
        ]
    }
}

fn editor_typed_tool_definitions<Form>(
    descriptor: McpFormDescriptor,
) -> Result<McpFormEditorToolDefinitions<Form>, McpToolError>
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
{
    let tool_names = editor_tool_names(descriptor);
    let snapshot_output_schema = edit_session_snapshot_output_schema(descriptor.fields());
    let list_output_schema = edit_session_list_output_schema(descriptor.fields());

    Ok(McpFormEditorToolDefinitions {
        open: editor_tool_definition::<McpFormEditOpenInput<Form>>(
            tool_names.open,
            format!("Open {} edit session", descriptor.form_name()),
            format!(
                "Create an editable {} form value-holder session.",
                descriptor.form_name()
            ),
            snapshot_output_schema.clone(),
            session_mutation_annotations(false),
        )?,
        list: editor_tool_definition::<()>(
            tool_names.list,
            format!("List {} edit sessions", descriptor.form_name()),
            format!(
                "Return active editable {} form sessions.",
                descriptor.form_name()
            ),
            list_output_schema,
            read_only_annotations(),
        )?,
        read: editor_tool_definition::<SessionIdInput>(
            tool_names.read,
            format!("Read {} edit session", descriptor.form_name()),
            format!(
                "Return the current values for an editable {} form session.",
                descriptor.form_name()
            ),
            snapshot_output_schema.clone(),
            read_only_annotations(),
        )?,
        patch: editor_tool_definition::<McpFormEditPatchInput<Form>>(
            tool_names.patch,
            format!("Patch {} fields", descriptor.form_name()),
            format!(
                "Update fields in an editable {} form session.",
                descriptor.form_name()
            ),
            snapshot_output_schema.clone(),
            session_mutation_annotations(true),
        )?,
        validate: editor_tool_definition::<SessionIdInput>(
            tool_names.validate,
            format!("Validate {} edit session", descriptor.form_name()),
            format!(
                "Validate the current values for an editable {} form session.",
                descriptor.form_name()
            ),
            snapshot_output_schema,
            read_only_annotations(),
        )?,
        close: editor_tool_definition::<McpFormEditCloseInput>(
            tool_names.close,
            format!("Close {} edit session", descriptor.form_name()),
            format!(
                "Close an editable {} form value-holder session.",
                descriptor.form_name()
            ),
            close_editor_output_schema(),
            session_mutation_annotations(true),
        )?,
        close_all: editor_tool_definition::<()>(
            tool_names.close_all,
            format!("Close all {} edit sessions", descriptor.form_name()),
            format!(
                "Close every active editable {} form value-holder session.",
                descriptor.form_name()
            ),
            close_all_editor_output_schema(descriptor),
            session_mutation_annotations(true),
        )?,
    })
}

fn editor_tool_definition<Input>(
    name: String,
    title: String,
    description: String,
    output_schema: McpSchema,
    annotations: McpToolAnnotations,
) -> Result<McpTypedTool<Input>, McpToolError>
where
    Input: McpToolInput,
{
    component_shape_mcp::tool_definition_for_input_with_annotations::<Input>(
        name,
        Some(title),
        Some(description),
        Some(output_schema),
        Some(annotations),
    )
}

fn read_only_annotations() -> McpToolAnnotations {
    McpToolAnnotations::new()
        .read_only(true)
        .destructive(false)
        .idempotent(true)
        .open_world(false)
}

fn session_mutation_annotations(destructive: bool) -> McpToolAnnotations {
    McpToolAnnotations::new()
        .read_only(false)
        .destructive(destructive)
        .idempotent(false)
        .open_world(false)
}

fn session_submit_tool_definition<Response>(
    descriptor: McpFormDescriptor,
) -> Result<McpTypedTool<McpFormEditSubmitInput>, McpToolError>
where
    Response: McpJsonSchema,
{
    let tool_names = editor_tool_names(descriptor);
    editor_tool_definition::<McpFormEditSubmitInput>(
        tool_names.submit,
        format!("Submit {} edit session", descriptor.form_name()),
        format!(
            "Submit the current values from an editable {} form session.",
            descriptor.form_name()
        ),
        descriptor.output_schema::<Response>(),
        McpToolAnnotations::new()
            .read_only(false)
            .destructive(true)
            .idempotent(false)
            .open_world(true),
    )
}

fn open_edit_session<Form>(
    input: McpFormEditOpenInput<Form>,
    sessions: &SharedEditSessions<Form::ValueHolder>,
) -> ToolCallResult
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
{
    let mut sessions = match sessions.lock() {
        Ok(sessions) => sessions,
        Err(error) => {
            return component_shape_mcp::tool_error_result_for(McpToolError::handler(format!(
                "form edit session lock failed: {error}"
            )));
        },
    };
    let opened = sessions.open(input.holder, input.present_fields, input.values);
    let session = sessions
        .sessions
        .get(&opened.session_id)
        .expect("opened session should be readable");
    match edit_session_snapshot::<Form>(&opened.session_id, session) {
        Ok(mut value) => {
            insert_edit_session_cleanup(&mut value, opened.cleanup);
            component_shape_mcp::tool_structured_result(value)
        },
        Err(error) => component_shape_mcp::tool_error_result_for(error),
    }
}

fn list_edit_sessions<Form>(sessions: &SharedEditSessions<Form::ValueHolder>) -> ToolCallResult
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
{
    let mut sessions = match sessions.lock() {
        Ok(sessions) => sessions,
        Err(error) => {
            return component_shape_mcp::tool_error_result_for(McpToolError::handler(format!(
                "form edit session lock failed: {error}"
            )));
        },
    };
    let cleanup = EditSessionCleanup {
        expired_session_ids: sessions.expire_idle_sessions(Instant::now()),
        evicted_session_ids: Vec::new(),
    };

    match edit_session_list_snapshot::<Form>(&sessions, cleanup) {
        Ok(value) => component_shape_mcp::tool_structured_result(value),
        Err(error) => component_shape_mcp::tool_error_result_for(error),
    }
}

fn read_edit_session<Form>(
    input: SessionIdInput,
    sessions: &SharedEditSessions<Form::ValueHolder>,
) -> ToolCallResult
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
{
    let mut sessions = match sessions.lock() {
        Ok(sessions) => sessions,
        Err(error) => {
            return component_shape_mcp::tool_error_result_for(McpToolError::handler(format!(
                "form edit session lock failed: {error}"
            )));
        },
    };
    match sessions.get(&input.session_id) {
        Ok(session) => edit_session_snapshot_result::<Form>(&input.session_id, session),
        Err(error) => component_shape_mcp::tool_error_result_for(error),
    }
}

fn patch_edit_session<Form>(
    input: McpFormEditPatchInput<Form>,
    sessions: &SharedEditSessions<Form::ValueHolder>,
) -> ToolCallResult
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
{
    let patch = input.into_patch();
    let mut sessions = match sessions.lock() {
        Ok(sessions) => sessions,
        Err(error) => {
            return component_shape_mcp::tool_error_result_for(McpToolError::handler(format!(
                "form edit session lock failed: {error}"
            )));
        },
    };
    let session = match sessions.get_mut(&patch.session_id) {
        Ok(session) => session,
        Err(error) => return component_shape_mcp::tool_error_result_for(error),
    };
    if let Some(expected_revision) = patch.expected_revision
        && session.revision != expected_revision
    {
        return component_shape_mcp::tool_error_result_for(revision_mismatch_error(
            expected_revision,
            session.revision,
        ));
    }

    let opened_order = session.opened_order;
    let mut values = if patch.replace {
        Map::new()
    } else {
        session.values.clone()
    };
    let next_revision = session.revision + 1;
    for field in patch.clear {
        if !Form::descriptor()
            .fields()
            .iter()
            .any(|descriptor| descriptor.name() == field)
        {
            return component_shape_mcp::tool_error_result_for(McpToolError::UnknownField {
                field,
            });
        }
        values.remove(&field);
    }
    for (field, value) in patch.values {
        values.insert(field, value);
    }
    let mut patched_session = match edit_session_from_values::<Form>(values) {
        Ok(patched_session) => patched_session,
        Err(error) => return component_shape_mcp::tool_error_result_for(error),
    };
    patched_session.revision = next_revision;
    patched_session.opened_order = opened_order;
    patched_session.last_accessed_at = Instant::now();

    *session = patched_session;

    edit_session_snapshot_result::<Form>(&patch.session_id, session)
}

fn validate_edit_session<Form>(
    input: SessionIdInput,
    sessions: &SharedEditSessions<Form::ValueHolder>,
) -> ToolCallResult
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
{
    let mut sessions = match sessions.lock() {
        Ok(sessions) => sessions,
        Err(error) => {
            return component_shape_mcp::tool_error_result_for(McpToolError::handler(format!(
                "form edit session lock failed: {error}"
            )));
        },
    };
    let session = match sessions.get(&input.session_id) {
        Ok(session) => session,
        Err(error) => return component_shape_mcp::tool_error_result_for(error),
    };

    edit_session_snapshot_result::<Form>(&input.session_id, session)
}

fn submit_edit_session<Form, Call>(
    input: McpFormEditSubmitInput,
    sessions: &SharedEditSessions<Form::ValueHolder>,
    call: &Call,
) -> ToolCallResult
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
    Call: Fn(Form::ValueHolder) -> ToolCallResult,
{
    match edit_session_submit_holder::<Form>(&input, sessions) {
        Ok(submitted) => {
            let result = call(submitted.holder);
            close_submitted_edit_session_on_success(&input, sessions, submitted.revision, &result);
            result
        },
        Err(error) => component_shape_mcp::tool_error_result_for(error),
    }
}

fn submit_edit_session_async<Form, Call>(
    input: McpFormEditSubmitInput,
    sessions: &SharedEditSessions<Form::ValueHolder>,
    call: &Call,
) -> ToolFuture
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default + Send + 'static,
    Call: Fn(Form::ValueHolder) -> ToolFuture,
{
    match edit_session_submit_holder::<Form>(&input, sessions) {
        Ok(submitted) => {
            let sessions = Arc::clone(sessions);
            let future = call(submitted.holder);
            Box::pin(async move {
                let result = future.await;
                close_submitted_edit_session_on_success(
                    &input,
                    &sessions,
                    submitted.revision,
                    &result,
                );
                result
            })
        },
        Err(error) => Box::pin(std::future::ready(
            component_shape_mcp::tool_error_result_for(error),
        )),
    }
}

fn edit_session_submit_holder<Form>(
    input: &McpFormEditSubmitInput,
    sessions: &SharedEditSessions<Form::ValueHolder>,
) -> Result<SubmittedEditSession<Form::ValueHolder>, McpToolError>
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
{
    let mut sessions = sessions.lock().map_err(|error| {
        McpToolError::handler(format!("form edit session lock failed: {error}"))
    })?;
    let session = sessions.get(&input.session_id)?;
    if let Some(expected_revision) = input.expected_revision
        && session.revision != expected_revision
    {
        return Err(revision_mismatch_error(expected_revision, session.revision));
    }
    let issues = edit_session_validation_issues::<Form>(Form::descriptor(), session);
    if !issues.is_empty() {
        return Err(validation_issues_error(issues));
    }
    Ok(SubmittedEditSession {
        holder: session.holder.clone(),
        revision: session.revision,
    })
}

fn close_submitted_edit_session_on_success<Holder>(
    input: &McpFormEditSubmitInput,
    sessions: &SharedEditSessions<Holder>,
    submitted_revision: u64,
    result: &ToolCallResult,
) {
    if !input.close_on_success || matches!(result.is_error, Some(true)) {
        return;
    }

    let Ok(mut sessions) = sessions.lock() else {
        return;
    };
    if matches!(sessions.get(&input.session_id), Ok(session) if session.revision == submitted_revision)
    {
        let _ = sessions.close(&input.session_id);
    }
}

fn revision_mismatch_error(expected_revision: u64, current_revision: u64) -> McpToolError {
    McpToolError::validation(format!(
        "edit session revision mismatch: expected {expected_revision}, current {current_revision}"
    ))
}

fn close_edit_session<Form>(
    input: McpFormEditCloseInput,
    sessions: &SharedEditSessions<Form::ValueHolder>,
) -> ToolCallResult
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
{
    let mut sessions = match sessions.lock() {
        Ok(sessions) => sessions,
        Err(error) => {
            return component_shape_mcp::tool_error_result_for(McpToolError::handler(format!(
                "form edit session lock failed: {error}"
            )));
        },
    };
    let session = match sessions.get(&input.session_id) {
        Ok(session) => session,
        Err(error) => return component_shape_mcp::tool_error_result_for(error),
    };
    if let Some(expected_revision) = input.expected_revision
        && session.revision != expected_revision
    {
        return component_shape_mcp::tool_error_result_for(revision_mismatch_error(
            expected_revision,
            session.revision,
        ));
    }
    match sessions.close(&input.session_id) {
        Ok(closed) => component_shape_mcp::tool_structured_result(serde_json::json!({
            "session_id": input.session_id,
            "closed": closed,
        })),
        Err(error) => component_shape_mcp::tool_error_result_for(error),
    }
}

fn close_all_edit_sessions<Form>(sessions: &SharedEditSessions<Form::ValueHolder>) -> ToolCallResult
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
{
    let mut sessions = match sessions.lock() {
        Ok(sessions) => sessions,
        Err(error) => {
            return component_shape_mcp::tool_error_result_for(McpToolError::handler(format!(
                "form edit session lock failed: {error}"
            )));
        },
    };
    let descriptor = Form::descriptor();
    let session_ids = sessions.close_all();
    component_shape_mcp::tool_structured_result(serde_json::json!({
        "form": descriptor.tool_name(),
        "form_name": descriptor.form_name(),
        "closed_count": session_ids.len(),
        "session_ids": session_ids,
    }))
}

fn decode_open_values<Form>(
    call: McpToolCall,
) -> Result<(Form::ValueHolder, BTreeSet<String>, Map<String, Value>), McpToolError>
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
{
    let mut arguments = call.into_arguments();
    let values = arguments.take_raw("values");
    arguments.finish()?;
    let Some(values) = values else {
        return Ok((Form::ValueHolder::default(), BTreeSet::new(), Map::new()));
    };
    let Value::Object(values) = values else {
        return Err(McpToolError::decode("values", "expected an object"));
    };

    let session = edit_session_from_values::<Form>(values)?;
    Ok((session.holder, session.present_fields, session.values))
}

fn edit_session_from_values<Form>(
    values: Map<String, Value>,
) -> Result<EditSession<Form::ValueHolder>, McpToolError>
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
{
    let mut holder = Form::ValueHolder::default();
    let mut present_fields = BTreeSet::new();
    let mut accepted_values = Map::new();
    for (field, value) in values {
        Form::decode_field(&mut holder, &field, McpAny::from(value.clone()))?;
        present_fields.insert(field.clone());
        accepted_values.insert(field, value);
    }

    Ok(EditSession {
        holder,
        present_fields,
        values: accepted_values,
        revision: 0,
        opened_order: 0,
        last_accessed_at: Instant::now(),
    })
}

struct FormPatch {
    session_id: String,
    values: Map<String, Value>,
    clear: Vec<String>,
    replace: bool,
    expected_revision: Option<u64>,
}

struct SubmittedEditSession<Holder> {
    holder: Holder,
    revision: u64,
}

struct McpFormEditOpenInput<Form>
where
    Form: McpForm,
{
    holder: Form::ValueHolder,
    present_fields: BTreeSet<String>,
    values: Map<String, Value>,
}

impl<Form> McpToolInput for McpFormEditOpenInput<Form>
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
{
    fn input_schema() -> McpSchema {
        open_editor_input_schema(Form::descriptor().fields())
    }

    fn from_tool_call(call: McpToolCall) -> Result<Self, McpToolError> {
        let (holder, present_fields, values) = decode_open_values::<Form>(call)?;
        Ok(Self {
            holder,
            present_fields,
            values,
        })
    }
}

struct McpFormEditPatchInput<Form>
where
    Form: McpForm,
{
    patch: FormPatch,
    _form: PhantomData<fn() -> Form>,
}

impl<Form> McpFormEditPatchInput<Form>
where
    Form: McpForm,
{
    fn into_patch(self) -> FormPatch {
        self.patch
    }
}

impl<Form> McpToolInput for McpFormEditPatchInput<Form>
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
{
    fn input_schema() -> McpSchema {
        patch_editor_input_schema(Form::descriptor().fields())
    }

    fn from_tool_call(call: McpToolCall) -> Result<Self, McpToolError> {
        let mut arguments = call.into_arguments();
        let session_id = arguments.take_required_tool_value::<String>("session_id")?;
        let values = arguments.take_raw("values");
        let clear = arguments.take_raw("clear");
        let replace = arguments
            .take_present_tool_value::<bool>("replace")?
            .unwrap_or(false);
        let expected_revision = arguments.take_present_tool_value::<u64>("expected_revision")?;
        arguments.finish()?;

        let values = match values {
            Some(Value::Object(values)) => values,
            Some(_) => return Err(McpToolError::decode("values", "expected an object")),
            None => Map::new(),
        };
        let clear = match clear {
            Some(Value::Array(values)) => values
                .into_iter()
                .map(|value| {
                    value.as_str().map(str::to_string).ok_or_else(|| {
                        McpToolError::decode("clear", "expected an array of field names")
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            Some(_) => {
                return Err(McpToolError::decode(
                    "clear",
                    "expected an array of field names",
                ));
            },
            None => Vec::new(),
        };

        Ok(Self {
            patch: FormPatch {
                session_id,
                values,
                clear,
                replace,
                expected_revision,
            },
            _form: PhantomData,
        })
    }
}

#[derive(component_shape_mcp::McpToolInput)]
#[mcp(crate = component_shape_mcp)]
struct SessionIdInput {
    session_id: String,
}

struct McpFormEditCloseInput {
    session_id: String,
    expected_revision: Option<u64>,
}

impl McpToolInput for McpFormEditCloseInput {
    fn input_schema() -> McpSchema {
        let mut properties = McpSchemaProperties::new();
        properties.insert("session_id".to_string(), McpSchema::string());
        properties.insert("expected_revision".to_string(), expected_revision_schema());
        component_shape_mcp::object_schema(properties, ["session_id"])
    }

    fn from_tool_call(call: McpToolCall) -> Result<Self, McpToolError> {
        let mut arguments = call.into_arguments();
        let session_id = arguments.take_required_tool_value::<String>("session_id")?;
        let expected_revision = arguments.take_present_tool_value::<u64>("expected_revision")?;
        arguments.finish()?;
        Ok(Self {
            session_id,
            expected_revision,
        })
    }
}

struct McpFormEditSubmitInput {
    session_id: String,
    close_on_success: bool,
    expected_revision: Option<u64>,
}

impl McpToolInput for McpFormEditSubmitInput {
    fn input_schema() -> McpSchema {
        let mut properties = McpSchemaProperties::new();
        properties.insert("session_id".to_string(), McpSchema::string());
        properties.insert(
            "close_on_success".to_string(),
            McpSchema::boolean()
                .with_default(true)
                .with_description("When true, close the edit session after a successful submit."),
        );
        properties.insert("expected_revision".to_string(), expected_revision_schema());
        component_shape_mcp::object_schema(properties, ["session_id"])
    }

    fn from_tool_call(call: McpToolCall) -> Result<Self, McpToolError> {
        let mut arguments = call.into_arguments();
        let session_id = arguments.take_required_tool_value::<String>("session_id")?;
        let close_on_success = arguments
            .take_present_tool_value::<bool>("close_on_success")?
            .unwrap_or(true);
        let expected_revision = arguments.take_present_tool_value::<u64>("expected_revision")?;
        arguments.finish()?;
        Ok(Self {
            session_id,
            close_on_success,
            expected_revision,
        })
    }
}

fn edit_session_snapshot_result<Form>(
    session_id: &str,
    session: &EditSession<Form::ValueHolder>,
) -> ToolCallResult
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
{
    match edit_session_snapshot::<Form>(session_id, session) {
        Ok(value) => component_shape_mcp::tool_structured_result(value),
        Err(error) => component_shape_mcp::tool_error_result_for(error),
    }
}

fn edit_session_snapshot<Form>(
    session_id: &str,
    session: &EditSession<Form::ValueHolder>,
) -> Result<Value, McpToolError>
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
{
    let descriptor = Form::descriptor();
    let issues = edit_session_validation_issues::<Form>(descriptor, session);
    let errors = validation_issue_values(&issues);

    Ok(serde_json::json!({
        "form": descriptor.tool_name(),
        "form_name": descriptor.form_name(),
        "session_id": session_id,
        "revision": session.revision,
        "fields": edit_session_fields(
            descriptor,
            &session.present_fields,
            &session.values,
            &issues
        ),
        "present_fields": session.present_fields.iter().collect::<Vec<_>>(),
        "missing_required": required_missing_fields(descriptor, &session.present_fields),
        "valid": errors.is_empty(),
        "errors": errors,
        "values": session.values,
        "submit_arguments": session.values,
    }))
}

fn edit_session_list_snapshot<Form>(
    sessions: &EditSessions<Form::ValueHolder>,
    cleanup: EditSessionCleanup,
) -> Result<Value, McpToolError>
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
{
    let descriptor = Form::descriptor();
    let session_limit = sessions.session_limit();
    let session_idle_timeout_ms = sessions
        .session_idle_timeout()
        .map(duration_millis_saturating);
    let sessions = sessions
        .sessions
        .iter()
        .map(|(session_id, session)| edit_session_snapshot::<Form>(session_id, session))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(serde_json::json!({
        "form": descriptor.tool_name(),
        "form_name": descriptor.form_name(),
        "session_limit": session_limit,
        "session_idle_timeout_ms": session_idle_timeout_ms,
        "session_count": sessions.len(),
        "sessions": sessions,
        "cleanup": edit_session_cleanup_value(cleanup),
    }))
}

fn insert_edit_session_cleanup(value: &mut Value, cleanup: EditSessionCleanup) {
    if let Value::Object(object) = value {
        object.insert("cleanup".to_string(), edit_session_cleanup_value(cleanup));
    }
}

fn edit_session_cleanup_value(cleanup: EditSessionCleanup) -> Value {
    serde_json::json!({
        "expired_count": cleanup.expired_session_ids.len(),
        "expired_session_ids": cleanup.expired_session_ids,
        "evicted_count": cleanup.evicted_session_ids.len(),
        "evicted_session_ids": cleanup.evicted_session_ids,
    })
}

fn duration_millis_saturating(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn edit_session_fields(
    descriptor: McpFormDescriptor,
    present_fields: &BTreeSet<String>,
    values: &Map<String, Value>,
    issues: &[McpValidationIssue],
) -> Vec<Value> {
    descriptor
        .fields()
        .iter()
        .map(|field| {
            let present = present_fields.contains(field.name());
            let value = values.get(field.name()).cloned().unwrap_or(Value::Null);
            let missing = field.required() && !present;
            let errors = issues
                .iter()
                .filter(|issue| issue.field() == Some(field.name()))
                .map(McpValidationIssue::to_value)
                .collect::<Vec<_>>();
            serde_json::json!({
                "name": field.name(),
                "label": field.label(),
                "description": field.description(),
                "examples": field.examples(),
                "value_type": field.value_type().as_str(),
                "required": field.required(),
                "present": present,
                "has_value": present,
                "value": value,
                "missing": missing,
                "errors": errors,
                "has_default": field.has_default(),
                "component_backed": field.component_backed(),
                "schema": schema_for_field(*field).into_value(),
            })
        })
        .collect()
}

fn edit_session_validation_issues<Form>(
    descriptor: McpFormDescriptor,
    session: &EditSession<Form::ValueHolder>,
) -> Vec<McpValidationIssue>
where
    Form: McpEditableForm,
    Form::ValueHolder: Clone + Default,
{
    let mut issues = required_missing_fields(descriptor, &session.present_fields)
        .into_iter()
        .map(McpValidationIssue::required)
        .collect::<Vec<_>>();
    if let Err(validation_issues) = <Form as McpForm>::validate_holder_issues(&session.holder) {
        issues.extend(validation_issues);
    }
    issues
}

fn validation_issue_values(issues: &[McpValidationIssue]) -> Vec<Value> {
    issues
        .iter()
        .map(McpValidationIssue::to_value)
        .collect::<Vec<_>>()
}

fn required_missing_fields(
    descriptor: McpFormDescriptor,
    present_fields: &BTreeSet<String>,
) -> Vec<&'static str> {
    descriptor
        .fields()
        .iter()
        .filter(|field| field.required() && !present_fields.contains(field.name()))
        .map(|field| field.name())
        .collect()
}

/// Build a server containing every inventory-discovered form submit handler and
/// editable form session tool.
pub fn server() -> Result<McpServer, McpToolError> {
    builder().build()
}

/// Serve every inventory-discovered form submit handler and editable form
/// session tool over stdio.
pub async fn serve_stdio() -> ServeStdioResult {
    server()?.serve_stdio().await
}

/// Serve every inventory-discovered form submit handler and editable form
/// session tool over stdio from a synchronous `main`.
pub fn serve_stdio_blocking() -> ServeStdioResult {
    server()?.serve_stdio_blocking()
}

/// Build a generated form submit server with application-owned metadata.
pub fn server_named(
    server_name: impl Into<std::borrow::Cow<'static, str>>,
    server_version: impl Into<std::borrow::Cow<'static, str>>,
) -> Result<McpServer, McpToolError> {
    builder_named(server_name, server_version).build()
}

/// Build a server builder containing every inventory-discovered form submit
/// handler and editable form session tool.
pub fn builder() -> McpServerBuilder {
    builder_named(DEFAULT_SERVER_NAME, env!("CARGO_PKG_VERSION"))
}

/// Build a generated form submit and editor server builder with
/// application-owned metadata.
pub fn builder_named(
    server_name: impl Into<std::borrow::Cow<'static, str>>,
    server_version: impl Into<std::borrow::Cow<'static, str>>,
) -> McpServerBuilder {
    McpServer::builder(server_name, server_version).register(register)
}

/// Add every inventory-discovered form submit handler and editable form session
/// tool to a shared MCP server.
pub fn register(server: &mut McpServer) -> Result<(), McpToolError> {
    register_with_options(server, McpFormRegistrationOptions::all()).map(|_| ())
}

/// Add inventory-discovered form tools to a shared MCP server using a
/// registration profile.
pub fn register_with_options(
    server: &mut McpServer,
    options: McpFormRegistrationOptions,
) -> Result<McpFormRegistrationReport, McpToolError> {
    let report = registration_report_for_options(options)?;
    if !options.registers_tools() {
        return Ok(report);
    }

    let resource_specs = form_resource_specs_for_options(options)?;
    ensure_tool_definitions_available(server, &report.tool_definitions)?;
    register_form_resource_specs_if_missing(server, resource_specs)?;
    let submit_tool_names = submit_handler_tool_names();
    for registration in registry::submit_handler_registrations() {
        if !options.submit_handlers {
            continue;
        }
        if options.editors && options.edit_submit {
            registration.register(server)?;
        } else {
            registration.register_submit(server)?;
        }
    }
    for registration in registry::editor_registrations() {
        if !options.editors {
            continue;
        }
        if options.submit_handlers
            && options.edit_submit
            && submit_tool_names.contains(&registration.descriptor().tool_name())
        {
            continue;
        }
        registration.register(server)?;
    }
    Ok(report)
}

/// Return MCP tool definitions selected by a registration profile without
/// mutating a server.
pub fn tool_definitions_with_options(
    options: McpFormRegistrationOptions,
) -> Result<Vec<ToolDefinition>, McpToolError> {
    Ok(registration_report_for_options(options)?.tool_definitions)
}

fn registration_report_for_options(
    options: McpFormRegistrationOptions,
) -> Result<McpFormRegistrationReport, McpToolError> {
    if !options.registers_tools() {
        return Ok(
            McpFormRegistrationReport::from_tool_definitions(tool_definitions()?).metadata_only(),
        );
    }

    let mut seen = BTreeSet::new();
    let mut definitions = Vec::new();
    let mut skipped_tools = Vec::new();
    let submit_tool_names = submit_handler_tool_names();

    for registration in registry::submit_handler_registrations() {
        let descriptor = registration.descriptor();
        if options.submit_handlers {
            let tool_definitions = if options.editors && options.edit_submit {
                registration.tool_definitions()?
            } else {
                registration.submit_tool_definitions()?
            };
            for definition in tool_definitions {
                push_tool_definition(&mut seen, &mut definitions, definition)?;
            }
        } else {
            push_skipped_tool(
                &mut skipped_tools,
                descriptor.tool_name(),
                "submit_handlers_disabled",
            );
        }

        if !options.editors {
            push_skipped_editor_tool_names(
                &mut skipped_tools,
                descriptor,
                "editors_disabled",
                options.edit_submit,
            );
            if !options.edit_submit {
                push_skipped_tool(
                    &mut skipped_tools,
                    editor_tool_names(descriptor).submit,
                    "edit_submit_disabled",
                );
            }
        } else if !options.edit_submit {
            push_skipped_tool(
                &mut skipped_tools,
                editor_tool_names(descriptor).submit,
                "edit_submit_disabled",
            );
        } else if !options.submit_handlers {
            push_skipped_tool(
                &mut skipped_tools,
                editor_tool_names(descriptor).submit,
                "submit_handlers_disabled",
            );
        }
    }

    for registration in registry::editor_registrations() {
        let descriptor = registration.descriptor();
        let covered_by_submit_handler = options.submit_handlers
            && options.edit_submit
            && submit_tool_names.contains(&descriptor.tool_name());
        if options.editors && !covered_by_submit_handler {
            for definition in registration.tool_definitions()? {
                push_tool_definition(&mut seen, &mut definitions, definition)?;
            }
        } else if !options.editors && !submit_tool_names.contains(&descriptor.tool_name()) {
            push_skipped_editor_tool_names(
                &mut skipped_tools,
                descriptor,
                "editors_disabled",
                false,
            );
        }
    }

    let registered_tool_names = definitions
        .iter()
        .map(|definition| definition.name.to_string())
        .collect();
    Ok(McpFormRegistrationReport {
        tool_definitions: definitions,
        registered_tool_names,
        skipped_tools,
        context_registration_count: 0,
    })
}

fn push_skipped_editor_tool_names(
    skipped_tools: &mut Vec<McpSkippedTool>,
    descriptor: McpFormDescriptor,
    reason: &'static str,
    include_submit: bool,
) {
    let names = editor_tool_names(descriptor);
    for name in [
        names.open,
        names.list,
        names.read,
        names.patch,
        names.validate,
        names.close,
        names.close_all,
    ] {
        push_skipped_tool(skipped_tools, name, reason);
    }
    if include_submit {
        push_skipped_tool(skipped_tools, names.submit, reason);
    }
}

fn push_skipped_tool(skipped_tools: &mut Vec<McpSkippedTool>, name: String, reason: &'static str) {
    if skipped_tools.iter().any(|skipped| skipped.name == name) {
        return;
    }
    skipped_tools.push(McpSkippedTool { name, reason });
}

fn input_schema_for_fields(fields: &[McpField]) -> McpSchema {
    input_schema_for_fields_with_required(fields, true)
}

fn optional_input_schema_for_fields(fields: &[McpField]) -> McpSchema {
    input_schema_for_fields_with_required(fields, false)
}

fn input_schema_for_fields_with_required(fields: &[McpField], include_required: bool) -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    let mut required = Vec::new();

    for field in fields {
        properties.insert(field.name().to_string(), schema_for_field(*field));
        if include_required && field.required() {
            required.push(field.name());
        }
    }

    component_shape_mcp::object_schema(properties, required)
}

fn open_editor_input_schema(fields: &[McpField]) -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert(
        "values".to_string(),
        optional_input_schema_for_fields(fields),
    );
    component_shape_mcp::object_schema(properties, std::iter::empty::<&str>())
}

fn patch_editor_input_schema(fields: &[McpField]) -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert("session_id".to_string(), McpSchema::string());
    properties.insert(
        "values".to_string(),
        optional_input_schema_for_fields(fields),
    );
    properties.insert("expected_revision".to_string(), expected_revision_schema());
    properties.insert("clear".to_string(), field_name_array_schema(fields));
    properties.insert(
        "replace".to_string(),
        McpSchema::boolean()
            .with_default(false)
            .with_description(
                "When true, replace the session values with this patch instead of merging into the existing values.",
            ),
    );
    component_shape_mcp::object_schema(properties, ["session_id"])
}

fn edit_session_snapshot_output_schema(fields: &[McpField]) -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert("form".to_string(), McpSchema::string());
    properties.insert("form_name".to_string(), McpSchema::string());
    properties.insert("session_id".to_string(), McpSchema::string());
    properties.insert(
        "revision".to_string(),
        McpSchema::integer().with_minimum(0_u64),
    );
    properties.insert(
        "fields".to_string(),
        edit_session_fields_output_schema(fields),
    );
    properties.insert(
        "present_fields".to_string(),
        field_name_array_schema(fields),
    );
    properties.insert(
        "missing_required".to_string(),
        field_name_array_schema(fields),
    );
    properties.insert("valid".to_string(), McpSchema::boolean());
    properties.insert(
        "errors".to_string(),
        component_shape_mcp::array_schema(validation_issue_schema()),
    );
    properties.insert(
        "values".to_string(),
        optional_input_schema_for_fields(fields),
    );
    properties.insert(
        "submit_arguments".to_string(),
        optional_input_schema_for_fields(fields),
    );
    properties.insert("cleanup".to_string(), edit_session_cleanup_schema());

    component_shape_mcp::object_schema(
        properties,
        [
            "form",
            "form_name",
            "session_id",
            "revision",
            "fields",
            "present_fields",
            "missing_required",
            "valid",
            "errors",
            "values",
            "submit_arguments",
        ],
    )
}

fn edit_session_list_output_schema(fields: &[McpField]) -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert("form".to_string(), McpSchema::string());
    properties.insert("form_name".to_string(), McpSchema::string());
    properties.insert("session_limit".to_string(), session_limit_schema());
    properties.insert(
        "session_idle_timeout_ms".to_string(),
        session_idle_timeout_schema(),
    );
    properties.insert(
        "session_count".to_string(),
        McpSchema::integer().with_minimum(0_u64),
    );
    properties.insert(
        "sessions".to_string(),
        component_shape_mcp::array_schema(edit_session_snapshot_output_schema(fields)),
    );
    properties.insert("cleanup".to_string(), edit_session_cleanup_schema());

    component_shape_mcp::object_schema(
        properties,
        [
            "form",
            "form_name",
            "session_limit",
            "session_idle_timeout_ms",
            "session_count",
            "sessions",
            "cleanup",
        ],
    )
}

fn edit_session_cleanup_schema() -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert(
        "expired_count".to_string(),
        McpSchema::integer().with_minimum(0_u64),
    );
    properties.insert(
        "expired_session_ids".to_string(),
        component_shape_mcp::array_schema(McpSchema::string()),
    );
    properties.insert(
        "evicted_count".to_string(),
        McpSchema::integer().with_minimum(0_u64),
    );
    properties.insert(
        "evicted_session_ids".to_string(),
        component_shape_mcp::array_schema(McpSchema::string()),
    );

    component_shape_mcp::object_schema(
        properties,
        [
            "expired_count",
            "expired_session_ids",
            "evicted_count",
            "evicted_session_ids",
        ],
    )
}

fn close_editor_output_schema() -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert("session_id".to_string(), McpSchema::string());
    properties.insert("closed".to_string(), McpSchema::boolean());

    component_shape_mcp::object_schema(properties, ["session_id", "closed"])
}

fn close_all_editor_output_schema(descriptor: McpFormDescriptor) -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert(
        "form".to_string(),
        McpSchema::string().with_const(descriptor.tool_name()),
    );
    properties.insert(
        "form_name".to_string(),
        McpSchema::string().with_const(descriptor.form_name()),
    );
    properties.insert(
        "closed_count".to_string(),
        McpSchema::integer().with_minimum(0_u64),
    );
    properties.insert(
        "session_ids".to_string(),
        component_shape_mcp::array_schema(McpSchema::string()),
    );

    component_shape_mcp::object_schema(
        properties,
        ["form", "form_name", "closed_count", "session_ids"],
    )
}

fn edit_session_fields_output_schema(fields: &[McpField]) -> McpSchema {
    if fields.is_empty() {
        return component_shape_mcp::array_schema(component_shape_mcp::object_schema(
            McpSchemaProperties::new(),
            std::iter::empty::<&str>(),
        ));
    }

    component_shape_mcp::array_schema(McpSchema::one_of(
        fields
            .iter()
            .map(|field| edit_session_field_output_schema(*field)),
    ))
}

fn edit_session_field_output_schema(field: McpField) -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert(
        "name".to_string(),
        McpSchema::string().with_enum_values([field.name()]),
    );
    properties.insert(
        "value_type".to_string(),
        McpSchema::string().with_enum_values([field.value_type().as_str()]),
    );
    properties.insert(
        "label".to_string(),
        McpSchema::string().with_const(field.label()),
    );
    properties.insert(
        "description".to_string(),
        component_shape_mcp::nullable_schema(McpSchema::string()),
    );
    properties.insert(
        "examples".to_string(),
        component_shape_mcp::array_schema(McpSchema::string()),
    );
    properties.insert(
        "required".to_string(),
        McpSchema::boolean().with_const(field.required()),
    );
    properties.insert("present".to_string(), McpSchema::boolean());
    properties.insert("has_value".to_string(), McpSchema::boolean());
    properties.insert(
        "value".to_string(),
        component_shape_mcp::nullable_schema(schema_for_field(field)),
    );
    properties.insert("missing".to_string(), McpSchema::boolean());
    properties.insert(
        "errors".to_string(),
        component_shape_mcp::array_schema(validation_issue_schema()),
    );
    properties.insert(
        "has_default".to_string(),
        McpSchema::boolean().with_const(field.has_default()),
    );
    properties.insert(
        "component_backed".to_string(),
        McpSchema::boolean().with_const(field.component_backed()),
    );
    properties.insert(
        "schema".to_string(),
        McpSchema::object().with_const(schema_for_field(field).into_value()),
    );

    component_shape_mcp::object_schema(
        properties,
        [
            "name",
            "label",
            "description",
            "examples",
            "value_type",
            "required",
            "present",
            "has_value",
            "value",
            "missing",
            "errors",
            "has_default",
            "component_backed",
            "schema",
        ],
    )
}

fn field_name_array_schema(fields: &[McpField]) -> McpSchema {
    McpSchema::array(McpSchema::string().with_enum_values(fields.iter().map(|field| field.name())))
        .with_unique_items(true)
}

fn validation_issue_schema() -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert(
        "scope".to_string(),
        McpSchema::string().with_enum_values(["form", "field", "element"]),
    );
    properties.insert("message".to_string(), McpSchema::string());
    properties.insert("field".to_string(), McpSchema::string());
    properties.insert("validator".to_string(), McpSchema::string());
    properties.insert("path".to_string(), McpSchema::string());
    properties.insert("label".to_string(), McpSchema::string());
    properties.insert(
        "target".to_string(),
        McpSchema::string().with_enum_values(["default", "full", "unwrapped"]),
    );
    properties.insert(
        "element_index".to_string(),
        McpSchema::integer().with_minimum(0_u64),
    );
    properties.insert(
        "params".to_string(),
        component_shape_mcp::array_schema(validation_param_schema()),
    );

    component_shape_mcp::object_schema(properties, ["scope", "message"])
}

fn validation_param_schema() -> McpSchema {
    let mut properties = McpSchemaProperties::new();
    properties.insert("name".to_string(), McpSchema::string());
    properties.insert("value".to_string(), McpSchema::string());
    properties.insert("expr".to_string(), McpSchema::string());

    component_shape_mcp::object_schema(properties, ["name"])
}

fn expected_revision_schema() -> McpSchema {
    McpSchema::integer().with_minimum(0_u64).with_description(
        "When set, apply this operation only if the edit session is still at this revision.",
    )
}

fn session_limit_schema() -> McpSchema {
    component_shape_mcp::nullable_schema(McpSchema::integer().with_minimum(1_u64).with_description(
        "Maximum active edit sessions retained for this form, or null when unlimited.",
    ))
}

fn session_idle_timeout_schema() -> McpSchema {
    component_shape_mcp::nullable_schema(McpSchema::integer().with_minimum(0_u64).with_description(
        "Idle milliseconds after which a session is expired, or null when idle expiry is disabled.",
    ))
}

fn schema_for_field(field: McpField) -> McpSchema {
    let base = (field.tool_value_schema())();
    let mut schema = if field.presence().optional() {
        component_shape_mcp::nullable_schema(base)
    } else {
        base
    };

    if let Some(object) = schema.as_object_mut() {
        object.insert(
            "x-rustType".to_string(),
            Value::String(field.value_type().as_str().to_string()),
        );
        object.insert(
            "x-gpuiFormPresence".to_string(),
            Value::String(format!("{:?}", field.presence())),
        );
        object.insert("title".to_string(), Value::String(field.label()));
        object.insert("x-gpuiFormLabel".to_string(), Value::String(field.label()));
        if let Some(label) = field.explicit_label() {
            object.insert(
                "x-gpuiFormExplicitLabel".to_string(),
                Value::String(label.to_string()),
            );
        }
        if let Some(description) = field.description() {
            object.insert(
                "description".to_string(),
                Value::String(description.to_string()),
            );
            object.insert(
                "x-gpuiFormDescription".to_string(),
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
        if field.component_backed() {
            object.insert("x-gpuiFormComponentBacked".to_string(), Value::Bool(true));
        }
        if field.mcp_input().supported() {
            object.insert(
                "x-gpuiFormComponentMcpInput".to_string(),
                component_shape_mcp::schema_for_input(field.mcp_input()).into_value(),
            );
        }
        if field.has_default() {
            object.insert("x-gpuiFormHasDefault".to_string(), Value::Bool(true));
        }
        component_shape_mcp::apply_validation_schema_metadata(
            object,
            "x-gpuiFormValidation",
            field.validation_rules(),
        );
    }

    schema
}

/// Return MCP tool definitions registered by the default generated server,
/// failing on duplicate names.
pub fn tool_definitions() -> Result<Vec<ToolDefinition>, McpToolError> {
    let mut seen = BTreeSet::new();
    let mut tools = Vec::new();
    let submit_tool_names = submit_handler_tool_names();
    for registration in registry::submit_handler_registrations() {
        for definition in registration.tool_definitions()? {
            push_tool_definition(&mut seen, &mut tools, definition)?;
        }
    }
    for registration in registry::editor_registrations() {
        if submit_tool_names.contains(&registration.descriptor().tool_name()) {
            continue;
        }
        for definition in registration.tool_definitions()? {
            push_tool_definition(&mut seen, &mut tools, definition)?;
        }
    }
    Ok(tools)
}

fn submit_handler_tool_names() -> BTreeSet<String> {
    registry::submit_handler_registrations()
        .map(|registration| registration.descriptor().tool_name())
        .collect()
}

fn push_tool_definition(
    seen: &mut BTreeSet<String>,
    tools: &mut Vec<ToolDefinition>,
    definition: ToolDefinition,
) -> Result<(), McpToolError> {
    let name = definition.name.to_string();
    if !seen.insert(name.clone()) {
        return Err(McpToolError::duplicate_tool(name));
    }
    tools.push(definition);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        McpField, McpForm as _, McpFormDescriptor, McpFormInput, McpInput, McpServer,
        McpToolInput as _, form, tool_name,
    };
    use gpui_form_schema::registry::{FieldValuePresence, RustPath, RustType};

    struct StringComponentShape;

    impl super::ComponentShapeMetadata for StringComponentShape {
        const MCP_INPUT: McpInput = McpInput::number();
    }
    impl super::ComponentShapeFor<String> for StringComponentShape {
        const MCP_INPUT: McpInput = McpInput::number();
    }

    #[test]
    fn tool_names_are_stable_and_mcp_friendly() {
        assert_eq!(
            tool_name("example::forms", "ContactRequest"),
            "example_forms_contact_request"
        );
        assert_eq!(tool_name("123", ""), "form_123");
    }

    #[test]
    fn input_schema_marks_required_and_optional_fields() {
        const FIELDS: &[McpField] = &[
            McpField::typed::<String>(
                "email",
                RustType::from_macro_tokens_unchecked("String"),
                FieldValuePresence::DirectStorage,
            ),
            McpField::typed::<u32>(
                "age",
                RustType::from_macro_tokens_unchecked("u32"),
                FieldValuePresence::Optional,
            ),
        ];
        let schema = McpFormDescriptor::new(
            "Contact",
            RustPath::from_macro_tokens_unchecked("tests"),
            FIELDS,
            false,
            component_shape_mcp::McpToolMetadata::new(),
        )
        .input_schema();

        assert_eq!(schema["properties"]["email"]["type"], "string");
        assert_eq!(schema["properties"]["age"]["anyOf"][0]["type"], "integer");
        assert_eq!(schema["required"], serde_json::json!(["email"]));
    }

    #[test]
    fn input_schema_records_component_mcp_input_when_present() {
        const FIELDS: &[McpField] = &[McpField::component::<String, StringComponentShape>(
            "available_on",
            RustType::from_macro_tokens_unchecked("String"),
            FieldValuePresence::DirectStorage,
        )];
        let schema = McpFormDescriptor::new(
            "Booking",
            RustPath::from_macro_tokens_unchecked("tests"),
            FIELDS,
            false,
            component_shape_mcp::McpToolMetadata::new(),
        )
        .input_schema();

        assert_eq!(schema["properties"]["available_on"]["type"], "string");
        assert_eq!(
            schema["properties"]["available_on"]["x-gpuiFormComponentMcpInput"]["type"],
            "number"
        );
        assert_eq!(
            schema["properties"]["available_on"]["x-gpuiFormComponentBacked"],
            true
        );
    }

    #[test]
    fn input_schema_supports_optional_wrapped_rust_type() {
        const FIELDS: &[McpField] = &[McpField::typed::<Option<String>>(
            "notes",
            RustType::from_macro_tokens_unchecked("Option<String>"),
            FieldValuePresence::DirectStorage,
        )];
        let schema = McpFormDescriptor::new(
            "Request",
            RustPath::from_macro_tokens_unchecked("tests"),
            FIELDS,
            false,
            component_shape_mcp::McpToolMetadata::new(),
        )
        .input_schema();

        assert_eq!(schema["properties"]["notes"]["anyOf"][0]["type"], "string");
        assert_eq!(schema["properties"]["notes"]["anyOf"][1]["type"], "null");
    }

    #[test]
    fn input_schema_supports_nested_optional_vec_types() {
        const FIELDS: &[McpField] = &[McpField::typed::<Vec<Option<String>>>(
            "tags",
            RustType::from_macro_tokens_unchecked("Vec<Option<String>>"),
            FieldValuePresence::DirectStorage,
        )];
        let schema = McpFormDescriptor::new(
            "Request",
            RustPath::from_macro_tokens_unchecked("tests"),
            FIELDS,
            false,
            component_shape_mcp::McpToolMetadata::new(),
        )
        .input_schema();

        assert_eq!(schema["properties"]["tags"]["type"], "array");
        assert_eq!(
            schema["properties"]["tags"]["items"]["anyOf"][0]["type"],
            "string"
        );
        assert_eq!(
            schema["properties"]["tags"]["items"]["anyOf"][1]["type"],
            "null"
        );
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn mcp_json_schema_supports_chrono_when_feature_enabled() {
        let schema = <chrono::NaiveDate as super::McpJsonSchema>::json_schema();

        assert_eq!(schema["type"], "string");
        assert_eq!(schema["format"], "date");
    }

    #[cfg(feature = "rust_decimal")]
    #[test]
    fn mcp_json_schema_supports_decimal_when_feature_enabled() {
        let schema = <rust_decimal::Decimal as super::McpJsonSchema>::json_schema();

        assert_eq!(schema["anyOf"][0]["type"], "number");
        assert_eq!(schema["anyOf"][1]["type"], "string");
    }

    #[test]
    fn input_schema_uses_field_tool_value_schema_when_no_component_input_exists() {
        type UserId = u64;

        const FIELDS: &[McpField] = &[McpField::typed::<UserId>(
            "user_id",
            RustType::from_macro_tokens_unchecked("UserId"),
            FieldValuePresence::DirectStorage,
        )];

        let schema = McpFormDescriptor::new(
            "Request",
            RustPath::from_macro_tokens_unchecked("tests"),
            FIELDS,
            false,
            component_shape_mcp::McpToolMetadata::new(),
        )
        .input_schema();

        assert_eq!(schema["properties"]["user_id"]["type"], "integer");
        assert_eq!(schema["properties"]["user_id"]["x-rustType"], "UserId");
    }

    #[test]
    fn input_schema_keeps_field_tool_value_schema_ahead_of_component_mcp_input() {
        const FIELDS: &[McpField] = &[McpField::component::<String, StringComponentShape>(
            "window",
            RustType::from_macro_tokens_unchecked("String"),
            FieldValuePresence::DirectStorage,
        )];

        let schema = McpFormDescriptor::new(
            "Request",
            RustPath::from_macro_tokens_unchecked("tests"),
            FIELDS,
            false,
            component_shape_mcp::McpToolMetadata::new(),
        )
        .input_schema();

        assert_eq!(schema["properties"]["window"]["type"], "string");
        assert_eq!(
            schema["properties"]["window"]["x-gpuiFormComponentMcpInput"]["type"],
            "number"
        );
        assert_eq!(schema["properties"]["window"]["x-rustType"], "String");
    }

    #[test]
    fn server_accepts_custom_metadata() {
        let mut server = McpServer::new("example-form-server".to_string(), "9.9.9");
        let descriptor = Booking::descriptor();

        assert_eq!(descriptor.tool_name(), "reserve_booking");
        assert_eq!(descriptor.title(), "Reserve booking");
        assert_eq!(descriptor.description(), "Reserve a booking window.");

        form::<Booking>(&mut server)
            .holder(|holder| {
                Ok::<_, String>(BookingResponse {
                    available_on: holder.available_on,
                })
            })
            .expect("tool should register");
        let resource_uris = super::form_resource_uris(descriptor);
        assert_eq!(server.resource_count(), 3);
        for uri in resource_uris.all() {
            assert!(server.contains_resource(uri));
        }
        let resources = server.list_resources();
        assert!(
            resources
                .iter()
                .any(|resource| resource.uri == resource_uris.descriptor
                    && resource.mime_type.as_deref() == Some("application/json"))
        );

        let result = server.call_tool(
            &Booking::descriptor().tool_name(),
            Some(serde_json::json!({ "available_on": "2026-06-10" })),
        );

        assert_eq!(result.is_error, Some(false));
        assert_eq!(
            result.structured_content,
            Some(serde_json::json!({ "available_on": "2026-06-10" }))
        );
    }

    #[test]
    fn form_input_pairs_descriptor_schema_with_generated_decode() {
        let tool = McpFormInput::<Booking>::tool_definition().expect("tool should build");
        fn assert_typed_tool(_tool: &super::McpTypedTool<super::McpFormInput<Booking>>) {}
        assert_typed_tool(&tool);
        assert_eq!(
            tool.input_schema["properties"]["available_on"]["type"],
            "string"
        );

        let input = McpFormInput::<Booking>::from_tool_call(
            super::McpToolCall::from_value(Some(serde_json::json!({
                "available_on": "2026-06-10"
            })))
            .expect("arguments should normalize"),
        )
        .expect("form input should decode");

        assert_eq!(input.into_value_holder().available_on, "2026-06-10");
    }

    #[test]
    fn form_resources_describe_generated_form() {
        let descriptor = Booking::descriptor();
        let resource_uris = super::form_resource_uris(descriptor);

        assert_eq!(
            resource_uris.descriptor,
            "gpui-form://forms/reserve_booking/descriptor"
        );
        assert_eq!(
            resource_uris.schema,
            "gpui-form://forms/reserve_booking/schema"
        );
        assert_eq!(
            resource_uris.examples,
            "gpui-form://forms/reserve_booking/examples"
        );

        let descriptor_value = super::form_descriptor_resource_value(descriptor);
        assert_eq!(descriptor_value["form_name"], "Booking");
        assert_eq!(descriptor_value["tool_name"], "reserve_booking");
        assert_eq!(
            descriptor_value["resources"]["schema"],
            "gpui-form://forms/reserve_booking/schema"
        );
        assert_eq!(descriptor_value["fields"][0]["name"], "available_on");
        assert_eq!(descriptor_value["fields"][0]["label"], "Available on");
        assert_eq!(
            descriptor_value["fields"][0]["examples"],
            serde_json::json!(["2026-06-10"])
        );
        assert_eq!(
            descriptor_value["fields"][0]["schema"]["examples"],
            serde_json::json!(["2026-06-10"])
        );

        let schema_value = super::form_schema_resource_value(descriptor);
        assert_eq!(schema_value["properties"]["available_on"]["type"], "string");

        let examples_value = super::form_examples_resource_value(descriptor);
        assert_eq!(
            examples_value["examples"]["available_on"],
            serde_json::json!(["2026-06-10"])
        );
    }

    #[test]
    fn form_prompt_templates_describe_generated_form_workflows() {
        let descriptor = Booking::descriptor();
        let prompt_names = super::form_prompt_names(descriptor);

        assert_eq!(prompt_names.fill, "fill_reserve_booking_form");
        assert_eq!(prompt_names.repair, "repair_reserve_booking_form");
        assert_eq!(prompt_names.submit, "submit_reserve_booking_form");
        assert_eq!(
            prompt_names.all(),
            [
                "fill_reserve_booking_form",
                "repair_reserve_booking_form",
                "submit_reserve_booking_form",
            ]
        );

        let mut server = McpServer::new("example-form-server".to_string(), "9.9.9");
        super::register_form_prompt_templates::<Booking>(&mut server)
            .expect("prompt templates should register");

        assert_eq!(server.resource_count(), 3);
        assert_eq!(server.prompt_count(), 3);
        for prompt_name in prompt_names.all() {
            assert!(server.contains_prompt(prompt_name));
        }

        let prompts = server.list_prompts();
        let fill = prompts
            .iter()
            .find(|prompt| prompt.name == "fill_reserve_booking_form")
            .expect("fill prompt should be listed");
        assert_eq!(fill.title.as_deref(), Some("Fill Reserve booking"));
        assert_eq!(
            fill.description.as_deref(),
            Some("Draft valid direct-submit arguments for Reserve booking.")
        );
        assert_eq!(
            fill.arguments
                .as_ref()
                .expect("fill prompt should expose arguments")[0]
                .name,
            "goal"
        );

        let repair_text = super::form_prompt_text(
            descriptor,
            super::McpFormPromptKind::Repair,
            Some(serde_json::Map::from_iter([(
                "validation_issues".to_string(),
                serde_json::json!([{ "field": "available_on", "message": "required" }]),
            )])),
        );
        assert!(repair_text.contains("gpui-form://forms/reserve_booking/descriptor"));
        assert!(repair_text.contains("reserve_booking_edit_patch"));
        assert!(repair_text.contains("Caller-provided context"));
    }

    #[test]
    fn prompt_first_registration_reuses_form_resources_for_tools() {
        let mut server = McpServer::new("example-form-server".to_string(), "9.9.9");

        super::register_form_prompt_templates::<Booking>(&mut server)
            .expect("prompt templates should register");
        form::<Booking>(&mut server)
            .holder(|holder| {
                Ok::<_, String>(BookingResponse {
                    available_on: holder.available_on,
                })
            })
            .expect("tool should register after prompt resources");

        assert_eq!(server.resource_count(), 3);
        assert_eq!(server.prompt_count(), 3);
        assert!(server.contains_tool("reserve_booking"));
    }

    struct Booking;

    impl super::McpForm for Booking {
        type ValueHolder = BookingHolder;

        fn descriptor() -> McpFormDescriptor {
            const FIELDS: &[McpField] = &[McpField::typed::<String>(
                "available_on",
                RustType::from_macro_tokens_unchecked("String"),
                FieldValuePresence::DirectStorage,
            )
            .with_examples(&["2026-06-10"])];

            McpFormDescriptor::new(
                "Booking",
                RustPath::from_macro_tokens_unchecked("tests"),
                FIELDS,
                false,
                component_shape_mcp::McpToolMetadata::new()
                    .with_name("reserve_booking")
                    .with_title("Reserve booking")
                    .with_description("Reserve a booking window."),
            )
        }

        fn decode_arguments(
            call: super::McpToolCall,
        ) -> Result<Self::ValueHolder, super::McpToolError> {
            let mut arguments = call.into_arguments();
            let available_on = arguments
                .take_present_tool_value::<String>("available_on")?
                .unwrap_or_default();
            arguments.finish()?;
            Ok(BookingHolder { available_on })
        }

        fn validate_holder(_holder: &Self::ValueHolder) -> Result<(), super::McpToolError> {
            Ok(())
        }
    }

    struct BookingHolder {
        available_on: String,
    }

    #[derive(serde::Serialize)]
    struct BookingResponse {
        available_on: String,
    }

    impl super::McpJsonSchema for BookingResponse {
        fn json_schema() -> super::McpSchema {
            let mut properties = super::McpSchemaProperties::new();
            properties.insert(
                "available_on".to_string(),
                <String as super::McpJsonSchema>::json_schema(),
            );
            super::object_schema(properties, ["available_on"])
        }
    }
}
