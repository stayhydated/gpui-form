use std::{
    collections::BTreeSet,
    marker::PhantomData,
    sync::Arc,
    time::{Duration, Instant},
};

use serde_json::{Map, Value};

use crate::*;

pub(crate) fn open_edit_session<Form>(
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

pub(crate) fn list_edit_sessions<Form>(
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
    let cleanup = EditSessionCleanup {
        expired_session_ids: sessions.expire_idle_sessions(Instant::now()),
        evicted_session_ids: Vec::new(),
    };

    match edit_session_list_snapshot::<Form>(&sessions, cleanup) {
        Ok(value) => component_shape_mcp::tool_structured_result(value),
        Err(error) => component_shape_mcp::tool_error_result_for(error),
    }
}

pub(crate) fn read_edit_session<Form>(
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

pub(crate) fn patch_edit_session<Form>(
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

pub(crate) fn validate_edit_session<Form>(
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

pub(crate) fn submit_edit_session<Form, Call>(
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

pub(crate) fn submit_edit_session_async<Form, Call>(
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

pub(crate) fn edit_session_submit_holder<Form>(
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

pub(crate) fn close_submitted_edit_session_on_success<Holder>(
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

pub(crate) fn close_edit_session<Form>(
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

pub(crate) fn close_all_edit_sessions<Form>(
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
    let descriptor = Form::descriptor();
    let session_ids = sessions.close_all();
    component_shape_mcp::tool_structured_result(serde_json::json!({
        "form": descriptor.tool_name(),
        "form_name": descriptor.form_name(),
        "closed_count": session_ids.len(),
        "session_ids": session_ids,
    }))
}

pub(crate) fn decode_open_values<Form>(
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

pub(crate) fn edit_session_from_values<Form>(
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

pub(crate) struct FormPatch {
    session_id: String,
    values: Map<String, Value>,
    clear: Vec<String>,
    replace: bool,
    expected_revision: Option<u64>,
}

pub(crate) struct SubmittedEditSession<Holder> {
    holder: Holder,
    revision: u64,
}

pub(crate) struct McpFormEditOpenInput<Form>
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

pub(crate) struct McpFormEditPatchInput<Form>
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
pub(crate) struct SessionIdInput {
    session_id: String,
}

pub(crate) struct McpFormEditCloseInput {
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

pub(crate) struct McpFormEditSubmitInput {
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

pub(crate) fn edit_session_snapshot_result<Form>(
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

pub(crate) fn edit_session_snapshot<Form>(
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

pub(crate) fn edit_session_list_snapshot<Form>(
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

pub(crate) fn insert_edit_session_cleanup(value: &mut Value, cleanup: EditSessionCleanup) {
    if let Value::Object(object) = value {
        object.insert("cleanup".to_string(), edit_session_cleanup_value(cleanup));
    }
}

pub(crate) fn edit_session_cleanup_value(cleanup: EditSessionCleanup) -> Value {
    serde_json::json!({
        "expired_count": cleanup.expired_session_ids.len(),
        "expired_session_ids": cleanup.expired_session_ids,
        "evicted_count": cleanup.evicted_session_ids.len(),
        "evicted_session_ids": cleanup.evicted_session_ids,
    })
}

pub(crate) fn duration_millis_saturating(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

pub(crate) fn edit_session_fields(
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

pub(crate) fn edit_session_validation_issues<Form>(
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

pub(crate) fn validation_issue_values(issues: &[McpValidationIssue]) -> Vec<Value> {
    issues
        .iter()
        .map(McpValidationIssue::to_value)
        .collect::<Vec<_>>()
}

pub(crate) fn required_missing_fields(
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
