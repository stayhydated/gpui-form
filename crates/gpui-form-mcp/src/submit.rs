use std::{
    any::TypeId, collections::BTreeSet, fmt, future::Future, marker::PhantomData, pin::Pin,
    sync::Arc,
};

use crate::*;

pub(crate) type ToolFuture = Pin<Box<dyn Future<Output = ToolCallResult> + Send + 'static>>;

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

pub(crate) fn register_context_submitters_report<Context>(
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

pub(crate) fn context_submit_registration_count<Context>() -> usize
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

pub(crate) fn insert_executor<Form, Response, Call>(
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

pub(crate) fn insert_executor_async<Form, Response, Call>(
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

pub(crate) fn decode_valid_holder<Form>(
    call: McpToolCall,
) -> Result<Form::ValueHolder, McpToolError>
where
    Form: McpForm,
{
    Form::decode_arguments(call).and_then(|holder| {
        <Form as McpForm>::validate_holder_issues(&holder)
            .map(|()| holder)
            .map_err(validation_issues_error)
    })
}

pub(crate) fn ensure_form_tool_and_editor_submit_tools_available<Form, Response>(
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
