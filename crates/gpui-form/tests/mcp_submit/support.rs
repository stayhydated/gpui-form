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
use koruma::NewtypeValue as _;
use koruma_collection::{
    collection::{LenValidation, NonEmptyValidation},
    numeric::RangeValidation,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
#[cfg(feature = "runtime")]
use std::{fmt, str::FromStr};
use std::{
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

type Attempts = u32;

#[derive(
    Clone, Debug, Default, Deserialize, Eq, gpui_form::mcp::McpJsonSchema, PartialEq, Serialize,
)]
#[serde(transparent)]
pub struct AccountId(u64);

/// Submit a plain request from inferred docs.
#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
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
        gpui_form::mcp::McpSchema::object()
    }
}

fn object_response(value: Value) -> ObjectResponse {
    assert!(value.is_object(), "test response must be a JSON object");
    ObjectResponse(value)
}

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
#[gpui_form(koruma)]
#[gpui_form(mcp)]
pub struct ValidatedRequest {
    #[gpui_form(hidden)]
    #[koruma(NonEmptyValidation::<_>)]
    name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
#[gpui_form(koruma)]
#[gpui_form(mcp)]
pub struct ConstrainedRequest {
    #[gpui_form(hidden)]
    #[koruma(LenValidation::<_>.min(2).max(8))]
    title: String,
    #[gpui_form(hidden)]
    #[koruma(RangeValidation::<_>.min(1).max(5).exclusive_max(true))]
    retries: u32,
}

#[derive(
    Clone,
    Debug,
    Default,
    Deserialize,
    Eq,
    gpui_form::mcp::McpJsonSchema,
    koruma::Koruma,
    PartialEq,
    Serialize,
)]
#[serde(transparent)]
#[koruma(newtype)]
pub struct RequestCode(#[koruma(LenValidation::<_>.min(2).max(8))] String);

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct NewtypeConvertedRequest {
    #[gpui_form(hidden(value(koruma_newtype)))]
    code: RequestCode,
}

#[cfg(feature = "runtime")]
#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct ComponentRequest {
    #[gpui_form(component(gpui_form_collection::input::Input::<String>))]
    title: String,
}

#[cfg(feature = "runtime")]
#[derive(
    Clone, Debug, Default, Deserialize, Eq, gpui_form::mcp::McpJsonSchema, PartialEq, Serialize,
)]
#[serde(transparent)]
pub struct ComponentOnlyValue(String);

#[cfg(feature = "runtime")]
impl fmt::Display for ComponentOnlyValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(feature = "runtime")]
impl FromStr for ComponentOnlyValue {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self(value.to_string()))
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, gpui_form::mcp::McpJsonSchema, PartialEq)]
#[serde(transparent)]
pub struct DecodeOnlyValue(String);

#[derive(
    Clone, Debug, Default, Deserialize, Eq, gpui_form::mcp::McpToolInput, PartialEq, Serialize,
)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPreferences {
    #[mcp(alias = "email")]
    email_updates: bool,
    #[serde(default)]
    topics: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SlashSeparatedTags(Vec<String>);

impl gpui_form::mcp::McpToolValue for SlashSeparatedTags {
    fn tool_value_schema() -> gpui_form::mcp::McpSchema {
        gpui_form::mcp::McpSchema::string()
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

#[cfg(feature = "runtime")]
#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct ComponentNoSchemaRequest {
    #[gpui_form(component(gpui_form_collection::input::Input::<ComponentOnlyValue>))]
    title: ComponentOnlyValue,
}

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq)]
#[gpui_form(mcp)]
pub struct DecodeOnlyRequest {
    #[gpui_form(hidden)]
    value: DecodeOnlyValue,
}

#[derive(Clone, Debug, Eq, GpuiForm, PartialEq)]
#[gpui_form(mcp)]
pub struct CustomToolValueRequest {
    #[gpui_form(hidden)]
    tags: SlashSeparatedTags,
}

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct ToolInputSchemaRequest {
    #[gpui_form(hidden)]
    preferences: NotificationPreferences,
}

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct CustomSchemaRequest {
    #[gpui_form(hidden)]
    account_id: AccountId,
    #[gpui_form(hidden)]
    attempts: Attempts,
}

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct TupleSchemaRequest {
    #[gpui_form(hidden)]
    coordinates: (u32, u32),
}

#[derive(
    Clone, Debug, Default, Deserialize, Eq, gpui_form::mcp::McpJsonSchema, PartialEq, Serialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum Priority {
    #[default]
    Low,
    #[serde(alias = "urgent")]
    HighPriority,
}

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct EnumSchemaRequest {
    #[gpui_form(hidden)]
    priority: Priority,
}

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct SkippedRequest {
    #[gpui_form(hidden)]
    name: String,
    #[gpui_form(skip)]
    tenant_id: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct DirectRequest {
    #[gpui_form(hidden)]
    name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct AsyncRequest {
    #[gpui_form(hidden)]
    title: String,
}

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct AsyncValueRequest {
    #[gpui_form(hidden)]
    label: String,
}

#[derive(Clone, Debug)]
pub struct SubmitContext {
    prefix: String,
}

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
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

    async fn submit_with_context(
        self,
        context: SubmitContext,
    ) -> Result<Self::Response, Self::Error> {
        Ok(ContextSubmitResponse {
            prefixed: format!("{}{}", context.prefix, self.title),
            title: self.title,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
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

#[derive(Clone, Debug)]
pub struct TraitSubmitContext {
    prefix: String,
}

#[derive(Debug, gpui_form::mcp::McpJsonSchema, Serialize)]
pub struct TraitSubmitResponse {
    title: String,
    prefixed: String,
}

pub trait TraitBackedSubmitOutput {
    fn trait_backed_output(self, context: TraitSubmitContext) -> TraitSubmitResponse;
}

pub trait TraitBackedSubmitter: TraitBackedSubmitOutput + Sized + Send + 'static {
    type Context: Clone + Send + Sync + 'static;
    type Response: Serialize + gpui_form::mcp::McpJsonSchema;
    type Error: std::fmt::Display;

    fn submit_with_context(
        self,
        context: Self::Context,
    ) -> impl std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + 'static;
}

impl<T> TraitBackedSubmitter for T
where
    T: TraitBackedSubmitOutput + Send + 'static,
{
    type Context = TraitSubmitContext;
    type Response = TraitSubmitResponse;
    type Error = String;

    async fn submit_with_context(
        self,
        context: Self::Context,
    ) -> Result<Self::Response, Self::Error> {
        Ok(self.trait_backed_output(context))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp(
    name = "submit_trait_backed_context_request",
    title = "Submit trait-backed context request",
    description = "Exercise context-backed MCP submit registration through a submitter trait.",
    read_only = true,
    destructive = false,
    idempotent = true,
    open_world = false,
    submitter(TraitBackedSubmitter)
))]
pub struct TraitBackedContextRequest {
    #[gpui_form(hidden)]
    title: String,
}

impl TraitBackedSubmitOutput for TraitBackedContextRequest {
    fn trait_backed_output(self, context: TraitSubmitContext) -> TraitSubmitResponse {
        TraitSubmitResponse {
            prefixed: format!("{}{}", context.prefix, self.title),
            title: self.title,
        }
    }
}

pub struct TraitRawSubmitResponse {
    title: String,
    prefixed: String,
}

pub trait MappedTraitBackedSubmitter: Sized + Send + 'static {
    type Context: Clone + Send + Sync + 'static;
    type Response;
    type Error: std::fmt::Display;

    fn submit_with_context(
        self,
        context: Self::Context,
    ) -> impl std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + 'static;
}

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp(
    name = "submit_mapped_trait_backed_context_request",
    title = "Submit mapped trait-backed context request",
    description = "Exercise context-backed MCP submitter registration with mapped raw responses.",
    read_only = true,
    destructive = false,
    idempotent = true,
    open_world = false,
    submitter(MappedTraitBackedSubmitter),
    map_response(map_trait_raw_response)
))]
pub struct MappedTraitBackedContextRequest {
    #[gpui_form(hidden)]
    title: String,
}

impl MappedTraitBackedSubmitter for MappedTraitBackedContextRequest {
    type Context = TraitSubmitContext;
    type Response = TraitRawSubmitResponse;
    type Error = String;

    async fn submit_with_context(
        self,
        context: Self::Context,
    ) -> Result<Self::Response, Self::Error> {
        Ok(TraitRawSubmitResponse {
            prefixed: format!("{}{}", context.prefix, self.title),
            title: self.title,
        })
    }
}

fn map_trait_raw_response(
    response: TraitRawSubmitResponse,
) -> Result<gpui_form::mcp::McpObject, String> {
    gpui_form::mcp::McpObject::from_value(json!({
        "title": response.title,
        "prefixed": response.prefixed
    }))
    .map_err(|error| error.to_string())
}

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp(
    name = "submit_metadata_request",
    title = "Submit metadata request",
    description = "Exercise explicit form MCP metadata.",
    read_only = true,
    destructive = false,
    idempotent = true,
    open_world = false,
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

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
#[gpui_form(mcp)]
pub struct ExplicitModelRequest {
    #[gpui_form(hidden)]
    title: String,
}

#[derive(Clone, Debug, Deserialize, Eq, GpuiForm, PartialEq, Serialize)]
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

#[path = "component_fields.rs"]
mod component_fields;
#[path = "editor.rs"]
mod editor;
#[path = "registration.rs"]
mod registration;
#[path = "schema_metadata.rs"]
mod schema_metadata;
#[path = "submission.rs"]
mod submission;
#[path = "validation.rs"]
mod validation;
