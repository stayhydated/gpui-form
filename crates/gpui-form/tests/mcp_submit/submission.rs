use super::*;

#[test]
fn tool_call_decodes_validates_converts_and_submits_model() {
    let mut server = test_server();
    form::<PlainRequest>(&mut server)
        .model(|request| {
            Ok::<ObjectResponse, String>(object_response(json!({
                "title": request.title,
                "retries": request.retries,
                "enabled": request.enabled
            })))
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
fn context_submitters_register_from_derive_inventory() {
    let context = SubmitContext {
        prefix: "ctx:".to_string(),
    };
    let definitions =
        context_submit_tool_definitions::<SubmitContext>().expect("definitions should build");
    let definition_names: Vec<_> = definitions
        .iter()
        .map(|definition| definition.name.to_string())
        .collect();
    let submit_name = ContextRequest::descriptor().tool_name();
    let editor_names = editor_tool_names(ContextRequest::descriptor());
    let dynamic_submit_name = DynamicContextRequest::descriptor().tool_name();
    assert!(definition_names.contains(&submit_name));
    assert!(definition_names.contains(&editor_names.open));
    assert!(definition_names.contains(&editor_names.submit));
    assert!(definition_names.contains(&dynamic_submit_name));

    let submit_definition = definitions
        .iter()
        .find(|definition| definition.name.as_ref() == submit_name)
        .expect("context submit definition should be present");
    let output_schema = Value::Object(
        submit_definition
            .output_schema
            .as_ref()
            .expect("context submit should publish an output schema")
            .as_ref()
            .clone(),
    );
    assert_eq!(
        output_schema["properties"]["prefixed"]["type"],
        json!("string")
    );
    let dynamic_submit_definition = definitions
        .iter()
        .find(|definition| definition.name.as_ref() == dynamic_submit_name)
        .expect("dynamic context submit definition should be present");
    let dynamic_output_schema = Value::Object(
        dynamic_submit_definition
            .output_schema
            .as_ref()
            .expect("dynamic context submit should publish an output schema")
            .as_ref()
            .clone(),
    );
    assert_eq!(dynamic_output_schema["type"], json!("object"));

    let mut server = test_server();
    register_context_submitters_with_editor_options(
        &mut server,
        context,
        McpFormEditorOptions::default().with_session_limit(4),
    )
    .expect("context submitters should register");

    assert!(server.contains_tool(&submit_name));
    assert!(server.contains_tool(&editor_names.open));
    assert!(server.contains_tool(&editor_names.submit));
    assert!(server.contains_tool(&dynamic_submit_name));

    let direct_submit = server.call_tool(&submit_name, Some(json!({ "title": "Direct" })));
    assert_eq!(direct_submit.is_error, Some(false));
    assert_eq!(
        direct_submit.structured_content,
        Some(json!({
            "title": "Direct",
            "prefixed": "ctx:Direct"
        }))
    );
    let dynamic_submit =
        server.call_tool(&dynamic_submit_name, Some(json!({ "title": "Dynamic" })));
    assert_eq!(dynamic_submit.is_error, Some(false));
    assert_eq!(
        dynamic_submit.structured_content,
        Some(json!({
            "title": "Dynamic",
            "prefixed": "ctx:Dynamic"
        }))
    );

    let opened = server.call_tool(
        &editor_names.open,
        Some(json!({ "values": { "title": "Editable" } })),
    );
    assert_eq!(opened.is_error, Some(false));
    let opened = opened
        .structured_content
        .expect("context edit open should return state");
    assert_eq!(opened["submit_arguments"], json!({ "title": "Editable" }));
    let session_id = opened["session_id"]
        .as_str()
        .expect("session id should be a string");

    let edit_submit = server.call_tool(
        &editor_names.submit,
        Some(json!({
            "session_id": session_id,
            "expected_revision": 0
        })),
    );
    assert_eq!(edit_submit.is_error, Some(false));
    assert_eq!(
        edit_submit.structured_content,
        Some(json!({
            "title": "Editable",
            "prefixed": "ctx:Editable"
        }))
    );
}

#[test]
fn submitter_trait_context_submitters_register_from_derive_inventory() {
    let context = TraitSubmitContext {
        prefix: "trait:".to_string(),
    };
    let definitions =
        context_submit_tool_definitions::<TraitSubmitContext>().expect("definitions should build");
    let definition_names: Vec<_> = definitions
        .iter()
        .map(|definition| definition.name.to_string())
        .collect();
    let submit_name = TraitBackedContextRequest::descriptor().tool_name();
    let editor_names = editor_tool_names(TraitBackedContextRequest::descriptor());
    assert!(definition_names.contains(&submit_name));
    assert!(definition_names.contains(&editor_names.submit));

    let submit_definition = definitions
        .iter()
        .find(|definition| definition.name.as_ref() == submit_name)
        .expect("trait-backed context submit definition should be present");
    let output_schema = Value::Object(
        submit_definition
            .output_schema
            .as_ref()
            .expect("trait-backed context submit should publish an output schema")
            .as_ref()
            .clone(),
    );
    assert_eq!(
        output_schema["properties"]["prefixed"]["type"],
        json!("string")
    );

    let mut server = test_server();
    register_context_submitters_with_editor_options(
        &mut server,
        context,
        McpFormEditorOptions::default().with_session_limit(4),
    )
    .expect("trait-backed context submitters should register");

    assert!(server.contains_tool(&submit_name));
    assert!(server.contains_tool(&editor_names.submit));

    let direct_submit = server.call_tool(&submit_name, Some(json!({ "title": "Direct" })));
    assert_eq!(direct_submit.is_error, Some(false));
    assert_eq!(
        direct_submit.structured_content,
        Some(json!({
            "title": "Direct",
            "prefixed": "trait:Direct"
        }))
    );
}

#[test]
fn mapped_submitter_trait_context_submitters_register_from_derive_inventory() {
    let context = TraitSubmitContext {
        prefix: "mapped:".to_string(),
    };
    let definitions =
        context_submit_tool_definitions::<TraitSubmitContext>().expect("definitions should build");
    let submit_name = MappedTraitBackedContextRequest::descriptor().tool_name();
    let submit_definition = definitions
        .iter()
        .find(|definition| definition.name.as_ref() == submit_name)
        .expect("mapped trait-backed context submit definition should be present");
    let output_schema = Value::Object(
        submit_definition
            .output_schema
            .as_ref()
            .expect("mapped trait-backed context submit should publish an output schema")
            .as_ref()
            .clone(),
    );
    assert_eq!(output_schema["type"], json!("object"));

    let mut server = test_server();
    register_context_submitters_with_editor_options(
        &mut server,
        context,
        McpFormEditorOptions::default().with_session_limit(4),
    )
    .expect("mapped trait-backed context submitters should register");

    let direct_submit = server.call_tool(&submit_name, Some(json!({ "title": "Raw" })));
    assert_eq!(direct_submit.is_error, Some(false));
    assert_eq!(
        direct_submit.structured_content,
        Some(json!({
            "title": "Raw",
            "prefixed": "mapped:Raw"
        }))
    );
}

#[test]
fn strict_context_submitters_report_matches_and_reject_zero_matches() {
    #[derive(Clone)]
    struct MissingContext;

    let mut missing_server = test_server();
    let error = register_context_submitters_strict(&mut missing_server, MissingContext)
        .expect_err("strict context registration should reject zero matches");
    assert_eq!(error.kind(), "validation");
    assert!(
        error
            .to_string()
            .contains("no MCP context submitters registered for context")
    );
    assert_eq!(missing_server.tool_count(), 0);

    let mut server = test_server();
    let report = register_context_submitters_strict(
        &mut server,
        SubmitContext {
            prefix: "strict:".to_string(),
        },
    )
    .expect("strict context submitters should register matching inventory");

    let submit_name = ContextRequest::descriptor().tool_name();
    let editor_names = editor_tool_names(ContextRequest::descriptor());
    assert_eq!(report.context_registration_count, 2);
    assert!(report.registered_tool_names.contains(&submit_name));
    assert!(report.registered_tool_names.contains(&editor_names.submit));
    assert!(server.contains_tool(&submit_name));
    assert!(server.contains_tool(&editor_names.submit));
}

#[test]
fn skipped_fields_use_holder_submission() {
    let mut server = test_server();
    form::<SkippedRequest>(&mut server)
        .holder(|holder| {
            Ok::<ObjectResponse, String>(object_response(json!({
                "name": holder.name,
                "tenant": "application-owned"
            })))
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
