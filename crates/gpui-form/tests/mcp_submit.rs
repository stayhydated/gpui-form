#![cfg(feature = "mcp")]

use gpui_form::{
    GpuiForm,
    mcp::{
        DEFAULT_EDITOR_SESSION_IDLE_TIMEOUT, DEFAULT_EDITOR_SESSION_LIMIT, McpAny,
        McpEditableForm as _, McpForm as _, McpFormEditorOptions, McpFormRegistrationOptions,
        McpServer, McpToolError, context_submit_tool_definitions, editor_tool_names, form,
        register_context_submitters_strict, register_context_submitters_with_editor_options,
        register_with_options, server as generated_server, tool_definitions,
        tool_definitions_with_options,
    },
};
use koruma_collection::{
    collection::{LenValidation, NonEmptyValidation},
    numeric::RangeValidation,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fmt,
    str::FromStr,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

type Attempts = u32;

#[derive(
    Clone, Debug, Default, Deserialize, gpui_form::mcp::McpJsonSchema, PartialEq, Serialize,
)]
#[serde(transparent)]
pub struct AccountId(u64);

/// Submit a plain request from inferred docs.
#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct PlainRequest {
    /// Title from rustdoc that explicit MCP field metadata overrides.
    #[gpui_form(
        label = "Support title",
        description = "Short title shown to agents and users.",
        example = "Cannot log in"
    )]
    #[gpui_form(hidden)]
    title: String,
    /// Number of retry attempts to allow.
    #[gpui_form(hidden)]
    retries: Option<u32>,
    #[gpui_form(hidden(default = true))]
    enabled: bool,
}

#[derive(Debug, gpui_form::mcp::McpJsonSchema, Serialize)]
struct PlainSubmitResponse {
    title: String,
    enabled: bool,
    source: String,
}

#[derive(Debug, Serialize)]
#[serde(transparent)]
struct ObjectResponse(Value);

impl gpui_form::mcp::McpJsonSchema for ObjectResponse {
    fn json_schema() -> gpui_form::mcp::McpSchema {
        gpui_form::mcp::McpSchema::new(json!({ "type": "object" }))
    }
}

fn object_response(value: Value) -> ObjectResponse {
    assert!(value.is_object(), "test response must be a JSON object");
    ObjectResponse(value)
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(koruma)]
#[gpui_form(mcp)]
pub struct ValidatedRequest {
    #[gpui_form(hidden)]
    #[koruma(NonEmptyValidation::<_>)]
    name: String,
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(koruma)]
#[gpui_form(mcp)]
pub struct ConstrainedRequest {
    #[gpui_form(hidden)]
    #[koruma(LenValidation::<_>::min(2).max(8))]
    title: String,
    #[gpui_form(hidden)]
    #[koruma(RangeValidation::<_>::min(1).max(5).exclusive_max(true))]
    retries: u32,
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct ComponentRequest {
    #[gpui_form(component(gpui_form_collection::input::Input::<String>))]
    title: String,
}

#[derive(
    Clone, Debug, Default, Deserialize, gpui_form::mcp::McpJsonSchema, PartialEq, Serialize,
)]
#[serde(transparent)]
pub struct ComponentOnlyValue(String);

impl fmt::Display for ComponentOnlyValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl FromStr for ComponentOnlyValue {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self(value.to_string()))
    }
}

#[derive(Clone, Debug, Default, Deserialize, gpui_form::mcp::McpJsonSchema, PartialEq)]
#[serde(transparent)]
pub struct DecodeOnlyValue(String);

#[derive(
    Clone, Debug, Default, Deserialize, gpui_form::mcp::McpToolInput, PartialEq, Serialize,
)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPreferences {
    #[mcp(alias = "email")]
    email_updates: bool,
    #[serde(default)]
    topics: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SlashSeparatedTags(Vec<String>);

impl gpui_form::mcp::McpToolValue for SlashSeparatedTags {
    fn tool_value_schema() -> gpui_form::mcp::McpSchema {
        gpui_form::mcp::McpSchema::new(json!({ "type": "string" }))
    }

    fn from_tool_value(field: &str, value: Value) -> Result<Self, McpToolError> {
        let raw = value
            .as_str()
            .ok_or_else(|| McpToolError::decode(field, "expected slash-separated tags"))?;
        Ok(Self(
            raw.split('/')
                .map(str::trim)
                .filter(|tag| !tag.is_empty())
                .map(ToString::to_string)
                .collect(),
        ))
    }
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct ComponentNoSchemaRequest {
    #[gpui_form(component(gpui_form_collection::input::Input::<ComponentOnlyValue>))]
    title: ComponentOnlyValue,
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq)]
#[gpui_form(mcp)]
pub struct DecodeOnlyRequest {
    #[gpui_form(hidden)]
    value: DecodeOnlyValue,
}

#[derive(Clone, Debug, GpuiForm, PartialEq)]
#[gpui_form(mcp)]
pub struct CustomToolValueRequest {
    #[gpui_form(hidden)]
    tags: SlashSeparatedTags,
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct ToolInputSchemaRequest {
    #[gpui_form(hidden)]
    preferences: NotificationPreferences,
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct CustomSchemaRequest {
    #[gpui_form(hidden)]
    account_id: AccountId,
    #[gpui_form(hidden)]
    attempts: Attempts,
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct TupleSchemaRequest {
    #[gpui_form(hidden)]
    coordinates: (u32, u32),
}

#[derive(
    Clone, Debug, Default, Deserialize, gpui_form::mcp::McpJsonSchema, PartialEq, Serialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum Priority {
    #[default]
    Low,
    #[serde(alias = "urgent")]
    HighPriority,
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct EnumSchemaRequest {
    #[gpui_form(hidden)]
    priority: Priority,
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct SkippedRequest {
    #[gpui_form(hidden)]
    name: String,
    #[gpui_form(skip)]
    tenant_id: u64,
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct DirectRequest {
    #[gpui_form(hidden)]
    name: String,
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct AsyncRequest {
    #[gpui_form(hidden)]
    title: String,
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct AsyncValueRequest {
    #[gpui_form(hidden)]
    label: String,
}

#[derive(Clone, Debug)]
pub struct SubmitContext {
    prefix: String,
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp(
    name = "submit_context_request",
    title = "Submit context request",
    description = "Exercise context-backed MCP submit registration.",
    read_only = true,
    destructive = false,
    idempotent = true,
    open_world = false,
    context(SubmitContext),
    response(ContextSubmitResponse)
))]
pub struct ContextRequest {
    #[gpui_form(hidden)]
    title: String,
}

#[derive(Debug, gpui_form::mcp::McpJsonSchema, Serialize)]
pub struct ContextSubmitResponse {
    title: String,
    prefixed: String,
}

impl gpui_form::mcp::McpContextSubmit<SubmitContext> for ContextRequest {
    type Response = ContextSubmitResponse;
    type Error = String;

    fn submit_with_context(
        self,
        context: SubmitContext,
    ) -> impl std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + 'static
    {
        async move {
            Ok(ContextSubmitResponse {
                prefixed: format!("{}{}", context.prefix, self.title),
                title: self.title,
            })
        }
    }
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp(
    name = "submit_dynamic_context_request",
    title = "Submit dynamic context request",
    description = "Exercise context-backed MCP submit registration with a dynamic JSON response.",
    read_only = true,
    destructive = false,
    idempotent = true,
    open_world = false,
    context(SubmitContext),
    submit(submit_dynamic_context_request),
    map_response(dynamic_context_response)
))]
pub struct DynamicContextRequest {
    #[gpui_form(hidden)]
    title: String,
}

async fn submit_dynamic_context_request(
    request: DynamicContextRequest,
    context: SubmitContext,
) -> Result<Value, String> {
    let title = request.title;
    let prefixed = format!("{}{}", context.prefix, title);
    Ok(json!({
        "title": title,
        "prefixed": prefixed
    }))
}

fn dynamic_context_response(value: Value) -> Result<gpui_form::mcp::McpObject, String> {
    gpui_form::mcp::McpObject::from_value(value).map_err(|error| error.to_string())
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp(
    name = "submit_metadata_request",
    title = "Submit metadata request",
    description = "Exercise explicit form MCP metadata.",
    read_only = true,
    destructive = false,
    idempotent = true,
    open_world = false,
    task_support = "optional",
    icon(
        src = "https://example.com/submit-metadata.png",
        mime_type = "image/png",
        size = "48x48",
        theme = "light"
    )
))]
pub struct MetadataRequest {
    #[gpui_form(hidden)]
    label: String,
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct ExplicitModelRequest {
    #[gpui_form(hidden)]
    title: String,
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp(name = "submit_explicit_holder_request"))]
pub struct ExplicitHolderRequest {
    #[gpui_form(skip)]
    tenant_id: u64,

    #[gpui_form(hidden)]
    body: String,
}

#[gpui_form::mcp_submit]
fn submit_plain_request(request: PlainRequest) -> Result<PlainSubmitResponse, String> {
    Ok(PlainSubmitResponse {
        title: request.title,
        enabled: request.enabled,
        source: "attribute".to_string(),
    })
}

#[gpui_form::mcp_submit]
fn submit_skipped_request(holder: SkippedRequestFormValueHolder) -> Result<ObjectResponse, String> {
    Ok(object_response(json!({
        "name": holder.name,
        "tenant": "attribute"
    })))
}

#[gpui_form::mcp_submit]
fn submit_direct_request(request: DirectRequest) -> Result<ObjectResponse, String> {
    Ok(object_response(json!({
        "name": request.name,
        "source": "direct"
    })))
}

#[gpui_form::mcp_submit]
async fn submit_async_request(request: AsyncRequest) -> Result<ObjectResponse, String> {
    Ok(object_response(json!({
        "title": request.title,
        "source": "async"
    })))
}

#[gpui_form::mcp_submit]
async fn submit_async_value_request(request: AsyncValueRequest) -> Result<ObjectResponse, String> {
    Ok(object_response(json!({
        "label": request.label,
        "source": "async-direct"
    })))
}

#[gpui_form::mcp_submit]
fn submit_metadata_request(request: MetadataRequest) -> Result<ObjectResponse, String> {
    Ok(object_response(json!({
        "label": request.label,
        "source": "metadata"
    })))
}

#[gpui_form::mcp_submit]
fn submit_explicit_model_request(request: ExplicitModelRequest) -> Result<ObjectResponse, String> {
    Ok(object_response(json!({
        "title": request.title,
        "mode": "model"
    })))
}

#[gpui_form::mcp_submit]
fn submit_explicit_holder_request(
    holder: ExplicitHolderRequestFormValueHolder,
) -> Result<ObjectResponse, String> {
    Ok(object_response(json!({
        "body": holder.body,
        "mode": "holder"
    })))
}

fn test_server() -> McpServer {
    McpServer::new("gpui-form-mcp-test", "0.0.0")
}

#[test]
fn generated_mcp_schema_marks_required_optional_default_and_component_fields() {
    let schema = PlainRequest::descriptor().input_schema();

    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["properties"]["title"]["type"], "string");
    assert_eq!(
        schema["properties"]["retries"]["anyOf"][0]["type"],
        "integer"
    );
    assert_eq!(schema["properties"]["enabled"]["type"], "boolean");
    assert_eq!(
        schema["properties"]["enabled"]["x-gpuiFormHasDefault"],
        true
    );
    assert_eq!(schema["required"], json!(["title"]));

    let component_schema = ComponentRequest::descriptor().input_schema();
    assert_eq!(
        component_schema["properties"]["title"]["x-gpuiFormComponentBacked"],
        true
    );

    let custom_schema = CustomSchemaRequest::descriptor().input_schema();
    assert_eq!(custom_schema["properties"]["account_id"]["type"], "integer");
    assert_eq!(custom_schema["properties"]["attempts"]["type"], "integer");
    assert_eq!(custom_schema["required"], json!(["account_id", "attempts"]));

    let tool_input_schema = ToolInputSchemaRequest::descriptor().input_schema();
    assert_eq!(
        tool_input_schema["properties"]["preferences"]["type"],
        "object"
    );
    assert_eq!(
        tool_input_schema["properties"]["preferences"]["properties"]["emailUpdates"]["type"],
        "boolean"
    );
    assert_eq!(
        tool_input_schema["properties"]["preferences"]["properties"]["emailUpdates"]["x-mcpAliases"],
        json!(["email"])
    );

    let tuple_schema = TupleSchemaRequest::descriptor().input_schema();
    assert_eq!(
        tuple_schema["properties"]["coordinates"]["prefixItems"][0]["type"],
        "integer"
    );
    assert_eq!(
        tuple_schema["properties"]["coordinates"]["prefixItems"][1]["type"],
        "integer"
    );
    assert_eq!(tuple_schema["properties"]["coordinates"]["minItems"], 2);

    let enum_schema = EnumSchemaRequest::descriptor().input_schema();
    assert_eq!(enum_schema["properties"]["priority"]["type"], "string");
    assert_eq!(
        enum_schema["properties"]["priority"]["enum"],
        json!(["low", "high-priority", "urgent"])
    );
}

#[test]
fn generated_mcp_schema_publishes_koruma_validation_metadata_and_constraints() {
    let non_empty_schema = ValidatedRequest::descriptor().input_schema();
    let name_schema = &non_empty_schema["properties"]["name"];
    assert_eq!(name_schema["minLength"], json!(1));
    assert_eq!(
        name_schema["x-gpuiFormValidation"],
        json!([{
            "scope": "field",
            "validator": "NonEmptyValidation",
            "path": "NonEmptyValidation",
            "target": "default",
            "type_arg_mode": "infer"
        }])
    );

    let constrained_schema = ConstrainedRequest::descriptor().input_schema();
    let title_schema = &constrained_schema["properties"]["title"];
    assert_eq!(title_schema["minLength"], json!(2));
    assert_eq!(title_schema["maxLength"], json!(8));
    assert_eq!(
        title_schema["x-gpuiFormValidation"][0]["params"],
        json!([
            { "name": "min", "value": "2" },
            { "name": "max", "value": "8" }
        ])
    );

    let retries_schema = &constrained_schema["properties"]["retries"];
    assert_eq!(retries_schema["minimum"], json!(1));
    assert_eq!(retries_schema["exclusiveMaximum"], json!(5));
    assert_eq!(
        retries_schema["x-gpuiFormValidation"][0]["params"],
        json!([
            { "name": "min", "value": "1" },
            { "name": "max", "value": "5" },
            { "name": "exclusive_max", "value": "true" }
        ])
    );
}

#[test]
fn generated_mcp_tool_definitions_include_registered_handlers_and_editors() {
    let tools = tool_definitions().expect("tool definitions should be generated");
    let registered_names: Vec<_> = tools.iter().map(|tool| tool.name.to_string()).collect();

    assert!(registered_names.contains(&PlainRequest::descriptor().tool_name()));
    assert!(registered_names.contains(&DirectRequest::descriptor().tool_name()));
    assert!(registered_names.contains(&AsyncRequest::descriptor().tool_name()));
    assert!(registered_names.contains(&AsyncValueRequest::descriptor().tool_name()));
    assert!(registered_names.contains(&MetadataRequest::descriptor().tool_name()));
    assert!(registered_names.contains(&ExplicitModelRequest::descriptor().tool_name()));
    assert!(registered_names.contains(&ExplicitHolderRequest::descriptor().tool_name()));
    assert!(!registered_names.contains(&ComponentRequest::descriptor().tool_name()));

    let plain_editor_names = editor_tool_names(PlainRequest::descriptor());
    assert!(registered_names.contains(&plain_editor_names.open));
    assert!(registered_names.contains(&plain_editor_names.list));
    assert!(registered_names.contains(&plain_editor_names.patch));
    assert!(registered_names.contains(&plain_editor_names.close_all));
    assert!(registered_names.contains(&plain_editor_names.submit));

    let metadata_submit = tools
        .iter()
        .find(|tool| tool.name.as_ref() == MetadataRequest::descriptor().tool_name())
        .expect("metadata submit tool should be registered");
    let metadata_annotations = metadata_submit
        .annotations
        .as_ref()
        .expect("explicit mcp hints should publish tool annotations");
    assert_eq!(
        metadata_annotations.title.as_deref(),
        Some("Submit metadata request")
    );
    assert_eq!(metadata_annotations.read_only_hint, Some(true));
    assert_eq!(metadata_annotations.destructive_hint, Some(false));
    assert_eq!(metadata_annotations.idempotent_hint, Some(true));
    assert_eq!(metadata_annotations.open_world_hint, Some(false));
    let metadata_icons = metadata_submit
        .icons
        .as_ref()
        .expect("explicit mcp icons should publish tool icons");
    assert_eq!(
        metadata_icons[0].src,
        "https://example.com/submit-metadata.png"
    );
    assert_eq!(metadata_icons[0].mime_type.as_deref(), Some("image/png"));
    assert_eq!(
        metadata_icons[0]
            .sizes
            .as_ref()
            .expect("icon sizes should be published")[0],
        "48x48"
    );
    assert_eq!(
        metadata_icons[0].theme,
        Some(gpui_form::mcp::McpIconTheme::Light)
    );
    assert_eq!(
        metadata_submit
            .execution
            .as_ref()
            .and_then(|execution| execution.task_support),
        Some(gpui_form::mcp::McpToolTaskSupport::Optional)
    );

    let component_editor_names = editor_tool_names(ComponentRequest::descriptor());
    assert!(registered_names.contains(&component_editor_names.open));
    assert!(registered_names.contains(&component_editor_names.list));
    assert!(registered_names.contains(&component_editor_names.patch));
    assert!(registered_names.contains(&component_editor_names.close_all));
    assert!(!registered_names.contains(&component_editor_names.submit));

    let plain_submit = tools
        .iter()
        .find(|tool| tool.name.as_ref() == PlainRequest::descriptor().tool_name())
        .expect("plain submit tool should be registered");
    let plain_submit_annotations = plain_submit
        .annotations
        .as_ref()
        .expect("plain submit tool should publish default annotations");
    assert_eq!(plain_submit_annotations.read_only_hint, Some(false));
    assert_eq!(plain_submit_annotations.destructive_hint, Some(true));
    assert_eq!(plain_submit_annotations.idempotent_hint, Some(false));
    assert_eq!(plain_submit_annotations.open_world_hint, Some(true));
    let plain_submit_output = Value::Object(
        plain_submit
            .output_schema
            .as_ref()
            .expect("submit tool should publish an output schema")
            .as_ref()
            .clone(),
    );
    assert_eq!(plain_submit_output["type"], json!("object"));
    assert_eq!(
        plain_submit_output["properties"]["title"]["type"],
        json!("string")
    );
    assert_eq!(
        plain_submit_output["properties"]["enabled"]["type"],
        json!("boolean")
    );
    assert_eq!(
        plain_submit_output["properties"]["source"]["type"],
        json!("string")
    );

    let plain_editor_submit = tools
        .iter()
        .find(|tool| tool.name.as_ref() == plain_editor_names.submit.as_str())
        .expect("plain editor submit tool should be registered");
    assert_eq!(
        plain_editor_submit.input_schema["properties"]["close_on_success"]["default"],
        json!(true)
    );
    assert_eq!(
        plain_editor_submit.input_schema["properties"]["expected_revision"]["type"],
        json!("integer")
    );
    let plain_editor_submit_output = Value::Object(
        plain_editor_submit
            .output_schema
            .as_ref()
            .expect("editor submit should publish an output schema")
            .as_ref()
            .clone(),
    );
    assert_eq!(plain_editor_submit_output, plain_submit_output);
    let plain_editor_submit_annotations = plain_editor_submit
        .annotations
        .as_ref()
        .expect("editor submit should publish tool annotations");
    assert_eq!(plain_editor_submit_annotations.read_only_hint, Some(false));
    assert_eq!(plain_editor_submit_annotations.destructive_hint, Some(true));
    assert_eq!(plain_editor_submit_annotations.idempotent_hint, Some(false));
    assert_eq!(plain_editor_submit_annotations.open_world_hint, Some(true));

    let plain_open = tools
        .iter()
        .find(|tool| tool.name.as_ref() == plain_editor_names.open.as_str())
        .expect("plain editor open tool should be registered");
    let plain_open_annotations = plain_open
        .annotations
        .as_ref()
        .expect("editor open should publish tool annotations");
    assert_eq!(plain_open_annotations.read_only_hint, Some(false));
    assert_eq!(plain_open_annotations.destructive_hint, Some(false));
    assert_eq!(plain_open_annotations.idempotent_hint, Some(false));
    assert_eq!(plain_open_annotations.open_world_hint, Some(false));
    let plain_open_output = Value::Object(
        plain_open
            .output_schema
            .as_ref()
            .expect("editor open should publish an output schema")
            .as_ref()
            .clone(),
    );
    assert_eq!(
        plain_open_output["properties"]["valid"]["type"],
        json!("boolean")
    );
    assert_eq!(
        plain_open_output["properties"]["revision"]["type"],
        json!("integer")
    );
    assert_eq!(
        plain_open_output["properties"]["revision"]["minimum"],
        json!(0)
    );
    assert_eq!(
        plain_open_output["properties"]["present_fields"]["items"]["enum"],
        json!(["title", "retries", "enabled"])
    );
    assert_eq!(
        plain_open_output["properties"]["fields"]["items"]["oneOf"][0]["properties"]["name"]["enum"],
        json!(["title"])
    );
    assert_eq!(
        plain_open_output["properties"]["fields"]["items"]["oneOf"][0]["properties"]["value"]["anyOf"]
            [0]["type"],
        json!("string")
    );
    assert_eq!(
        plain_open_output["properties"]["fields"]["items"]["oneOf"][0]["properties"]["value"]["anyOf"]
            [1]["type"],
        json!("null")
    );
    assert_eq!(
        plain_open_output["properties"]["fields"]["items"]["oneOf"][0]["properties"]["schema"]["const"]
            ["type"],
        json!("string")
    );
    assert_eq!(
        plain_open_output["properties"]["fields"]["items"]["oneOf"][0]["properties"]["has_value"]["type"],
        json!("boolean")
    );
    assert_eq!(
        plain_open_output["properties"]["fields"]["items"]["oneOf"][0]["properties"]["errors"]["items"]
            ["type"],
        json!("object")
    );
    assert_eq!(
        plain_open_output["properties"]["fields"]["items"]["oneOf"][0]["properties"]["errors"]["items"]
            ["properties"]["scope"]["enum"],
        json!(["form", "field", "element"])
    );
    assert_eq!(
        plain_open_output["properties"]["fields"]["items"]["oneOf"][2]["properties"]["has_default"]
            ["const"],
        json!(true)
    );
    assert_eq!(
        plain_open_output["properties"]["submit_arguments"]["properties"]["title"]["type"],
        json!("string")
    );
    assert_eq!(
        plain_open_output["properties"]["cleanup"]["properties"]["expired_count"]["type"],
        json!("integer")
    );
    assert_eq!(
        plain_open_output["properties"]["cleanup"]["properties"]["evicted_session_ids"]["items"]["type"],
        json!("string")
    );

    let plain_list = tools
        .iter()
        .find(|tool| tool.name.as_ref() == plain_editor_names.list.as_str())
        .expect("plain editor list tool should be registered");
    let plain_list_annotations = plain_list
        .annotations
        .as_ref()
        .expect("editor list should publish tool annotations");
    assert_eq!(plain_list_annotations.read_only_hint, Some(true));
    assert_eq!(plain_list_annotations.destructive_hint, Some(false));
    assert_eq!(plain_list_annotations.idempotent_hint, Some(true));
    assert_eq!(plain_list_annotations.open_world_hint, Some(false));
    let plain_list_output = Value::Object(
        plain_list
            .output_schema
            .as_ref()
            .expect("editor list should publish an output schema")
            .as_ref()
            .clone(),
    );
    assert_eq!(
        plain_list_output["properties"]["session_limit"]["anyOf"][0]["type"],
        json!("integer")
    );
    assert_eq!(
        plain_list_output["properties"]["session_limit"]["anyOf"][0]["minimum"],
        json!(1)
    );
    assert_eq!(
        plain_list_output["properties"]["session_limit"]["anyOf"][1]["type"],
        json!("null")
    );
    assert_eq!(
        plain_list_output["properties"]["session_idle_timeout_ms"]["anyOf"][0]["type"],
        json!("integer")
    );
    assert_eq!(
        plain_list_output["properties"]["session_idle_timeout_ms"]["anyOf"][0]["minimum"],
        json!(0)
    );
    assert_eq!(
        plain_list_output["properties"]["session_idle_timeout_ms"]["anyOf"][1]["type"],
        json!("null")
    );
    assert_eq!(
        plain_list_output["properties"]["session_count"]["type"],
        json!("integer")
    );
    assert_eq!(
        plain_list_output["properties"]["cleanup"]["properties"]["evicted_count"]["type"],
        json!("integer")
    );
    assert_eq!(
        plain_list_output["properties"]["sessions"]["items"]["properties"]["revision"]["type"],
        json!("integer")
    );
    assert_eq!(
        plain_list_output["properties"]["sessions"]["items"]["properties"]["fields"]["items"]["oneOf"]
            [0]["properties"]["name"]["enum"],
        json!(["title"])
    );

    let plain_patch = tools
        .iter()
        .find(|tool| tool.name.as_ref() == plain_editor_names.patch.as_str())
        .expect("plain editor patch tool should be registered");
    let plain_patch_annotations = plain_patch
        .annotations
        .as_ref()
        .expect("editor patch should publish tool annotations");
    assert_eq!(plain_patch_annotations.read_only_hint, Some(false));
    assert_eq!(plain_patch_annotations.destructive_hint, Some(true));
    assert_eq!(plain_patch_annotations.idempotent_hint, Some(false));
    assert_eq!(plain_patch_annotations.open_world_hint, Some(false));
    assert_eq!(
        plain_patch.input_schema["properties"]["expected_revision"]["type"],
        json!("integer")
    );

    let plain_close = tools
        .iter()
        .find(|tool| tool.name.as_ref() == plain_editor_names.close.as_str())
        .expect("plain editor close tool should be registered");
    let plain_close_annotations = plain_close
        .annotations
        .as_ref()
        .expect("editor close should publish tool annotations");
    assert_eq!(plain_close_annotations.read_only_hint, Some(false));
    assert_eq!(plain_close_annotations.destructive_hint, Some(true));
    assert_eq!(plain_close_annotations.idempotent_hint, Some(false));
    assert_eq!(plain_close_annotations.open_world_hint, Some(false));
    assert_eq!(
        plain_close.input_schema["properties"]["expected_revision"]["type"],
        json!("integer")
    );
    let plain_close_output = Value::Object(
        plain_close
            .output_schema
            .as_ref()
            .expect("editor close should publish an output schema")
            .as_ref()
            .clone(),
    );
    assert_eq!(
        plain_close_output["properties"]["closed"]["type"],
        json!("boolean")
    );
    assert_eq!(
        plain_close_output["required"],
        json!(["session_id", "closed"])
    );

    let plain_close_all = tools
        .iter()
        .find(|tool| tool.name.as_ref() == plain_editor_names.close_all.as_str())
        .expect("plain editor close_all tool should be registered");
    let plain_close_all_annotations = plain_close_all
        .annotations
        .as_ref()
        .expect("editor close_all should publish tool annotations");
    assert_eq!(plain_close_all_annotations.read_only_hint, Some(false));
    assert_eq!(plain_close_all_annotations.destructive_hint, Some(true));
    assert_eq!(plain_close_all_annotations.idempotent_hint, Some(false));
    assert_eq!(plain_close_all_annotations.open_world_hint, Some(false));
    let plain_close_all_output = Value::Object(
        plain_close_all
            .output_schema
            .as_ref()
            .expect("editor close_all should publish an output schema")
            .as_ref()
            .clone(),
    );
    assert_eq!(
        plain_close_all_output["properties"]["closed_count"]["type"],
        json!("integer")
    );
    assert_eq!(
        plain_close_all_output["properties"]["form"]["const"],
        json!(PlainRequest::descriptor().tool_name())
    );
    assert_eq!(
        plain_close_all_output["properties"]["form_name"]["const"],
        json!("PlainRequest")
    );
    assert_eq!(
        plain_close_all_output["properties"]["session_ids"]["items"]["type"],
        json!("string")
    );
    assert_eq!(
        plain_close_all_output["required"],
        json!(["form", "form_name", "closed_count", "session_ids"])
    );
}

#[test]
fn generated_mcp_descriptor_uses_explicit_metadata() {
    let descriptor = MetadataRequest::descriptor();

    assert_eq!(descriptor.tool_name(), "submit_metadata_request");
    assert_eq!(descriptor.title(), "Submit metadata request");
    assert_eq!(
        descriptor.description(),
        "Exercise explicit form MCP metadata."
    );
    let annotations = descriptor
        .tool_annotations()
        .expect("explicit mcp hints should publish annotations");
    assert_eq!(annotations.read_only_hint, Some(true));
    assert_eq!(annotations.destructive_hint, Some(false));
    assert_eq!(annotations.idempotent_hint, Some(true));
    assert_eq!(annotations.open_world_hint, Some(false));
    let icons = descriptor
        .tool_icons()
        .expect("explicit icon metadata should publish icons");
    assert_eq!(icons[0].src, "https://example.com/submit-metadata.png");
    assert_eq!(icons[0].mime_type.as_deref(), Some("image/png"));
    assert_eq!(
        descriptor
            .tool_execution()
            .and_then(|execution| execution.task_support),
        Some(gpui_form::mcp::McpToolTaskSupport::Optional)
    );
}

#[test]
fn generated_mcp_descriptor_infers_doc_description() {
    assert_eq!(
        PlainRequest::descriptor().description(),
        "Submit a plain request from inferred docs."
    );
}

#[test]
fn generated_mcp_fields_publish_inferred_and_explicit_metadata() {
    let descriptor = PlainRequest::descriptor();
    let fields = descriptor.fields();
    let title = fields
        .iter()
        .find(|field| field.name() == "title")
        .expect("title field should be described");
    let retries = fields
        .iter()
        .find(|field| field.name() == "retries")
        .expect("retries field should be described");

    assert_eq!(title.explicit_label(), Some("Support title"));
    assert_eq!(title.label(), "Support title");
    assert_eq!(
        title.description(),
        Some("Short title shown to agents and users.")
    );
    assert_eq!(title.examples(), &["Cannot log in"]);
    assert_eq!(retries.explicit_label(), None);
    assert_eq!(retries.label(), "Retries");
    assert_eq!(
        retries.description(),
        Some("Number of retry attempts to allow.")
    );

    let schema = descriptor.input_schema();
    assert_eq!(schema["properties"]["title"]["title"], "Support title");
    assert_eq!(
        schema["properties"]["title"]["description"],
        "Short title shown to agents and users."
    );
    assert_eq!(
        schema["properties"]["title"]["x-gpuiFormExplicitLabel"],
        "Support title"
    );
    assert_eq!(
        schema["properties"]["title"]["examples"],
        json!(["Cannot log in"])
    );
    assert_eq!(schema["properties"]["retries"]["title"], "Retries");
    assert_eq!(
        schema["properties"]["retries"]["description"],
        "Number of retry attempts to allow."
    );

    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .editor()
        .expect("editor tools should register");
    let tools = editor_tool_names(descriptor);
    let opened = server.call_tool(&tools.open, Some(json!({})));
    assert_eq!(opened.is_error, Some(false));
    let opened = opened
        .structured_content
        .expect("open should return a session");
    assert_eq!(opened["fields"][0]["label"], "Support title");
    assert_eq!(
        opened["fields"][0]["description"],
        "Short title shown to agents and users."
    );
    assert_eq!(opened["fields"][0]["examples"], json!(["Cannot log in"]));
    assert_eq!(opened["fields"][1]["label"], "Retries");
    assert_eq!(
        opened["fields"][1]["description"],
        "Number of retry attempts to allow."
    );
}

#[test]
fn generated_registration_profiles_select_tools_and_report_skips() {
    let plain_submit = PlainRequest::descriptor().tool_name();
    let plain_editor = editor_tool_names(PlainRequest::descriptor());

    let submit_only_definitions =
        tool_definitions_with_options(McpFormRegistrationOptions::submit_only())
            .expect("submit-only definitions should build");
    let submit_only_names: Vec<_> = submit_only_definitions
        .iter()
        .map(|tool| tool.name.to_string())
        .collect();
    assert!(submit_only_names.contains(&plain_submit));
    assert!(!submit_only_names.contains(&plain_editor.open));
    assert!(!submit_only_names.contains(&plain_editor.submit));

    let mut submit_only_server = test_server();
    let submit_only_report = register_with_options(
        &mut submit_only_server,
        McpFormRegistrationOptions::submit_only(),
    )
    .expect("submit-only profile should register");
    assert!(submit_only_server.contains_tool(&plain_submit));
    assert!(!submit_only_server.contains_tool(&plain_editor.open));
    assert_eq!(submit_only_report.registered_tool_names, submit_only_names);
    assert!(
        submit_only_report.skipped_tools.iter().any(
            |skipped| skipped.name == plain_editor.open && skipped.reason == "editors_disabled"
        )
    );
    assert!(
        submit_only_report
            .skipped_tools
            .iter()
            .any(|skipped| skipped.name == plain_editor.submit
                && skipped.reason == "edit_submit_disabled")
    );

    let mut editor_only_server = test_server();
    let editor_only_report = register_with_options(
        &mut editor_only_server,
        McpFormRegistrationOptions::editor_only(),
    )
    .expect("editor-only profile should register");
    assert!(!editor_only_server.contains_tool(&plain_submit));
    assert!(editor_only_server.contains_tool(&plain_editor.open));
    assert!(!editor_only_server.contains_tool(&plain_editor.submit));
    assert!(editor_only_report.skipped_tools.iter().any(
        |skipped| skipped.name == plain_submit && skipped.reason == "submit_handlers_disabled"
    ));
    assert!(
        editor_only_report
            .skipped_tools
            .iter()
            .any(|skipped| skipped.name == plain_editor.submit
                && skipped.reason == "edit_submit_disabled")
    );

    let mut metadata_server = test_server();
    let metadata_report = register_with_options(
        &mut metadata_server,
        McpFormRegistrationOptions::metadata_only(),
    )
    .expect("metadata-only profile should build report");
    assert_eq!(metadata_server.tool_count(), 0);
    assert!(metadata_report.registered_tool_names.is_empty());
    assert!(!metadata_report.tool_definitions.is_empty());
    assert_eq!(
        metadata_report.skipped_tools.len(),
        metadata_report.tool_definitions.len()
    );
    assert!(
        metadata_report
            .skipped_tools
            .iter()
            .all(|skipped| skipped.reason == "metadata_only")
    );
}

#[test]
fn tool_call_decodes_validates_converts_and_submits_model() {
    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .model(|request| {
            Ok::<ObjectResponse, String>(object_response(json!({
                "title": request.title,
                "retries": request.retries,
                "enabled": request.enabled
            })))
        })
        .expect("tool should register");

    let result = server.call_tool(
        &PlainRequest::descriptor().tool_name(),
        Some(json!({
            "title": "Ship it",
            "retries": 2
        })),
    );

    assert_eq!(result.is_error, Some(false));
    assert_eq!(
        result.structured_content,
        Some(json!({
            "title": "Ship it",
            "retries": 2,
            "enabled": true
        }))
    );
}

#[test]
fn form_tool_can_register_model_submit_with_editor_tools() {
    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .model_with_editor(|request| {
            Ok::<ObjectResponse, String>(object_response(json!({
                "title": request.title,
                "mode": "submit"
            })))
        })
        .expect("submit and editor tools should register");

    let result = server.call_tool(
        &PlainRequest::descriptor().tool_name(),
        Some(json!({ "title": "Ship it" })),
    );
    assert_eq!(result.is_error, Some(false));
    assert_eq!(
        result.structured_content,
        Some(json!({
            "title": "Ship it",
            "mode": "submit"
        }))
    );

    let tools = editor_tool_names(PlainRequest::descriptor());
    assert!(server.contains_tool(&tools.submit));
    let open = server.call_tool(&tools.open, Some(json!({ "values": { "title": "Draft" } })));
    assert_eq!(open.is_error, Some(false));
    let opened = open
        .structured_content
        .expect("open should return editor state");
    assert_eq!(opened["submit_arguments"], json!({ "title": "Draft" }));
    assert_eq!(opened["revision"], json!(0));
    let session_id = opened["session_id"]
        .as_str()
        .expect("session id should be a string");

    let session_submit = server.call_tool(
        &tools.submit,
        Some(json!({
            "session_id": session_id,
            "expected_revision": 0
        })),
    );
    assert_eq!(session_submit.is_error, Some(false));
    assert_eq!(
        session_submit.structured_content,
        Some(json!({
            "title": "Draft",
            "mode": "submit"
        }))
    );
    let read_after_submit = server.call_tool(
        &tools.read,
        Some(json!({
            "session_id": session_id
        })),
    );
    assert_eq!(read_after_submit.is_error, Some(true));

    let invalid_open = server.call_tool(&tools.open, Some(json!({})));
    assert_eq!(invalid_open.is_error, Some(false));
    let invalid_open = invalid_open
        .structured_content
        .expect("invalid open should return editor state");
    let invalid_session_id = invalid_open["session_id"]
        .as_str()
        .expect("session id should be a string");
    let invalid_submit = server.call_tool(
        &tools.submit,
        Some(json!({
            "session_id": invalid_session_id
        })),
    );
    assert_eq!(invalid_submit.is_error, Some(true));
    assert_eq!(
        invalid_submit.structured_content.expect("structured error")["error"],
        json!({
            "kind": "validation",
            "message": "validation failed: missing required field `title`",
            "detail": "missing required field `title`",
            "details": [{
                "scope": "field",
                "message": "missing required field `title`",
                "field": "title",
                "validator": "required"
            }]
        })
    );
    let invalid_read = server.call_tool(
        &tools.read,
        Some(json!({
            "session_id": invalid_session_id
        })),
    );
    assert_eq!(invalid_read.is_error, Some(false));
    assert_eq!(
        invalid_read
            .structured_content
            .as_ref()
            .expect("failed submit should leave session open")["valid"],
        json!(false)
    );

    let persistent_open = server.call_tool(
        &tools.open,
        Some(json!({ "values": { "title": "Keep editing" } })),
    );
    assert_eq!(persistent_open.is_error, Some(false));
    let persistent_open = persistent_open
        .structured_content
        .expect("persistent session should open");
    let persistent_session_id = persistent_open["session_id"]
        .as_str()
        .expect("persistent session id should be a string");
    assert_eq!(persistent_open["revision"], json!(0));
    let persistent_submit = server.call_tool(
        &tools.submit,
        Some(json!({
            "session_id": persistent_session_id,
            "close_on_success": false,
            "expected_revision": 0
        })),
    );
    assert_eq!(persistent_submit.is_error, Some(false));
    let persistent_read = server.call_tool(
        &tools.read,
        Some(json!({
            "session_id": persistent_session_id
        })),
    );
    assert_eq!(persistent_read.is_error, Some(false));
    assert_eq!(
        persistent_read
            .structured_content
            .as_ref()
            .expect("close_on_success=false should leave session open")["submit_arguments"],
        json!({ "title": "Keep editing" })
    );
}

#[test]
fn context_submitters_register_from_derive_inventory() {
    let context = SubmitContext {
        prefix: "ctx:".to_string(),
    };
    let definitions =
        context_submit_tool_definitions::<SubmitContext>().expect("definitions should build");
    let definition_names: Vec<_> = definitions
        .iter()
        .map(|definition| definition.name.to_string())
        .collect();
    let submit_name = ContextRequest::descriptor().tool_name();
    let editor_names = editor_tool_names(ContextRequest::descriptor());
    let dynamic_submit_name = DynamicContextRequest::descriptor().tool_name();
    assert!(definition_names.contains(&submit_name));
    assert!(definition_names.contains(&editor_names.open));
    assert!(definition_names.contains(&editor_names.submit));
    assert!(definition_names.contains(&dynamic_submit_name));

    let submit_definition = definitions
        .iter()
        .find(|definition| definition.name.as_ref() == submit_name)
        .expect("context submit definition should be present");
    let output_schema = Value::Object(
        submit_definition
            .output_schema
            .as_ref()
            .expect("context submit should publish an output schema")
            .as_ref()
            .clone(),
    );
    assert_eq!(
        output_schema["properties"]["prefixed"]["type"],
        json!("string")
    );
    let dynamic_submit_definition = definitions
        .iter()
        .find(|definition| definition.name.as_ref() == dynamic_submit_name)
        .expect("dynamic context submit definition should be present");
    let dynamic_output_schema = Value::Object(
        dynamic_submit_definition
            .output_schema
            .as_ref()
            .expect("dynamic context submit should publish an output schema")
            .as_ref()
            .clone(),
    );
    assert_eq!(dynamic_output_schema["type"], json!("object"));

    let mut server = test_server();
    register_context_submitters_with_editor_options(
        &mut server,
        context,
        McpFormEditorOptions::default().with_session_limit(4),
    )
    .expect("context submitters should register");

    assert!(server.contains_tool(&submit_name));
    assert!(server.contains_tool(&editor_names.open));
    assert!(server.contains_tool(&editor_names.submit));
    assert!(server.contains_tool(&dynamic_submit_name));

    let direct_submit = server.call_tool(&submit_name, Some(json!({ "title": "Direct" })));
    assert_eq!(direct_submit.is_error, Some(false));
    assert_eq!(
        direct_submit.structured_content,
        Some(json!({
            "title": "Direct",
            "prefixed": "ctx:Direct"
        }))
    );
    let dynamic_submit =
        server.call_tool(&dynamic_submit_name, Some(json!({ "title": "Dynamic" })));
    assert_eq!(dynamic_submit.is_error, Some(false));
    assert_eq!(
        dynamic_submit.structured_content,
        Some(json!({
            "title": "Dynamic",
            "prefixed": "ctx:Dynamic"
        }))
    );

    let opened = server.call_tool(
        &editor_names.open,
        Some(json!({ "values": { "title": "Editable" } })),
    );
    assert_eq!(opened.is_error, Some(false));
    let opened = opened
        .structured_content
        .expect("context edit open should return state");
    assert_eq!(opened["submit_arguments"], json!({ "title": "Editable" }));
    let session_id = opened["session_id"]
        .as_str()
        .expect("session id should be a string");

    let edit_submit = server.call_tool(
        &editor_names.submit,
        Some(json!({
            "session_id": session_id,
            "expected_revision": 0
        })),
    );
    assert_eq!(edit_submit.is_error, Some(false));
    assert_eq!(
        edit_submit.structured_content,
        Some(json!({
            "title": "Editable",
            "prefixed": "ctx:Editable"
        }))
    );
}

#[test]
fn strict_context_submitters_report_matches_and_reject_zero_matches() {
    #[derive(Clone)]
    struct MissingContext;

    let mut missing_server = test_server();
    let error = register_context_submitters_strict(&mut missing_server, MissingContext)
        .expect_err("strict context registration should reject zero matches");
    assert_eq!(error.kind(), "validation");
    assert!(
        error
            .to_string()
            .contains("no MCP context submitters registered for context")
    );
    assert_eq!(missing_server.tool_count(), 0);

    let mut server = test_server();
    let report = register_context_submitters_strict(
        &mut server,
        SubmitContext {
            prefix: "strict:".to_string(),
        },
    )
    .expect("strict context submitters should register matching inventory");

    let submit_name = ContextRequest::descriptor().tool_name();
    let editor_names = editor_tool_names(ContextRequest::descriptor());
    assert_eq!(report.context_registration_count, 2);
    assert!(report.registered_tool_names.contains(&submit_name));
    assert!(report.registered_tool_names.contains(&editor_names.submit));
    assert!(server.contains_tool(&submit_name));
    assert!(server.contains_tool(&editor_names.submit));
}

#[test]
fn form_tool_with_editor_preflights_editor_name_collisions() {
    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .editor()
        .expect("editor tools should register");

    let error = form::<PlainRequest>(&mut server)
        .model_with_editor(|_| Ok::<ObjectResponse, String>(object_response(json!({}))))
        .expect_err("duplicate editor tools should reject combined registration");

    let tools = editor_tool_names(PlainRequest::descriptor());
    assert_eq!(error, McpToolError::duplicate_tool(tools.open));
    assert!(!server.contains_tool(&PlainRequest::descriptor().tool_name()));
}

#[test]
fn async_editor_submit_does_not_close_session_if_revision_changes_before_success() {
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Arc::new(Mutex::new(release_rx));

    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .model_async_with_editor({
            let release_rx = Arc::clone(&release_rx);
            move |request| {
                let started_tx = started_tx.clone();
                let release_rx = Arc::clone(&release_rx);
                async move {
                    started_tx.send(()).expect("submit should signal start");
                    release_rx
                        .lock()
                        .expect("release receiver should lock")
                        .recv()
                        .expect("submit should be released");
                    Ok::<ObjectResponse, String>(object_response(json!({
                        "title": request.title,
                        "mode": "async-submit"
                    })))
                }
            }
        })
        .expect("async submit and editor tools should register");

    let tools = editor_tool_names(PlainRequest::descriptor());
    let open = server.call_tool(
        &tools.open,
        Some(json!({ "values": { "title": "Initial title" } })),
    );
    assert_eq!(open.is_error, Some(false));
    let opened = open.structured_content.expect("open should return state");
    assert_eq!(opened["revision"], json!(0));
    let session_id = opened["session_id"]
        .as_str()
        .expect("session id should be a string")
        .to_string();

    let submit_server = server.clone();
    let submit_tool = tools.submit.clone();
    let submit_session_id = session_id.clone();
    let submit_handle = std::thread::spawn(move || {
        submit_server.call_tool(
            &submit_tool,
            Some(json!({
                "session_id": submit_session_id,
                "expected_revision": 0
            })),
        )
    });

    started_rx.recv().expect("submit should start");

    let patched = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id.clone(),
            "expected_revision": 0,
            "values": {
                "title": "Updated while submit runs"
            }
        })),
    );
    assert_eq!(patched.is_error, Some(false));
    let patched = patched
        .structured_content
        .expect("patch should return state");
    assert_eq!(patched["revision"], json!(1));
    assert_eq!(
        patched["submit_arguments"],
        json!({ "title": "Updated while submit runs" })
    );

    release_tx.send(()).expect("submit should release");
    let submit = submit_handle.join().expect("submit thread should join");
    assert_eq!(submit.is_error, Some(false));
    assert_eq!(
        submit.structured_content,
        Some(json!({
            "title": "Initial title",
            "mode": "async-submit"
        }))
    );

    let read = server.call_tool(
        &tools.read,
        Some(json!({
            "session_id": session_id.clone()
        })),
    );
    assert_eq!(read.is_error, Some(false));
    let read = read
        .structured_content
        .expect("newer revision should remain open");
    assert_eq!(read["revision"], json!(1));
    assert_eq!(
        read["submit_arguments"],
        json!({ "title": "Updated while submit runs" })
    );
}

#[test]
fn tool_call_reports_missing_values_as_tool_errors() {
    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .model(|_| Ok::<ObjectResponse, String>(object_response(json!({}))))
        .expect("tool should register");

    let result = server.call_tool(&PlainRequest::descriptor().tool_name(), Some(json!({})));

    assert_eq!(result.is_error, Some(true));
    assert!(
        result.content[0]
            .as_text()
            .is_some_and(|text| text.text.contains("missing required field `title`"))
    );
}

#[test]
fn koruma_validation_runs_before_submit_handler() {
    let mut server = test_server();
    form::<ValidatedRequest>(&mut server)
        .model(|request| {
            Ok::<ObjectResponse, String>(object_response(json!({ "name": request.name })))
        })
        .expect("tool should register");

    let result = server.call_tool(
        &ValidatedRequest::descriptor().tool_name(),
        Some(json!({ "name": "" })),
    );

    assert_eq!(result.is_error, Some(true));
    let error = result
        .structured_content
        .as_ref()
        .expect("validation should be structured")
        .get("error")
        .expect("structured validation error");
    assert_eq!(error["kind"], json!("validation"));
    let detail = error["detail"]
        .as_str()
        .expect("validation detail should be text");
    assert!(!detail.is_empty());
    let details = error["details"]
        .as_array()
        .expect("validation details should be an array");
    assert_eq!(details.len(), 1);
    assert_eq!(details[0]["scope"], json!("field"));
    assert_eq!(details[0]["field"], json!("name"));
    assert_eq!(details[0]["validator"], json!("NonEmptyValidation"));
    assert_eq!(details[0]["path"], json!("NonEmptyValidation"));
    assert_eq!(details[0]["target"], json!("default"));
    assert_eq!(details[0]["message"], json!(detail));
    assert!(
        result.content[0]
            .as_text()
            .is_some_and(|text| text.text.contains("validation failed"))
    );
}

#[test]
fn component_backed_field_decodes_without_gpui_runtime() {
    let mut server = test_server();
    form::<ComponentRequest>(&mut server)
        .model(|request| {
            Ok::<ObjectResponse, String>(object_response(json!({ "title": request.title })))
        })
        .expect("tool should register");

    let result = server.call_tool(
        &ComponentRequest::descriptor().tool_name(),
        Some(json!({ "title": "Headless" })),
    );

    assert_eq!(result.is_error, Some(false));
    assert_eq!(
        result.structured_content,
        Some(json!({ "title": "Headless" }))
    );
}

#[test]
fn generated_mcp_editor_decodes_single_field_updates() {
    let mut holder = PlainRequestFormValueHolder::default();

    PlainRequest::decode_field(&mut holder, "title", McpAny::from(json!("Draft")))
        .expect("title should patch");
    PlainRequest::decode_field(&mut holder, "retries", McpAny::from(json!(3)))
        .expect("optional value should patch");
    PlainRequest::decode_field(&mut holder, "enabled", McpAny::from(json!(false)))
        .expect("defaulted value should patch");

    assert_eq!(holder.title, "Draft");
    assert_eq!(holder.retries, Some(3));
    assert!(!holder.enabled);

    let error = PlainRequest::decode_field(&mut holder, "unknown", McpAny::from(json!(true)))
        .expect_err("unknown fields should fail");
    assert_eq!(
        error,
        McpToolError::UnknownField {
            field: "unknown".to_string()
        }
    );
}

#[test]
fn editor_tools_open_patch_read_validate_and_close_sessions() {
    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .editor()
        .expect("editor tools should register");
    let tools = editor_tool_names(PlainRequest::descriptor());
    assert!(!server.contains_tool(&tools.submit));
    let default_idle_timeout_ms = DEFAULT_EDITOR_SESSION_IDLE_TIMEOUT.as_millis() as u64;

    let listed = server.call_tool(&tools.list, Some(json!({})));
    assert_eq!(listed.is_error, Some(false));
    assert_eq!(
        listed
            .structured_content
            .as_ref()
            .expect("list should return sessions")["session_count"],
        json!(0)
    );
    assert_eq!(
        listed
            .structured_content
            .as_ref()
            .expect("list should return sessions")["session_limit"],
        json!(DEFAULT_EDITOR_SESSION_LIMIT)
    );
    assert_eq!(
        listed
            .structured_content
            .as_ref()
            .expect("list should return sessions")["session_idle_timeout_ms"],
        json!(default_idle_timeout_ms)
    );
    assert_eq!(
        listed
            .structured_content
            .as_ref()
            .expect("list should return sessions")["cleanup"],
        json!({
            "expired_count": 0,
            "expired_session_ids": [],
            "evicted_count": 0,
            "evicted_session_ids": []
        })
    );

    let open = server.call_tool(&tools.open, Some(json!({})));
    assert_eq!(open.is_error, Some(false));
    let opened = open
        .structured_content
        .expect("open should return a session");
    let session_id = opened["session_id"]
        .as_str()
        .expect("session id should be a string");
    assert_eq!(opened["revision"], json!(0));
    assert_eq!(
        opened["cleanup"],
        json!({
            "expired_count": 0,
            "expired_session_ids": [],
            "evicted_count": 0,
            "evicted_session_ids": []
        })
    );
    assert_eq!(opened["missing_required"], json!(["title"]));
    assert_eq!(opened["valid"], false);
    assert_eq!(
        opened["errors"],
        json!([{
            "scope": "field",
            "message": "missing required field `title`",
            "field": "title",
            "validator": "required"
        }])
    );
    assert_eq!(opened["fields"][0]["name"], "title");
    assert_eq!(opened["fields"][0]["required"], true);
    assert_eq!(opened["fields"][0]["present"], false);
    assert_eq!(opened["fields"][0]["has_value"], false);
    assert_eq!(opened["fields"][0]["value"], Value::Null);
    assert_eq!(opened["fields"][0]["missing"], true);
    assert_eq!(
        opened["fields"][0]["errors"],
        json!([{
            "scope": "field",
            "message": "missing required field `title`",
            "field": "title",
            "validator": "required"
        }])
    );
    assert_eq!(opened["fields"][0]["schema"]["type"], "string");
    assert_eq!(opened["fields"][2]["name"], "enabled");
    assert_eq!(opened["fields"][2]["required"], false);
    assert_eq!(opened["fields"][2]["has_default"], true);
    assert_eq!(opened["fields"][2]["errors"], json!([]));
    assert_eq!(opened["submit_arguments"], json!({}));

    let listed = server.call_tool(&tools.list, Some(json!({})));
    assert_eq!(listed.is_error, Some(false));
    let listed = listed
        .structured_content
        .expect("list should return active session");
    assert_eq!(listed["session_count"], json!(1));
    assert_eq!(listed["sessions"][0]["session_id"], session_id);
    assert_eq!(listed["sessions"][0]["revision"], json!(0));
    assert_eq!(listed["sessions"][0]["valid"], false);
    assert_eq!(listed["sessions"][0]["fields"][0]["missing"], true);

    let patched = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "expected_revision": 0,
            "values": {
                "title": "Edited title"
            }
        })),
    );
    assert_eq!(patched.is_error, Some(false));
    let patched = patched
        .structured_content
        .expect("patch should return state");
    assert_eq!(patched["revision"], json!(1));
    assert_eq!(patched["missing_required"], json!([]));
    assert_eq!(patched["valid"], true);
    assert_eq!(patched["errors"], json!([]));
    assert_eq!(patched["fields"][0]["present"], true);
    assert_eq!(patched["fields"][0]["has_value"], true);
    assert_eq!(patched["fields"][0]["value"], "Edited title");
    assert_eq!(patched["fields"][0]["missing"], false);
    assert_eq!(patched["fields"][0]["errors"], json!([]));
    assert_eq!(patched["values"]["title"], "Edited title");
    assert_eq!(
        patched["submit_arguments"],
        json!({ "title": "Edited title" })
    );

    let listed = server.call_tool(&tools.list, Some(json!({})));
    assert_eq!(listed.is_error, Some(false));
    let listed = listed
        .structured_content
        .expect("list should return patched session");
    assert_eq!(
        listed["sessions"][0]["submit_arguments"],
        json!({ "title": "Edited title" })
    );
    assert_eq!(listed["sessions"][0]["revision"], json!(1));

    let stale_patch = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "expected_revision": 0,
            "values": {
                "title": "Stale title"
            }
        })),
    );
    assert_eq!(stale_patch.is_error, Some(true));
    assert_eq!(
        stale_patch.structured_content.expect("structured error")["error"],
        json!({
            "kind": "validation",
            "message": "validation failed: edit session revision mismatch: expected 0, current 1",
            "detail": "edit session revision mismatch: expected 0, current 1"
        })
    );

    let patched = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "expected_revision": 1,
            "values": {
                "retries": 5
            }
        })),
    );
    assert_eq!(patched.is_error, Some(false));
    let patched = patched
        .structured_content
        .expect("patch should return state");
    assert_eq!(patched["revision"], json!(2));
    assert_eq!(patched["values"]["retries"], 5);
    assert_eq!(
        patched["submit_arguments"],
        json!({ "title": "Edited title", "retries": 5 })
    );

    let patched = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "clear": ["retries"]
        })),
    );
    assert_eq!(patched.is_error, Some(false));
    let patched = patched
        .structured_content
        .expect("clear should return state");
    assert_eq!(patched["valid"], true);
    assert_eq!(patched["values"].get("retries"), None);
    assert_eq!(
        patched["submit_arguments"],
        json!({ "title": "Edited title" })
    );

    let patched = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "values": {
                "retries": null
            }
        })),
    );
    assert_eq!(patched.is_error, Some(false));
    let patched = patched
        .structured_content
        .expect("patch should return state");
    assert_eq!(patched["values"]["retries"], Value::Null);
    assert_eq!(patched["fields"][1]["name"], "retries");
    assert_eq!(patched["fields"][1]["has_value"], true);
    assert_eq!(patched["fields"][1]["value"], Value::Null);
    assert_eq!(
        patched["submit_arguments"],
        json!({ "title": "Edited title", "retries": null })
    );

    let patched = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "values": {
                "retries": 5
            }
        })),
    );
    assert_eq!(patched.is_error, Some(false));

    let validation = server.call_tool(
        &tools.validate,
        Some(json!({
            "session_id": session_id
        })),
    );
    assert_eq!(validation.is_error, Some(false));
    let validation = validation
        .structured_content
        .expect("validation should return state");
    assert_eq!(validation["valid"], true);
    assert_eq!(validation["errors"], json!([]));
    assert_eq!(
        validation["submit_arguments"],
        json!({ "title": "Edited title", "retries": 5 })
    );

    let read = server.call_tool(
        &tools.read,
        Some(json!({
            "session_id": session_id
        })),
    );
    assert_eq!(read.is_error, Some(false));
    assert_eq!(
        read.structured_content.expect("read")["submit_arguments"],
        json!({ "title": "Edited title", "retries": 5 })
    );

    let replaced = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "replace": true,
            "values": {
                "title": "Replacement title"
            }
        })),
    );
    assert_eq!(replaced.is_error, Some(false));
    let replaced = replaced
        .structured_content
        .expect("replace patch should return state");
    assert_eq!(replaced["valid"], true);
    assert_eq!(
        replaced["submit_arguments"],
        json!({ "title": "Replacement title" })
    );

    let patched = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "values": {
                "title": "Edited title",
                "retries": 5
            }
        })),
    );
    assert_eq!(patched.is_error, Some(false));

    let patched = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "clear": ["title"]
        })),
    );
    assert_eq!(patched.is_error, Some(false));
    let patched = patched
        .structured_content
        .expect("required clear should return state");
    assert_eq!(patched["valid"], false);
    assert_eq!(patched["missing_required"], json!(["title"]));
    assert_eq!(
        patched["fields"][0]["errors"],
        json!([{
            "scope": "field",
            "message": "missing required field `title`",
            "field": "title",
            "validator": "required"
        }])
    );
    assert_eq!(patched["submit_arguments"], json!({ "retries": 5 }));
    let close_revision = patched["revision"]
        .as_u64()
        .expect("patched revision should be an integer");

    let stale_close = server.call_tool(
        &tools.close,
        Some(json!({
            "session_id": session_id,
            "expected_revision": 0
        })),
    );
    assert_eq!(stale_close.is_error, Some(true));
    assert_eq!(
        stale_close.structured_content.expect("structured error")["error"],
        json!({
            "kind": "validation",
            "message": format!(
                "validation failed: edit session revision mismatch: expected 0, current {close_revision}"
            ),
            "detail": format!(
                "edit session revision mismatch: expected 0, current {close_revision}"
            )
        })
    );

    let close = server.call_tool(
        &tools.close,
        Some(json!({
            "session_id": session_id,
            "expected_revision": close_revision
        })),
    );
    assert_eq!(close.is_error, Some(false));

    let listed = server.call_tool(&tools.list, Some(json!({})));
    assert_eq!(listed.is_error, Some(false));
    assert_eq!(
        listed
            .structured_content
            .as_ref()
            .expect("list after close should return no sessions")["session_count"],
        json!(0)
    );

    let read_after_close = server.call_tool(
        &tools.read,
        Some(json!({
            "session_id": session_id
        })),
    );
    assert_eq!(read_after_close.is_error, Some(true));
    assert_eq!(
        read_after_close
            .structured_content
            .expect("structured error")["error"],
        json!({
            "kind": "invalid_field_value",
            "message": format!("unknown value `{session_id}` for field `session_id`"),
            "field": "session_id",
            "value": session_id
        })
    );

    let first = server.call_tool(&tools.open, Some(json!({})));
    assert_eq!(first.is_error, Some(false));
    let first_id = first.structured_content.unwrap()["session_id"]
        .as_str()
        .expect("first reopened session id")
        .to_string();
    let second = server.call_tool(
        &tools.open,
        Some(json!({
            "values": {
                "title": "Second session"
            }
        })),
    );
    assert_eq!(second.is_error, Some(false));
    let second_id = second.structured_content.unwrap()["session_id"]
        .as_str()
        .expect("second reopened session id")
        .to_string();

    let close_all = server.call_tool(&tools.close_all, Some(json!({})));
    assert_eq!(close_all.is_error, Some(false));
    let close_all = close_all
        .structured_content
        .expect("close_all should return closed session ids");
    assert_eq!(
        close_all["form"],
        json!(PlainRequest::descriptor().tool_name())
    );
    assert_eq!(close_all["form_name"], "PlainRequest");
    assert_eq!(close_all["closed_count"], json!(2));
    assert_eq!(close_all["session_ids"], json!([first_id, second_id]));

    let listed = server.call_tool(&tools.list, Some(json!({})));
    assert_eq!(listed.is_error, Some(false));
    assert_eq!(
        listed
            .structured_content
            .as_ref()
            .expect("list after close_all should return no sessions")["session_count"],
        json!(0)
    );

    let empty_close_all = server.call_tool(&tools.close_all, Some(json!({})));
    assert_eq!(empty_close_all.is_error, Some(false));
    let empty_close_all = empty_close_all
        .structured_content
        .expect("empty close_all should still succeed");
    assert_eq!(empty_close_all["closed_count"], json!(0));
    assert_eq!(empty_close_all["session_ids"], json!([]));
}

#[test]
fn editor_options_evict_oldest_sessions_past_limit() {
    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .editor_with_options(McpFormEditorOptions::default().with_session_limit(2))
        .expect("editor tools should register with options");
    let tools = editor_tool_names(PlainRequest::descriptor());

    let mut session_ids = Vec::new();
    let mut third_opened = None;
    for title in ["First", "Second", "Third"] {
        let opened = server.call_tool(
            &tools.open,
            Some(json!({
                "values": {
                    "title": title
                }
            })),
        );
        assert_eq!(opened.is_error, Some(false));
        let opened = opened
            .structured_content
            .expect("open should return a session");
        session_ids.push(
            opened["session_id"]
                .as_str()
                .expect("session id should be a string")
                .to_string(),
        );
        if title == "Third" {
            third_opened = Some(opened);
        }
    }
    let third_opened = third_opened.expect("third open should be captured");
    assert_eq!(
        third_opened["cleanup"],
        json!({
            "expired_count": 0,
            "expired_session_ids": [],
            "evicted_count": 1,
            "evicted_session_ids": [session_ids[0]]
        })
    );

    let listed = server.call_tool(&tools.list, Some(json!({})));
    assert_eq!(listed.is_error, Some(false));
    let listed = listed
        .structured_content
        .expect("list should return bounded sessions");
    assert_eq!(listed["session_limit"], json!(2));
    assert_eq!(
        listed["cleanup"],
        json!({
            "expired_count": 0,
            "expired_session_ids": [],
            "evicted_count": 0,
            "evicted_session_ids": []
        })
    );
    assert_eq!(listed["session_count"], json!(2));
    assert_eq!(listed["sessions"][0]["session_id"], session_ids[1]);
    assert_eq!(
        listed["sessions"][0]["submit_arguments"],
        json!({ "title": "Second" })
    );
    assert_eq!(listed["sessions"][1]["session_id"], session_ids[2]);
    assert_eq!(
        listed["sessions"][1]["submit_arguments"],
        json!({ "title": "Third" })
    );

    let evicted_read = server.call_tool(
        &tools.read,
        Some(json!({
            "session_id": session_ids[0]
        })),
    );
    assert_eq!(evicted_read.is_error, Some(true));
    assert_eq!(
        evicted_read
            .structured_content
            .expect("read should return a structured error")["error"]["kind"],
        json!("invalid_field_value")
    );
}

#[test]
fn editor_options_expire_idle_sessions_before_next_operation() {
    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .editor_with_options(
            McpFormEditorOptions::default().with_session_idle_timeout(Duration::ZERO),
        )
        .expect("editor tools should register with idle timeout");
    let tools = editor_tool_names(PlainRequest::descriptor());

    let opened = server.call_tool(
        &tools.open,
        Some(json!({
            "values": {
                "title": "Expires immediately after open"
            }
        })),
    );
    assert_eq!(opened.is_error, Some(false));
    let session_id = opened
        .structured_content
        .expect("open should return a session")["session_id"]
        .as_str()
        .expect("session id should be a string")
        .to_string();

    let listed = server.call_tool(&tools.list, Some(json!({})));
    assert_eq!(listed.is_error, Some(false));
    let listed = listed
        .structured_content
        .expect("list should expire idle sessions");
    assert_eq!(listed["session_idle_timeout_ms"], json!(0));
    assert_eq!(
        listed["cleanup"],
        json!({
            "expired_count": 1,
            "expired_session_ids": [session_id],
            "evicted_count": 0,
            "evicted_session_ids": []
        })
    );
    assert_eq!(listed["session_count"], json!(0));

    let expired_read = server.call_tool(
        &tools.read,
        Some(json!({
            "session_id": session_id
        })),
    );
    assert_eq!(expired_read.is_error, Some(true));
    assert_eq!(
        expired_read
            .structured_content
            .expect("read should return a structured error")["error"]["kind"],
        json!("invalid_field_value")
    );
}

#[test]
fn editor_bulk_patch_rejects_invalid_values_without_partial_mutation() {
    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .editor()
        .expect("editor tools should register");
    let tools = editor_tool_names(PlainRequest::descriptor());

    let open = server.call_tool(&tools.open, Some(json!({})));
    assert_eq!(open.is_error, Some(false));
    let opened = open
        .structured_content
        .expect("open should return a session");
    let session_id = opened["session_id"]
        .as_str()
        .expect("session id should be a string");

    let failed = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "values": {
                "title": "Should not commit",
                "unknown": true
            }
        })),
    );
    assert_eq!(failed.is_error, Some(true));
    assert_eq!(
        failed.structured_content.expect("structured error")["error"],
        json!({
            "kind": "unknown_field",
            "message": "unknown field `unknown`",
            "field": "unknown"
        })
    );

    let read = server.call_tool(
        &tools.read,
        Some(json!({
            "session_id": session_id
        })),
    );
    assert_eq!(read.is_error, Some(false));
    let read = read.structured_content.expect("read should return state");
    assert_eq!(read["submit_arguments"], json!({}));
    assert_eq!(read["missing_required"], json!(["title"]));
    assert_eq!(read["valid"], false);

    let failed = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "clear": ["unknown"]
        })),
    );
    assert_eq!(failed.is_error, Some(true));
    assert_eq!(
        failed.structured_content.expect("structured error")["error"],
        json!({
            "kind": "unknown_field",
            "message": "unknown field `unknown`",
            "field": "unknown"
        })
    );
}

#[test]
fn editor_tools_patch_component_backed_fields_through_shape_schema() {
    let mut server = test_server();
    form::<ComponentRequest>(&mut server)
        .editor()
        .expect("component-backed editor tools should register");
    let tools = editor_tool_names(ComponentRequest::descriptor());
    let patch_tool = server
        .list_tools()
        .into_iter()
        .find(|tool| tool.name == tools.patch)
        .expect("patch tool should be registered");
    assert_eq!(
        patch_tool.input_schema["properties"]["values"]["properties"]["title"]["type"],
        "string"
    );
    assert_eq!(
        patch_tool.input_schema["properties"]["clear"]["items"]["enum"],
        json!(["title"])
    );
    assert_eq!(
        patch_tool.input_schema["properties"]["replace"]["type"],
        "boolean"
    );

    let open = server.call_tool(
        &tools.open,
        Some(json!({
            "values": {
                "title": "Component title"
            }
        })),
    );
    assert_eq!(open.is_error, Some(false));
    let opened = open
        .structured_content
        .expect("open should return a session");
    let session_id = opened["session_id"]
        .as_str()
        .expect("session id should be a string");
    assert_eq!(opened["values"]["title"], "Component title");
    assert_eq!(opened["missing_required"], json!([]));
    assert_eq!(opened["valid"], true);

    let patched = server.call_tool(
        &tools.patch,
        Some(json!({
            "session_id": session_id,
            "values": {
                "title": "Updated component title"
            }
        })),
    );
    assert_eq!(patched.is_error, Some(false));
    assert_eq!(
        patched.structured_content.expect("patch")["submit_arguments"],
        json!({ "title": "Updated component title" })
    );
}

#[test]
fn generated_mcp_submit_uses_field_mcp_tool_value_decoder() {
    let schema = CustomToolValueRequest::descriptor().input_schema();
    assert_eq!(schema["properties"]["tags"]["type"], "string");

    let mut server = test_server();
    form::<CustomToolValueRequest>(&mut server)
        .model(|request| {
            Ok::<ObjectResponse, String>(object_response(json!({ "tags": request.tags.0 })))
        })
        .expect("tool should register with the field McpToolValue impl");

    let result = server.call_tool(
        &CustomToolValueRequest::descriptor().tool_name(),
        Some(json!({ "tags": "alpha / beta / gamma" })),
    );

    assert_eq!(result.is_error, Some(false));
    assert_eq!(
        result.structured_content,
        Some(json!({ "tags": ["alpha", "beta", "gamma"] }))
    );

    let result = server.call_tool(
        &CustomToolValueRequest::descriptor().tool_name(),
        Some(json!({ "tags": ["alpha"] })),
    );

    assert_eq!(result.is_error, Some(true));
    assert!(
        result.content[0]
            .as_text()
            .is_some_and(|text| text.text.contains("expected slash-separated tags"))
    );
    assert_eq!(
        result.structured_content.expect("structured error")["error"],
        json!({
            "kind": "decode_field",
            "message": "failed to decode field `tags`: expected slash-separated tags",
            "field": "tags",
            "detail": "expected slash-separated tags"
        })
    );
}

#[test]
fn generated_mcp_submit_reuses_mcp_tool_input_as_field_schema() {
    let mut server = test_server();
    form::<ToolInputSchemaRequest>(&mut server)
        .model(|request| {
            Ok::<ObjectResponse, String>(object_response(json!({
                "email_updates": request.preferences.email_updates,
                "topics": request.preferences.topics,
            })))
        })
        .expect("tool should register with the field McpToolInput-derived schema");

    let result = server.call_tool(
        &ToolInputSchemaRequest::descriptor().tool_name(),
        Some(json!({
            "preferences": {
                "email": true,
                "topics": ["rust", "gpui"]
            }
        })),
    );

    assert_eq!(result.is_error, Some(false));
    assert_eq!(
        result.structured_content,
        Some(json!({
            "email_updates": true,
            "topics": ["rust", "gpui"]
        }))
    );

    let result = server.call_tool(
        &ToolInputSchemaRequest::descriptor().tool_name(),
        Some(json!({
            "preferences": {
                "email": true,
                "unexpected": true
            }
        })),
    );

    assert_eq!(result.is_error, Some(true));
    assert!(matches!(
        result.content[0].as_text(),
        Some(text) if text.text.contains("unknown field `preferences.unexpected`")
    ));
    assert_eq!(
        result.structured_content.expect("structured error")["error"],
        json!({
            "kind": "unknown_field",
            "message": "unknown field `preferences.unexpected`",
            "field": "preferences.unexpected"
        })
    );
}

#[test]
fn component_backed_field_uses_typed_schema_when_shape_input_is_unavailable() {
    let schema = ComponentNoSchemaRequest::descriptor().input_schema();
    assert_eq!(schema["properties"]["title"]["type"], "string");
    assert_eq!(
        schema["properties"]["title"]["x-gpuiFormComponentBacked"],
        true
    );

    let mut server = test_server();
    form::<ComponentNoSchemaRequest>(&mut server)
        .model(|request| {
            Ok::<ObjectResponse, String>(object_response(json!({ "title": request.title.0 })))
        })
        .expect("tool should register with the field McpJsonSchema impl");

    let result = server.call_tool(
        &ComponentNoSchemaRequest::descriptor().tool_name(),
        Some(json!({ "title": "Headless" })),
    );

    assert_eq!(result.is_error, Some(false));
    assert_eq!(
        result.structured_content,
        Some(json!({ "title": "Headless" }))
    );
}

#[test]
fn submit_only_mcp_forms_do_not_require_serializable_field_values() {
    let mut server = test_server();
    form::<DecodeOnlyRequest>(&mut server)
        .model(|request| {
            Ok::<ObjectResponse, String>(object_response(json!({ "value": request.value.0 })))
        })
        .expect("submit tool should register without editor serialization bounds");

    let result = server.call_tool(
        &DecodeOnlyRequest::descriptor().tool_name(),
        Some(json!({ "value": "decode-only" })),
    );

    assert_eq!(result.is_error, Some(false));
    assert_eq!(
        result.structured_content,
        Some(json!({ "value": "decode-only" }))
    );
}

#[test]
fn skipped_fields_use_holder_submission() {
    let mut server = test_server();
    form::<SkippedRequest>(&mut server)
        .holder(|holder| {
            Ok::<ObjectResponse, String>(object_response(json!({
                "name": holder.name,
                "tenant": "application-owned"
            })))
        })
        .expect("tool should register");

    let result = server.call_tool(
        &SkippedRequest::descriptor().tool_name(),
        Some(json!({ "name": "Ada" })),
    );

    assert_eq!(result.is_error, Some(false));
    assert_eq!(
        result.structured_content,
        Some(json!({
            "name": "Ada",
            "tenant": "application-owned"
        }))
    );
}

#[test]
fn attribute_handlers_register_with_inventory_registry() {
    let server = generated_server().expect("generated tools should register");

    let plain = server.call_tool(
        &PlainRequest::descriptor().tool_name(),
        Some(json!({ "title": "Attr" })),
    );
    assert_eq!(plain.is_error, Some(false));
    assert_eq!(
        plain.structured_content,
        Some(json!({
            "title": "Attr",
            "enabled": true,
            "source": "attribute"
        }))
    );

    let plain_editor = editor_tool_names(PlainRequest::descriptor());
    let opened = server.call_tool(
        &plain_editor.open,
        Some(json!({ "values": { "title": "Editable Attr" } })),
    );
    assert_eq!(opened.is_error, Some(false));
    let opened = opened
        .structured_content
        .expect("generated server should expose editor tools");
    assert_eq!(
        opened["submit_arguments"],
        json!({ "title": "Editable Attr" })
    );
    let session_id = opened["session_id"]
        .as_str()
        .expect("session id should be a string");
    let edit_submit = server.call_tool(
        &plain_editor.submit,
        Some(json!({
            "session_id": session_id
        })),
    );
    assert_eq!(edit_submit.is_error, Some(false));
    assert_eq!(
        edit_submit.structured_content,
        Some(json!({
            "title": "Editable Attr",
            "enabled": true,
            "source": "attribute"
        }))
    );
    let read_after_edit_submit = server.call_tool(
        &plain_editor.read,
        Some(json!({
            "session_id": session_id
        })),
    );
    assert_eq!(read_after_edit_submit.is_error, Some(true));

    let skipped = server.call_tool(
        &SkippedRequest::descriptor().tool_name(),
        Some(json!({ "name": "Ada" })),
    );
    assert_eq!(skipped.is_error, Some(false));
    assert_eq!(
        skipped.structured_content,
        Some(json!({
            "name": "Ada",
            "tenant": "attribute"
        }))
    );

    let direct = server.call_tool(
        &DirectRequest::descriptor().tool_name(),
        Some(json!({ "name": "Direct" })),
    );
    assert_eq!(direct.is_error, Some(false));
    assert_eq!(
        direct.structured_content,
        Some(json!({
            "name": "Direct",
            "source": "direct"
        }))
    );

    let asynchronous = server.call_tool(
        &AsyncRequest::descriptor().tool_name(),
        Some(json!({ "title": "Async" })),
    );
    assert_eq!(asynchronous.is_error, Some(false));
    assert_eq!(
        asynchronous.structured_content,
        Some(json!({
            "title": "Async",
            "source": "async"
        }))
    );

    let async_value = server.call_tool(
        &AsyncValueRequest::descriptor().tool_name(),
        Some(json!({ "label": "Value" })),
    );
    assert_eq!(async_value.is_error, Some(false));
    assert_eq!(
        async_value.structured_content,
        Some(json!({
            "label": "Value",
            "source": "async-direct"
        }))
    );

    let metadata = server.call_tool(
        &MetadataRequest::descriptor().tool_name(),
        Some(json!({ "label": "Named" })),
    );
    assert_eq!(metadata.is_error, Some(false));
    assert_eq!(
        metadata.structured_content,
        Some(json!({
            "label": "Named",
            "source": "metadata"
        }))
    );

    let explicit_model = server.call_tool(
        &ExplicitModelRequest::descriptor().tool_name(),
        Some(json!({ "title": "Explicit" })),
    );
    assert_eq!(explicit_model.is_error, Some(false));
    assert_eq!(
        explicit_model.structured_content,
        Some(json!({
            "title": "Explicit",
            "mode": "model"
        }))
    );

    let explicit_holder = server.call_tool(
        &ExplicitHolderRequest::descriptor().tool_name(),
        Some(json!({ "body": "Hello" })),
    );
    assert_eq!(explicit_holder.is_error, Some(false));
    assert_eq!(
        explicit_holder.structured_content,
        Some(json!({
            "body": "Hello",
            "mode": "holder"
        }))
    );
}

#[test]
fn duplicate_form_registration_returns_setup_error() {
    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .model(|request| {
            Ok::<ObjectResponse, String>(object_response(json!({ "title": request.title })))
        })
        .expect("first registration should succeed");

    let error = match form::<PlainRequest>(&mut server).model(|request| {
        Ok::<ObjectResponse, String>(object_response(json!({ "title": request.title })))
    }) {
        Ok(_) => panic!("duplicate tool should fail registration"),
        Err(error) => error,
    };

    assert_eq!(
        error,
        McpToolError::DuplicateTool {
            name: PlainRequest::descriptor().tool_name()
        }
    );
}
