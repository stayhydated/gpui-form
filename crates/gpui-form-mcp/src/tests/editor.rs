use super::*;
use crate::McpForm as _;

#[test]
fn editor_options_bound_sessions_and_allow_explicit_unbounded_modes() {
    let defaults = McpFormEditorOptions::default();
    assert_eq!(
        defaults.session_limit(),
        Some(crate::DEFAULT_EDITOR_SESSION_LIMIT)
    );
    assert_eq!(
        defaults.session_idle_timeout(),
        Some(crate::DEFAULT_EDITOR_SESSION_IDLE_TIMEOUT)
    );

    let bounded = McpFormEditorOptions::new()
        .with_session_limit(0)
        .with_session_idle_timeout(Duration::ZERO);
    assert_eq!(bounded.session_limit(), Some(1));
    assert_eq!(bounded.session_idle_timeout(), Some(Duration::ZERO));

    let unbounded = bounded
        .without_session_limit()
        .without_session_idle_timeout();
    assert_eq!(unbounded.session_limit(), None);
    assert_eq!(unbounded.session_idle_timeout(), None);
}

#[test]
fn editor_schema_helpers_publish_typed_protocol_contracts() {
    let descriptor = Booking::descriptor();
    let fields = descriptor.fields();

    let open = crate::open_editor_input_schema(fields);
    assert_eq!(open["type"], "object");
    assert_eq!(open["properties"]["values"]["type"], "object");

    let patch = crate::patch_editor_input_schema(fields);
    assert_eq!(patch["required"], serde_json::json!(["session_id"]));
    assert_eq!(patch["properties"]["replace"]["default"], false);
    assert_eq!(
        patch["properties"]["clear"]["items"]["enum"],
        serde_json::json!(["available_on"])
    );

    let snapshot = crate::edit_session_snapshot_output_schema(fields);
    assert_eq!(snapshot["properties"]["revision"]["minimum"], 0);
    assert_eq!(
        snapshot["properties"]["fields"]["items"]["oneOf"][0]["properties"]["name"]["enum"],
        serde_json::json!(["available_on"])
    );

    let list = crate::edit_session_list_output_schema(fields);
    assert_eq!(list["properties"]["session_count"]["minimum"], 0);
    assert_eq!(list["properties"]["sessions"]["type"], "array");

    let cleanup = crate::edit_session_cleanup_schema();
    assert_eq!(cleanup["properties"]["expired_count"]["minimum"], 0);
    assert_eq!(
        cleanup["properties"]["evicted_session_ids"]["type"],
        "array"
    );

    let close = crate::close_editor_output_schema();
    assert_eq!(
        close["required"],
        serde_json::json!(["session_id", "closed"])
    );

    let close_all = crate::close_all_editor_output_schema(descriptor);
    assert_eq!(close_all["properties"]["form"]["const"], "reserve_booking");
    assert_eq!(close_all["properties"]["form_name"]["const"], "Booking");

    let empty_fields = crate::edit_session_fields_output_schema(&[]);
    assert_eq!(empty_fields["type"], "array");
    assert_eq!(empty_fields["items"]["type"], "object");

    let issue = crate::validation_issue_schema();
    assert_eq!(
        issue["properties"]["scope"]["enum"],
        serde_json::json!(["form", "field", "element"])
    );
    assert_eq!(issue["required"], serde_json::json!(["scope", "message"]));

    let parameter = crate::validation_param_schema();
    assert_eq!(parameter["required"], serde_json::json!(["name"]));
    assert_eq!(crate::expected_revision_schema()["minimum"], 0);
    assert_eq!(crate::session_limit_schema()["anyOf"][0]["minimum"], 1);
    assert_eq!(
        crate::session_idle_timeout_schema()["anyOf"][0]["minimum"],
        0
    );
}
