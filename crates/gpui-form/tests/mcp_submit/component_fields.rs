use super::*;

#[cfg(feature = "runtime")]
#[test]
fn component_backed_field_decodes_through_shape_schema() {
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

#[cfg(feature = "runtime")]
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

#[cfg(feature = "runtime")]
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
