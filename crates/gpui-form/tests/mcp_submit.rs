#![cfg(feature = "mcp")]

use gpui_form::{
    GpuiForm,
    mcp::{
        McpForm as _, McpServer, McpToolError, form, server as generated_server, tool_definitions,
    },
};
use koruma_collection::collection::NonEmptyValidation;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{fmt, str::FromStr};

type Attempts = u32;

#[derive(
    Clone, Debug, Default, Deserialize, gpui_form::mcp::McpJsonSchema, PartialEq, Serialize,
)]
#[mcp(crate = gpui_form::mcp)]
#[serde(transparent)]
pub struct AccountId(u64);

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct PlainRequest {
    #[gpui_form(hidden)]
    title: String,
    #[gpui_form(hidden)]
    retries: Option<u32>,
    #[gpui_form(hidden(default = true))]
    enabled: bool,
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
#[gpui_form(mcp)]
pub struct ComponentRequest {
    #[gpui_form(component(gpui_form_collection::input::Input::<String>))]
    title: String,
}

#[derive(
    Clone, Debug, Default, Deserialize, gpui_form::mcp::McpJsonSchema, PartialEq, Serialize,
)]
#[mcp(crate = gpui_form::mcp)]
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

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct ComponentNoSchemaRequest {
    #[gpui_form(component(gpui_form_collection::input::Input::<ComponentOnlyValue>))]
    title: ComponentOnlyValue,
}

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct CustomSchemaRequest {
    #[gpui_form(hidden)]
    account_id: AccountId,
    #[gpui_form(hidden)]
    attempts: Attempts,
}

#[derive(
    Clone, Debug, Default, Deserialize, gpui_form::mcp::McpJsonSchema, PartialEq, Serialize,
)]
#[mcp(crate = gpui_form::mcp)]
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

#[derive(Clone, Debug, Deserialize, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp(
    name = "submit_metadata_request",
    title = "Submit metadata request",
    description = "Exercise explicit form MCP metadata."
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
fn submit_plain_request(request: PlainRequest) -> Result<Value, String> {
    Ok(json!({
        "title": request.title,
        "enabled": request.enabled,
        "source": "attribute"
    }))
}

#[gpui_form::mcp_submit]
fn submit_skipped_request(holder: SkippedRequestFormValueHolder) -> Result<Value, String> {
    Ok(json!({
        "name": holder.name,
        "tenant": "attribute"
    }))
}

#[gpui_form::mcp_submit]
fn submit_direct_request(request: DirectRequest) -> Result<Value, String> {
    Ok(json!({
        "name": request.name,
        "source": "direct"
    }))
}

#[gpui_form::mcp_submit]
async fn submit_async_request(request: AsyncRequest) -> Result<Value, String> {
    Ok(json!({
        "title": request.title,
        "source": "async"
    }))
}

#[gpui_form::mcp_submit]
async fn submit_async_value_request(request: AsyncValueRequest) -> Result<Value, String> {
    Ok(json!({
        "label": request.label,
        "source": "async-direct"
    }))
}

#[gpui_form::mcp_submit]
fn submit_metadata_request(request: MetadataRequest) -> Result<Value, String> {
    Ok(json!({
        "label": request.label,
        "source": "metadata"
    }))
}

#[gpui_form::mcp_submit]
fn submit_explicit_model_request(request: ExplicitModelRequest) -> Result<Value, String> {
    Ok(json!({
        "title": request.title,
        "mode": "model"
    }))
}

#[gpui_form::mcp_submit]
fn submit_explicit_holder_request(
    holder: ExplicitHolderRequestFormValueHolder,
) -> Result<Value, String> {
    Ok(json!({
        "body": holder.body,
        "mode": "holder"
    }))
}

fn test_server() -> McpServer {
    McpServer::new("gpui-form-mcp-test", "0.0.0")
}

#[test]
fn generated_mcp_schema_marks_required_optional_default_and_component_fields() {
    let schema = PlainRequest::descriptor().input_schema();

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

    let enum_schema = EnumSchemaRequest::descriptor().input_schema();
    assert_eq!(enum_schema["properties"]["priority"]["type"], "string");
    assert_eq!(
        enum_schema["properties"]["priority"]["enum"],
        json!(["low", "high-priority", "urgent"])
    );
}

#[test]
fn generated_mcp_submit_registration_is_inventory_visible() {
    let registered_names: Vec<_> = tool_definitions()
        .expect("tool definitions should be generated")
        .into_iter()
        .map(|tool| tool.name.to_string())
        .collect();

    assert!(registered_names.contains(&PlainRequest::descriptor().tool_name()));
    assert!(registered_names.contains(&ComponentRequest::descriptor().tool_name()));
    assert!(registered_names.contains(&ComponentNoSchemaRequest::descriptor().tool_name()));
    assert!(registered_names.contains(&CustomSchemaRequest::descriptor().tool_name()));
    assert!(registered_names.contains(&DirectRequest::descriptor().tool_name()));
    assert!(registered_names.contains(&AsyncRequest::descriptor().tool_name()));
    assert!(registered_names.contains(&AsyncValueRequest::descriptor().tool_name()));
    assert!(registered_names.contains(&MetadataRequest::descriptor().tool_name()));
    assert!(registered_names.contains(&ExplicitModelRequest::descriptor().tool_name()));
    assert!(registered_names.contains(&ExplicitHolderRequest::descriptor().tool_name()));
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
}

#[test]
fn tool_call_decodes_validates_converts_and_submits_model() {
    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .model(|request| {
            Ok::<Value, String>(json!({
                "title": request.title,
                "retries": request.retries,
                "enabled": request.enabled
            }))
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
fn tool_call_reports_missing_values_as_tool_errors() {
    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .model(|_| Ok::<Value, String>(json!(null)))
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
        .model(|request| Ok::<Value, String>(json!({ "name": request.name })))
        .expect("tool should register");

    let result = server.call_tool(
        &ValidatedRequest::descriptor().tool_name(),
        Some(json!({ "name": "" })),
    );

    assert_eq!(result.is_error, Some(true));
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
        .model(|request| Ok::<Value, String>(json!({ "title": request.title })))
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
fn component_backed_field_uses_typed_schema_when_shape_input_is_unavailable() {
    let schema = ComponentNoSchemaRequest::descriptor().input_schema();
    assert_eq!(schema["properties"]["title"]["type"], "string");
    assert_eq!(
        schema["properties"]["title"]["x-gpuiFormComponentBacked"],
        true
    );

    let mut server = test_server();
    form::<ComponentNoSchemaRequest>(&mut server)
        .model(|request| Ok::<Value, String>(json!({ "title": request.title.0 })))
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
fn skipped_fields_use_holder_submission() {
    let mut server = test_server();
    form::<SkippedRequest>(&mut server)
        .holder(|holder| {
            Ok::<Value, String>(json!({
                "name": holder.name,
                "tenant": "application-owned"
            }))
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
        .model(|request| Ok::<Value, String>(json!({ "title": request.title })))
        .expect("first registration should succeed");

    let error = match form::<PlainRequest>(&mut server)
        .model(|request| Ok::<Value, String>(json!({ "title": request.title })))
    {
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
