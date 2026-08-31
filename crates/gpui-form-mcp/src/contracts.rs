use std::{any::Any, fmt, future::Future, sync::Arc};

use serde_json::{Map, Value};

use crate::*;

pub type McpSubmitContext = Arc<dyn Any + Send + Sync>;

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
