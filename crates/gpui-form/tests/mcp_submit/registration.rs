use super::*;

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
    #[cfg(feature = "runtime")]
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
    #[cfg(feature = "runtime")]
    {
        let component_editor_names = editor_tool_names(ComponentRequest::descriptor());
        assert!(registered_names.contains(&component_editor_names.open));
        assert!(registered_names.contains(&component_editor_names.list));
        assert!(registered_names.contains(&component_editor_names.patch));
        assert!(registered_names.contains(&component_editor_names.close_all));
        assert!(!registered_names.contains(&component_editor_names.submit));
    }

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
