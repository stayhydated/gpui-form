//! Experimental MCP submit integration for generated `gpui-form` value holders.
//!
//! This crate intentionally keeps GPUI out of the submit execution path. It
//! owns form holder/model decoding and validation while delegating shared MCP
//! server and stdio serving mechanics to `component-shape-mcp`.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::{Arc, Mutex},
};

use gpui_form_schema::registry::{FieldValuePresence, RustPath, RustType};
pub use serde::Serialize;
use serde_json::{Map, Value};

pub use component_shape_mcp::{
    ComponentShapeFor, ComponentShapeMetadata, ContentBlock, MCP_PROTOCOL_VERSION, McpAny,
    McpArguments, McpInput, McpInputShape, McpJsonSchema, McpPrimitiveKind, McpRange,
    McpRangeBoundKind, McpSchema, McpSchemaFn, McpSchemaProperties, McpServer, McpServerBuilder,
    McpToolArguments, McpToolCall, McpToolError, McpToolInput, McpToolMetadata, McpToolValue,
    McpTypedTool, ServeStdioResult, ToolCallResult, ToolDefinition, object_schema, rmcp, serde,
    serde_json,
};

pub type FieldToolValueSchemaFn = McpSchemaFn;
type ToolFuture = Pin<Box<dyn Future<Output = ToolCallResult> + Send + 'static>>;

#[derive(Clone, Copy, Debug)]
pub struct McpField {
    name: &'static str,
    value_type: RustType,
    presence: FieldValuePresence,
    has_default: bool,
    component_backed: bool,
    mcp_input: McpInput,
    tool_value_schema: FieldToolValueSchemaFn,
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
        }
    }

    pub const fn with_default(mut self, has_default: bool) -> Self {
        self.has_default = has_default;
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

    pub const fn required(self) -> bool {
        !self.presence.optional() && !self.has_default
    }
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

    fn tool_definition(self) -> Result<ToolDefinition, McpToolError> {
        component_shape_mcp::tool_definition(
            self.tool_name(),
            Some(self.title()),
            Some(self.description()),
            self.input_schema(),
            None,
        )
    }
}

pub trait McpForm: Sized + 'static {
    type ValueHolder: 'static;

    fn descriptor() -> McpFormDescriptor;

    fn decode_arguments(call: McpToolCall) -> Result<Self::ValueHolder, McpToolError>;

    fn validate_holder(holder: &Self::ValueHolder) -> Result<(), McpToolError>;
}

/// Contract for generated forms that can be edited over MCP.
///
/// The MCP editor tools use the same generated field decoders as submit tools,
/// so component-backed fields keep their shape-owned input metadata and
/// value-storage policy.
pub trait McpEditableForm: McpForm
where
    Self::ValueHolder: Default,
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
        component_shape_mcp::tool_definition_for_input::<Self>(
            descriptor.tool_name(),
            Some(descriptor.title()),
            Some(descriptor.description()),
            None,
        )
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
        Response: Serialize,
        Error: fmt::Display;

    fn register_async<Handler, Fut, Response, Error>(
        server: &mut McpServer,
        handler: Handler,
    ) -> Result<(), McpToolError>
    where
        Handler: Fn(Self) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, Error>> + Send + 'static,
        Response: Serialize,
        Error: fmt::Display;
}

pub mod registry {
    use super::{McpFormDescriptor, McpServer, McpToolError};

    pub use inventory;

    inventory::collect!(McpSubmitRegistration);
    inventory::collect!(McpSubmitHandlerRegistration);

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

    pub struct McpSubmitHandlerRegistration {
        register: fn(&mut McpServer) -> Result<(), McpToolError>,
    }

    impl McpSubmitHandlerRegistration {
        pub const fn new(register: fn(&mut McpServer) -> Result<(), McpToolError>) -> Self {
            Self { register }
        }

        pub fn register(&self, server: &mut McpServer) -> Result<(), McpToolError> {
            (self.register)(server)
        }
    }

    pub fn submit_handler_registrations()
    -> impl Iterator<Item = &'static McpSubmitHandlerRegistration> {
        inventory::iter::<McpSubmitHandlerRegistration>.into_iter()
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
        Response: Serialize,
        Error: fmt::Display,
    {
        insert_executor::<Form, _>(self.server, move |input| {
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
        Response: Serialize,
        Error: fmt::Display,
    {
        let handler = Arc::new(handler);
        insert_executor_async::<Form, _>(self.server, move |input| {
            let handler = Arc::clone(&handler);
            let future = handler(input.into_value_holder());
            Box::pin(async move { component_shape_mcp::serialize_handler_response(future.await) })
        })
    }
}

impl<'server, Form> FormTool<'server, Form>
where
    Form: McpEditableForm,
    Form::ValueHolder: Default + Send + 'static,
{
    /// Register stateful MCP tools that let an agent open, inspect, patch,
    /// validate, and close editable form-holder sessions.
    pub fn editor(self) -> Result<(), McpToolError> {
        insert_editor_tools::<Form>(self.server, Form::descriptor())
    }
}

impl<'server, Form> FormTool<'server, Form>
where
    Form: McpFormModel,
{
    pub fn model<Handler, Response, Error>(self, handler: Handler) -> Result<(), McpToolError>
    where
        Handler: Fn(Form) -> Result<Response, Error> + Send + Sync + 'static,
        Response: Serialize,
        Error: fmt::Display,
    {
        insert_executor::<Form, _>(self.server, move |input| {
            let model = match Form::holder_into_model(input.into_value_holder()) {
                Ok(model) => model,
                Err(error) => return component_shape_mcp::tool_error_result(error.to_string()),
            };

            component_shape_mcp::serialize_handler_response(handler(model))
        })
    }

    pub fn model_async<Handler, Fut, Response, Error>(
        self,
        handler: Handler,
    ) -> Result<(), McpToolError>
    where
        Handler: Fn(Form) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, Error>> + Send + 'static,
        Response: Serialize,
        Error: fmt::Display,
    {
        let handler = Arc::new(handler);
        insert_executor_async::<Form, _>(self.server, move |input| {
            let handler = Arc::clone(&handler);
            match Form::holder_into_model(input.into_value_holder()) {
                Ok(model) => {
                    let future = handler(model);
                    Box::pin(async move {
                        component_shape_mcp::serialize_handler_response(future.await)
                    })
                },
                Err(error) => Box::pin(std::future::ready(component_shape_mcp::tool_error_result(
                    error.to_string(),
                ))),
            }
        })
    }
}

fn insert_executor<Form, Call>(server: &mut McpServer, call: Call) -> Result<(), McpToolError>
where
    Form: McpForm,
    Call: Fn(McpFormInput<Form>) -> ToolCallResult + Send + Sync + 'static,
{
    server.add_typed_tool(McpFormInput::<Form>::tool_definition()?, call)
}

fn insert_executor_async<Form, Call>(server: &mut McpServer, call: Call) -> Result<(), McpToolError>
where
    Form: McpForm,
    Call: Fn(McpFormInput<Form>) -> ToolFuture + Send + Sync + 'static,
{
    server.add_typed_tool_async(McpFormInput::<Form>::tool_definition()?, call)
}

pub fn tool_name(source_module_path: &str, form_name: &str) -> String {
    component_shape_mcp::tool_name(source_module_path, form_name, "form_")
}

fn decode_valid_holder<Form>(call: McpToolCall) -> Result<Form::ValueHolder, McpToolError>
where
    Form: McpForm,
{
    Form::decode_arguments(call).and_then(|holder| Form::validate_holder(&holder).map(|()| holder))
}

#[derive(Debug)]
struct EditSession<Holder> {
    holder: Holder,
    present_fields: BTreeSet<String>,
    values: Map<String, Value>,
}

#[derive(Debug)]
struct EditSessions<Holder> {
    next_id: u64,
    sessions: BTreeMap<String, EditSession<Holder>>,
}

impl<Holder> Default for EditSessions<Holder> {
    fn default() -> Self {
        Self {
            next_id: 1,
            sessions: BTreeMap::new(),
        }
    }
}

impl<Holder> EditSessions<Holder> {
    fn open(
        &mut self,
        holder: Holder,
        present_fields: BTreeSet<String>,
        values: Map<String, Value>,
    ) -> String {
        let session_id = self.next_id.to_string();
        self.next_id += 1;
        self.sessions.insert(
            session_id.clone(),
            EditSession {
                holder,
                present_fields,
                values,
            },
        );
        session_id
    }

    fn get(&self, session_id: &str) -> Result<&EditSession<Holder>, McpToolError> {
        self.sessions
            .get(session_id)
            .ok_or_else(|| McpToolError::invalid_field_value("session_id", session_id))
    }

    fn get_mut(&mut self, session_id: &str) -> Result<&mut EditSession<Holder>, McpToolError> {
        self.sessions
            .get_mut(session_id)
            .ok_or_else(|| McpToolError::invalid_field_value("session_id", session_id))
    }

    fn close(&mut self, session_id: &str) -> Result<bool, McpToolError> {
        if self.sessions.remove(session_id).is_some() {
            Ok(true)
        } else {
            Err(McpToolError::invalid_field_value("session_id", session_id))
        }
    }
}

type SharedEditSessions<Holder> = Arc<Mutex<EditSessions<Holder>>>;

fn insert_editor_tools<Form>(
    server: &mut McpServer,
    descriptor: McpFormDescriptor,
) -> Result<(), McpToolError>
where
    Form: McpEditableForm,
    Form::ValueHolder: Default + Send + 'static,
{
    let sessions = Arc::new(Mutex::new(EditSessions::default()));
    let tool_names = editor_tool_names(descriptor);

    {
        let sessions = Arc::clone(&sessions);
        server.add_typed_tool(
            editor_tool_definition::<McpFormEditOpenInput<Form>>(
                tool_names.open.clone(),
                format!("Open {} edit session", descriptor.form_name()),
                format!(
                    "Create an editable {} form value-holder session.",
                    descriptor.form_name()
                ),
            )?,
            move |input| open_edit_session::<Form>(input, &sessions),
        )?;
    }

    {
        let sessions = Arc::clone(&sessions);
        server.add_typed_tool(
            editor_tool_definition::<SessionIdInput>(
                tool_names.read.clone(),
                format!("Read {} edit session", descriptor.form_name()),
                format!(
                    "Return the current values for an editable {} form session.",
                    descriptor.form_name()
                ),
            )?,
            move |input| read_edit_session::<Form>(input, &sessions),
        )?;
    }

    {
        let sessions = Arc::clone(&sessions);
        server.add_typed_tool(
            editor_tool_definition::<McpFormEditPatchInput<Form>>(
                tool_names.patch.clone(),
                format!("Patch {} field", descriptor.form_name()),
                format!(
                    "Update one field in an editable {} form session.",
                    descriptor.form_name()
                ),
            )?,
            move |input| patch_edit_session::<Form>(input, &sessions),
        )?;
    }

    {
        let sessions = Arc::clone(&sessions);
        server.add_typed_tool(
            editor_tool_definition::<SessionIdInput>(
                tool_names.validate.clone(),
                format!("Validate {} edit session", descriptor.form_name()),
                format!(
                    "Validate the current values for an editable {} form session.",
                    descriptor.form_name()
                ),
            )?,
            move |input| validate_edit_session::<Form>(input, &sessions),
        )?;
    }

    server.add_typed_tool(
        editor_tool_definition::<SessionIdInput>(
            tool_names.close,
            format!("Close {} edit session", descriptor.form_name()),
            format!(
                "Close an editable {} form value-holder session.",
                descriptor.form_name()
            ),
        )?,
        move |input| close_edit_session::<Form>(input, &sessions),
    )?;

    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpFormEditorToolNames {
    pub open: String,
    pub read: String,
    pub patch: String,
    pub validate: String,
    pub close: String,
}

pub fn editor_tool_names(descriptor: McpFormDescriptor) -> McpFormEditorToolNames {
    let prefix = descriptor.tool_name();
    McpFormEditorToolNames {
        open: format!("{prefix}_edit_open"),
        read: format!("{prefix}_edit_read"),
        patch: format!("{prefix}_edit_patch"),
        validate: format!("{prefix}_edit_validate"),
        close: format!("{prefix}_edit_close"),
    }
}

fn editor_tool_definition<Input>(
    name: String,
    title: String,
    description: String,
) -> Result<McpTypedTool<Input>, McpToolError>
where
    Input: McpToolInput,
{
    component_shape_mcp::tool_definition_for_input::<Input>(
        name,
        Some(title),
        Some(description),
        None,
    )
}

fn open_edit_session<Form>(
    input: McpFormEditOpenInput<Form>,
    sessions: &SharedEditSessions<Form::ValueHolder>,
) -> ToolCallResult
where
    Form: McpEditableForm,
    Form::ValueHolder: Default,
{
    let mut sessions = match sessions.lock() {
        Ok(sessions) => sessions,
        Err(error) => {
            return component_shape_mcp::tool_error_result(
                McpToolError::handler(format!("form edit session lock failed: {error}"))
                    .to_string(),
            );
        },
    };
    let session_id = sessions.open(input.holder, input.present_fields, input.values);
    let session = sessions
        .get(&session_id)
        .expect("opened session should be readable");
    edit_session_snapshot_result::<Form>(&session_id, session)
}

fn read_edit_session<Form>(
    input: SessionIdInput,
    sessions: &SharedEditSessions<Form::ValueHolder>,
) -> ToolCallResult
where
    Form: McpEditableForm,
    Form::ValueHolder: Default,
{
    let sessions = match sessions.lock() {
        Ok(sessions) => sessions,
        Err(error) => {
            return component_shape_mcp::tool_error_result(
                McpToolError::handler(format!("form edit session lock failed: {error}"))
                    .to_string(),
            );
        },
    };
    match sessions.get(&input.session_id) {
        Ok(session) => edit_session_snapshot_result::<Form>(&input.session_id, session),
        Err(error) => component_shape_mcp::tool_error_result(error.to_string()),
    }
}

fn patch_edit_session<Form>(
    input: McpFormEditPatchInput<Form>,
    sessions: &SharedEditSessions<Form::ValueHolder>,
) -> ToolCallResult
where
    Form: McpEditableForm,
    Form::ValueHolder: Default,
{
    let patch = input.into_patch();
    let mut sessions = match sessions.lock() {
        Ok(sessions) => sessions,
        Err(error) => {
            return component_shape_mcp::tool_error_result(
                McpToolError::handler(format!("form edit session lock failed: {error}"))
                    .to_string(),
            );
        },
    };
    let session = match sessions.get_mut(&patch.session_id) {
        Ok(session) => session,
        Err(error) => return component_shape_mcp::tool_error_result(error.to_string()),
    };

    if let Err(error) = Form::decode_field(&mut session.holder, &patch.field, patch.value.clone()) {
        return component_shape_mcp::tool_error_result(error.to_string());
    }
    session.present_fields.insert(patch.field.clone());
    session.values.insert(patch.field, patch.value.into_value());

    edit_session_snapshot_result::<Form>(&patch.session_id, session)
}

fn validate_edit_session<Form>(
    input: SessionIdInput,
    sessions: &SharedEditSessions<Form::ValueHolder>,
) -> ToolCallResult
where
    Form: McpEditableForm,
    Form::ValueHolder: Default,
{
    let sessions = match sessions.lock() {
        Ok(sessions) => sessions,
        Err(error) => {
            return component_shape_mcp::tool_error_result(
                McpToolError::handler(format!("form edit session lock failed: {error}"))
                    .to_string(),
            );
        },
    };
    let session = match sessions.get(&input.session_id) {
        Ok(session) => session,
        Err(error) => return component_shape_mcp::tool_error_result(error.to_string()),
    };

    let mut errors = required_missing_fields(Form::descriptor(), &session.present_fields)
        .into_iter()
        .map(|field| format!("missing required field `{field}`"))
        .collect::<Vec<_>>();
    if let Err(error) = Form::validate_holder(&session.holder) {
        errors.push(error.to_string());
    }

    component_shape_mcp::tool_structured_result(serde_json::json!({
        "form": Form::descriptor().tool_name(),
        "session_id": input.session_id,
        "valid": errors.is_empty(),
        "errors": errors,
    }))
}

fn close_edit_session<Form>(
    input: SessionIdInput,
    sessions: &SharedEditSessions<Form::ValueHolder>,
) -> ToolCallResult
where
    Form: McpEditableForm,
    Form::ValueHolder: Default,
{
    let mut sessions = match sessions.lock() {
        Ok(sessions) => sessions,
        Err(error) => {
            return component_shape_mcp::tool_error_result(
                McpToolError::handler(format!("form edit session lock failed: {error}"))
                    .to_string(),
            );
        },
    };
    match sessions.close(&input.session_id) {
        Ok(closed) => component_shape_mcp::tool_structured_result(serde_json::json!({
            "session_id": input.session_id,
            "closed": closed,
        })),
        Err(error) => component_shape_mcp::tool_error_result(error.to_string()),
    }
}

fn decode_open_values<Form>(
    call: McpToolCall,
) -> Result<(Form::ValueHolder, BTreeSet<String>, Map<String, Value>), McpToolError>
where
    Form: McpEditableForm,
    Form::ValueHolder: Default,
{
    let mut holder = Form::ValueHolder::default();
    let mut arguments = call.into_arguments();
    let values = arguments.take_raw("values");
    arguments.finish()?;
    let Some(values) = values else {
        return Ok((holder, BTreeSet::new(), Map::new()));
    };
    let Value::Object(values) = values else {
        return Err(McpToolError::decode("values", "expected an object"));
    };

    let mut present_fields = BTreeSet::new();
    let mut accepted_values = Map::new();
    for (field, value) in values {
        Form::decode_field(&mut holder, &field, McpAny::from(value.clone()))?;
        present_fields.insert(field.clone());
        accepted_values.insert(field, value);
    }

    Ok((holder, present_fields, accepted_values))
}

struct FieldPatch {
    session_id: String,
    field: String,
    value: McpAny,
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
    Form::ValueHolder: Default,
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

#[derive(component_shape_mcp::McpToolInput)]
#[mcp(crate = component_shape_mcp)]
struct FieldPatchInput {
    session_id: String,
    field: String,
    value: McpAny,
}

impl From<FieldPatchInput> for FieldPatch {
    fn from(input: FieldPatchInput) -> Self {
        Self {
            session_id: input.session_id,
            field: input.field,
            value: input.value,
        }
    }
}

struct McpFormEditPatchInput<Form>
where
    Form: McpForm,
{
    patch: FieldPatch,
    _form: PhantomData<fn() -> Form>,
}

impl<Form> McpFormEditPatchInput<Form>
where
    Form: McpForm,
{
    fn into_patch(self) -> FieldPatch {
        self.patch
    }
}

impl<Form> McpToolInput for McpFormEditPatchInput<Form>
where
    Form: McpForm,
{
    fn input_schema() -> McpSchema {
        patch_editor_input_schema(Form::descriptor().fields())
    }

    fn from_tool_call(call: McpToolCall) -> Result<Self, McpToolError> {
        Ok(Self {
            patch: <FieldPatchInput as McpToolInput>::from_tool_call(call)?.into(),
            _form: PhantomData,
        })
    }
}

#[derive(component_shape_mcp::McpToolInput)]
#[mcp(crate = component_shape_mcp)]
struct SessionIdInput {
    session_id: String,
}

fn edit_session_snapshot_result<Form>(
    session_id: &str,
    session: &EditSession<Form::ValueHolder>,
) -> ToolCallResult
where
    Form: McpEditableForm,
    Form::ValueHolder: Default,
{
    match edit_session_snapshot::<Form>(session_id, session) {
        Ok(value) => component_shape_mcp::tool_structured_result(value),
        Err(error) => component_shape_mcp::tool_error_result(error.to_string()),
    }
}

fn edit_session_snapshot<Form>(
    session_id: &str,
    session: &EditSession<Form::ValueHolder>,
) -> Result<Value, McpToolError>
where
    Form: McpEditableForm,
    Form::ValueHolder: Default,
{
    let descriptor = Form::descriptor();

    Ok(serde_json::json!({
        "form": descriptor.tool_name(),
        "form_name": descriptor.form_name(),
        "session_id": session_id,
        "present_fields": session.present_fields.iter().collect::<Vec<_>>(),
        "missing_required": required_missing_fields(descriptor, &session.present_fields),
        "values": session.values,
        "submit_arguments": session.values,
    }))
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

/// Build a server containing every inventory-discovered form submit handler.
pub fn server() -> Result<McpServer, McpToolError> {
    builder().build()
}

/// Serve every inventory-discovered form submit handler over stdio.
pub async fn serve_stdio() -> ServeStdioResult {
    server()?.serve_stdio().await
}

/// Serve every inventory-discovered form submit handler over stdio from a
/// synchronous `main`.
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

/// Build a server builder containing every inventory-discovered form submit handler.
pub fn builder() -> McpServerBuilder {
    builder_named(DEFAULT_SERVER_NAME, env!("CARGO_PKG_VERSION"))
}

/// Build a generated form submit server builder with application-owned metadata.
pub fn builder_named(
    server_name: impl Into<std::borrow::Cow<'static, str>>,
    server_version: impl Into<std::borrow::Cow<'static, str>>,
) -> McpServerBuilder {
    McpServer::builder(server_name, server_version).register(register)
}

/// Add every inventory-discovered form submit handler to a shared MCP server.
pub fn register(server: &mut McpServer) -> Result<(), McpToolError> {
    for registration in registry::submit_handler_registrations() {
        registration.register(server)?;
    }
    Ok(())
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
    properties.insert(
        "session_id".to_string(),
        McpSchema::new(serde_json::json!({ "type": "string" })),
    );
    properties.insert(
        "field".to_string(),
        McpSchema::new(serde_json::json!({
            "type": "string",
            "enum": fields.iter().map(|field| field.name()).collect::<Vec<_>>()
        })),
    );
    properties.insert("value".to_string(), McpSchema::new(serde_json::json!({})));
    let mut schema =
        component_shape_mcp::object_schema(properties, ["session_id", "field", "value"]);

    if let Some(object) = schema.as_object_mut() {
        object.insert(
            "oneOf".to_string(),
            Value::Array(
                fields
                    .iter()
                    .map(|field| {
                        serde_json::json!({
                            "properties": {
                                "field": { "const": field.name() },
                                "value": schema_for_field(*field).into_value()
                            }
                        })
                    })
                    .collect(),
            ),
        );
    }

    schema
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
    }

    schema
}

/// Return MCP form tool definitions for every derived descriptor, failing on duplicate names.
pub fn tool_definitions() -> Result<Vec<ToolDefinition>, McpToolError> {
    let mut seen = BTreeSet::new();
    let mut tools = Vec::new();
    for registration in registry::submit_registrations() {
        let definition = registration.descriptor().tool_definition()?;
        let name = definition.name.to_string();
        if !seen.insert(name.clone()) {
            return Err(McpToolError::duplicate_tool(name));
        }
        tools.push(definition);
    }
    Ok(tools)
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
            .holder(|holder| Ok::<_, String>(holder.available_on))
            .expect("tool should register");

        let result = server.call_tool(
            &Booking::descriptor().tool_name(),
            Some(serde_json::json!({ "available_on": "2026-06-10" })),
        );

        assert_eq!(result.is_error, Some(false));
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

    struct Booking;

    impl super::McpForm for Booking {
        type ValueHolder = BookingHolder;

        fn descriptor() -> McpFormDescriptor {
            const FIELDS: &[McpField] = &[McpField::typed::<String>(
                "available_on",
                RustType::from_macro_tokens_unchecked("String"),
                FieldValuePresence::DirectStorage,
            )];

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
}
