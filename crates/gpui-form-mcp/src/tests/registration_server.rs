use super::*;
use crate::{McpForm as _, McpToolInput as _};

#[test]
fn registration_profiles_select_the_intended_tool_families() {
    let all = McpFormRegistrationOptions::default();
    assert_eq!(all, McpFormRegistrationOptions::all());
    assert!(all.registers_tools());

    let submit = McpFormRegistrationOptions::submit_only();
    assert!(submit.submit_handlers);
    assert!(!submit.editors);
    assert!(!submit.edit_submit);
    assert!(submit.registers_tools());

    let editor = McpFormRegistrationOptions::editor_only();
    assert!(!editor.submit_handlers);
    assert!(editor.editors);
    assert!(!editor.edit_submit);
    assert!(editor.registers_tools());

    let metadata = McpFormRegistrationOptions::metadata_only();
    assert!(!metadata.registers_tools());
}

#[test]
fn metadata_only_registration_reports_do_not_mutate_servers() {
    let definitions =
        crate::tool_definitions_with_options(McpFormRegistrationOptions::metadata_only()).unwrap();
    let mut server = McpServer::new("metadata-only".to_owned(), "1.0.0");
    let report =
        crate::register_with_options(&mut server, McpFormRegistrationOptions::metadata_only())
            .unwrap();

    assert_eq!(report.tool_definitions.len(), definitions.len());
    assert!(report.registered_tool_names.is_empty());
    assert_eq!(report.context_registration_count, 0);
    assert_eq!(server.tool_count(), 0);
}

#[test]
fn server_accepts_custom_metadata() {
    let mut server = McpServer::new("example-form-server".to_string(), "9.9.9");
    let descriptor = Booking::descriptor();

    assert_eq!(descriptor.tool_name(), "reserve_booking");
    assert_eq!(descriptor.title(), "Reserve booking");
    assert_eq!(descriptor.description(), "Reserve a booking window.");

    form::<Booking>(&mut server)
        .holder(|holder| {
            Ok::<_, String>(BookingResponse {
                available_on: holder.available_on,
            })
        })
        .expect("tool should register");
    let resource_uris = crate::form_resource_uris(descriptor);
    assert_eq!(server.resource_count(), 3);
    for uri in resource_uris.all() {
        assert!(server.contains_resource(uri));
    }
    let resources = server.list_resources();
    assert!(
        resources
            .iter()
            .any(|resource| resource.uri == resource_uris.descriptor
                && resource.mime_type.as_deref() == Some("application/json"))
    );

    let result = server.call_tool(
        &Booking::descriptor().tool_name(),
        Some(serde_json::json!({ "available_on": "2026-06-10" })),
    );

    assert_eq!(result.is_error, Some(false));
    assert_eq!(
        result.structured_content,
        Some(serde_json::json!({ "available_on": "2026-06-10" }))
    );
}

#[test]
fn form_input_pairs_descriptor_schema_with_generated_decode() {
    let tool = McpFormInput::<Booking>::tool_definition().expect("tool should build");
    fn assert_typed_tool(_tool: &crate::McpTypedTool<crate::McpFormInput<Booking>>) {}
    assert_typed_tool(&tool);
    assert_eq!(
        tool.input_schema["properties"]["available_on"]["type"],
        "string"
    );

    let input = McpFormInput::<Booking>::from_tool_call(
        crate::McpToolCall::from_value(Some(serde_json::json!({
            "available_on": "2026-06-10"
        })))
        .expect("arguments should normalize"),
    )
    .expect("form input should decode");

    assert_eq!(input.into_value_holder().available_on, "2026-06-10");
}
