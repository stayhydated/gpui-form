//! Experimental MCP submit integration for generated `gpui-form` value holders.
//!
//! This crate intentionally keeps GPUI out of the submit execution path. It
//! owns form holder/model decoding and validation while delegating shared MCP
//! server and stdio serving mechanics to `component-shape-mcp`.

use std::{collections::BTreeSet, fmt, future::Future, marker::PhantomData, sync::Arc};

use gpui_form_schema::registry::{FieldValuePresence, RustPath, RustType};
pub use serde::Serialize;
use serde_json::{Map, Value};

pub use component_shape_mcp::{
    ComponentShapeFor, ComponentShapeMetadata, ContentBlock, MCP_PROTOCOL_VERSION, McpArguments,
    McpInput, McpInputShape, McpJsonSchema, McpPrimitiveKind, McpRange, McpServer,
    McpServerBuilder, McpToolArguments, McpToolCall, McpToolError, McpToolMetadata,
    ServeStdioResult, ToolCallResult, ToolDefinition, object_schema, rmcp, serde_json,
};

pub type FieldSchemaFn = fn() -> Value;

#[derive(Clone, Copy, Debug)]
pub struct McpField {
    name: &'static str,
    value_type: RustType,
    presence: FieldValuePresence,
    has_default: bool,
    component_backed: bool,
    mcp_input: McpInput,
    json_schema: Option<FieldSchemaFn>,
}

impl McpField {
    pub const fn typed<T>(
        name: &'static str,
        value_type: RustType,
        presence: FieldValuePresence,
    ) -> Self
    where
        T: McpJsonSchema,
    {
        Self {
            name,
            value_type,
            presence,
            has_default: false,
            component_backed: false,
            mcp_input: McpInput::unsupported(),
            json_schema: Some(T::json_schema),
        }
    }

    pub const fn component<T, Shape>(
        name: &'static str,
        value_type: RustType,
        presence: FieldValuePresence,
    ) -> Self
    where
        T: McpJsonSchema,
        Shape: ComponentShapeFor<T>,
    {
        Self {
            name,
            value_type,
            presence,
            has_default: false,
            component_backed: true,
            mcp_input: <Shape as ComponentShapeFor<T>>::MCP_INPUT,
            json_schema: Some(T::json_schema),
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

    pub const fn json_schema(self) -> Option<FieldSchemaFn> {
        self.json_schema
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

    pub fn input_schema(self) -> Value {
        input_schema_for_fields(self.fields)
    }

    pub fn tool_definition(self) -> Result<ToolDefinition, McpToolError> {
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
        insert_executor(self.server, Form::descriptor(), move |arguments| {
            let holder = match decode_valid_holder::<Form>(arguments) {
                Ok(holder) => holder,
                Err(error) => return component_shape_mcp::tool_error_result(error.to_string()),
            };

            component_shape_mcp::serialize_handler_response(handler(holder))
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
        insert_executor_async(self.server, Form::descriptor(), move |arguments| {
            let handler = Arc::clone(&handler);
            async move {
                let holder = match decode_valid_holder::<Form>(arguments) {
                    Ok(holder) => holder,
                    Err(error) => return component_shape_mcp::tool_error_result(error.to_string()),
                };

                component_shape_mcp::serialize_handler_response(handler(holder).await)
            }
        })
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
        insert_executor(self.server, Form::descriptor(), move |arguments| {
            let model = match decode_valid_model::<Form>(arguments) {
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
        insert_executor_async(self.server, Form::descriptor(), move |arguments| {
            let handler = Arc::clone(&handler);
            async move {
                let model = match decode_valid_model::<Form>(arguments) {
                    Ok(model) => model,
                    Err(error) => return component_shape_mcp::tool_error_result(error.to_string()),
                };

                component_shape_mcp::serialize_handler_response(handler(model).await)
            }
        })
    }
}

fn insert_executor<Call>(
    server: &mut McpServer,
    descriptor: McpFormDescriptor,
    call: Call,
) -> Result<(), McpToolError>
where
    Call: Fn(McpToolCall) -> ToolCallResult + Send + Sync + 'static,
{
    server.add_tool(descriptor.tool_definition()?, call)
}

fn insert_executor_async<Call, Fut>(
    server: &mut McpServer,
    descriptor: McpFormDescriptor,
    call: Call,
) -> Result<(), McpToolError>
where
    Call: Fn(McpToolCall) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ToolCallResult> + Send + 'static,
{
    server.add_tool_async(descriptor.tool_definition()?, call)
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

fn decode_valid_model<Form>(call: McpToolCall) -> Result<Form, McpToolError>
where
    Form: McpFormModel,
{
    decode_valid_holder::<Form>(call).and_then(Form::holder_into_model)
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

fn input_schema_for_fields(fields: &[McpField]) -> Value {
    let mut properties = Map::new();
    let mut required = Vec::new();

    for field in fields {
        properties.insert(field.name().to_string(), schema_for_field(*field));
        if field.required() {
            required.push(field.name());
        }
    }

    component_shape_mcp::object_schema(properties, required)
}

fn schema_for_field(field: McpField) -> Value {
    let base = if field.mcp_input().supported() {
        component_shape_mcp::schema_for_input(field.mcp_input())
    } else if let Some(json_schema) = field.json_schema() {
        json_schema()
    } else {
        component_shape_mcp::schema_for_input(McpInput::unsupported())
    };
    let mut schema = if field.presence().optional() {
        component_shape_mcp::nullable_schema(base)
    } else {
        base
    };

    if let Value::Object(object) = &mut schema {
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
    use super::{McpField, McpForm as _, McpFormDescriptor, McpInput, McpServer, form, tool_name};
    use gpui_form_schema::registry::{FieldValuePresence, RustPath, RustType};

    struct StringComponentShape;

    impl super::ComponentShapeMetadata for StringComponentShape {
        const MCP_INPUT: McpInput = McpInput::number();
    }
    impl super::ComponentShapeFor<String> for StringComponentShape {
        const MCP_INPUT: McpInput = McpInput::string();
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
    fn input_schema_prefers_component_mcp_input_when_present() {
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
    fn input_schema_uses_field_json_schema_when_no_component_input_exists() {
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
    fn input_schema_keeps_component_mcp_input_ahead_of_field_json_schema() {
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
                .take_present::<String>("available_on")?
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
