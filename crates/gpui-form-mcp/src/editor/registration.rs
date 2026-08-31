use std::sync::{Arc, Mutex};

use crate::*;

pub(crate) fn insert_editor_tools<Form>(
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

pub(crate) fn insert_editor_tools_with_session_submit<Form, Response, Call>(
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

pub(crate) fn insert_editor_tools_with_session_submit_async<Form, Response, Call>(
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

pub(crate) fn insert_editor_tools_with_sessions<Form>(
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

pub(crate) struct McpFormEditorToolDefinitions<Form>
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

pub(crate) fn editor_typed_tool_definitions<Form>(
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

pub(crate) fn editor_tool_definition<Input>(
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

pub(crate) fn read_only_annotations() -> McpToolAnnotations {
    McpToolAnnotations::new()
        .read_only(true)
        .destructive(false)
        .idempotent(true)
        .open_world(false)
}

pub(crate) fn session_mutation_annotations(destructive: bool) -> McpToolAnnotations {
    McpToolAnnotations::new()
        .read_only(false)
        .destructive(destructive)
        .idempotent(false)
        .open_world(false)
}

pub(crate) fn session_submit_tool_definition<Response>(
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
